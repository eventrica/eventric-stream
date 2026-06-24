//! The [`Owner`] — holds a [`Stream`]'s dedicated writer
//! thread and hands out [`Proxy`] clones for concurrent access.

use std::thread::{
    self,
    JoinHandle,
};

use crossbeam::channel;
use error_stack::Report;
use fancy_constructor::new;

use super::{
    processor::{
        Operation,
        Processor,
    },
    proxy::Proxy,
};
use crate::{
    error::Error,
    stream::{
        Reader,
        Stream,
        Writer,
    },
};

// =================================================================================================
// Owner
// =================================================================================================

/// Owns a [`Stream`] across a dedicated writer thread,
/// handing out [`Proxy`] clones for concurrent access. The unique holder of the
/// stream's `Writer`; reclaim the underlying stream with [`Owner::into_inner`].
#[derive(new, Debug)]
#[new(const_fn, name(new_inner), vis())]
pub struct Owner {
    handle: JoinHandle<Result<Writer, Report<Error>>>,
    reader: Reader,
    sender: channel::Sender<Operation>,
}

impl Owner {
    /// Take ownership of `stream`, spawning the dedicated writer thread that
    /// serialises all writes.
    #[must_use]
    pub fn new(stream: Stream) -> Self {
        let stream = stream.split();
        let channel = channel::bounded::<Operation>(128);

        let handle = thread::spawn(move || Processor::new(channel.1, stream.1).process());
        let reader = stream.0;
        let sender = channel.0;

        Self::new_inner(handle, reader, sender)
    }
}

impl Owner {
    /// Create a [`Proxy`] — a cheaply-cloneable handle for concurrent reads and
    /// (channelled) writes against this owner's stream.
    #[must_use]
    pub fn proxy(&self) -> Proxy {
        Proxy::new(self.reader.clone(), self.sender.clone())
    }
}

impl Owner {
    /// Shut the writer thread down and reclaim the underlying [`Stream`].
    ///
    /// # Errors
    ///
    /// Returns an error if the writer thread cannot be signalled or joined.
    pub fn into_inner(self) -> Result<Stream, Report<Error>> {
        self.sender
            .send(Operation::Exit)
            .map_err(|_| Report::new(Error).attach("owner/into_inner/send"))?;

        self.handle
            .join()
            .map_err(|_| Report::new(Error).attach("owner/into_inner/join"))
            .flatten()
            .map(Into::into)
    }
}
