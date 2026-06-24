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

#[derive(new, Debug)]
#[new(const_fn, name(new_inner), vis())]
pub struct Owner {
    handle: JoinHandle<Result<Writer, Report<Error>>>,
    reader: Reader,
    sender: channel::Sender<Operation>,
}

impl Owner {
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
