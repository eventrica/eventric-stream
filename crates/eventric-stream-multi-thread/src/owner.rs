use std::thread::{
    self,
    JoinHandle,
};

use crossbeam::channel;
use eventric_stream_core::{
    error::Error,
    stream::{
        Reader,
        Stream,
        Writer,
    },
};
use fancy_constructor::new;

use crate::{
    processor::{
        Operation,
        Processor,
    },
    proxy::Proxy,
};

// =================================================================================================
// Owner
// =================================================================================================

#[derive(new, Debug)]
#[new(const_fn, name(new_inner), vis())]
pub struct Owner {
    handle: JoinHandle<Result<Writer, Error>>,
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
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn into_inner(self) -> Result<Stream, Error> {
        self.sender
            .send(Operation::Exit)
            .map_err(|_| Error::general("owner/into_inner/send"))?;

        self.handle
            .join()
            .map_err(|_| Error::general("owner/into_inner/join"))
            .flatten()
            .map(Into::into)
    }
}
