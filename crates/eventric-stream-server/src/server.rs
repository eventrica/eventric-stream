use std::thread::{
    self,
    JoinHandle,
};

use crossbeam::channel;
use eventric_stream::{
    error::Error,
    stream::{
        Reader,
        Stream,
        Writer,
    },
};
use fancy_constructor::new;

use crate::{
    client::Client,
    processor::{
        Operation,
        Processor,
    },
};

// =================================================================================================
// Server
// =================================================================================================

#[derive(new, Debug)]
#[new(const_fn, name(new_inner), vis())]
pub struct Server {
    handle: JoinHandle<Result<Writer, Error>>,
    reader: Reader,
    sender: channel::Sender<Operation>,
}

impl Server {
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

impl Server {
    #[must_use]
    pub fn client(&self) -> Client {
        Client::new(self.reader.clone(), self.sender.clone())
    }
}

impl Server {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn into_inner(self) -> Result<Stream, Error> {
        self.sender
            .send(Operation::Exit)
            .map_err(|_| Error::general("Server/Into Inner/Send"))?;

        self.handle
            .join()
            .map_err(|_| Error::general("Server/Into Inner/Join"))
            .flatten()
            .map(Into::into)
    }
}
