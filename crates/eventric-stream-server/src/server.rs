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

#[derive(new, Debug)]
#[new(const_fn, name(new_inner), vis())]
pub struct Server {
    reader: Reader,
    writer: State,
}

impl Server {
    #[must_use]
    pub fn new(stream: Stream) -> Self {
        let stream = stream.split();
        let reader = stream.0;
        let writer = State::Inactive(stream.1);

        Self::new_inner(reader, writer)
    }
}

impl Server {
    /// Returns the client of this [`Server`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn client(&self) -> Result<Client, Error> {
        match &self.writer {
            State::Active(_, writer) => Ok(Client::new(self.reader.clone(), writer.clone())),
            State::Inactive(_) => Err(Error::general("Server/Client/Inactive")),
        }
    }
}

impl Server {
    pub fn start(&mut self) {
        replace_with::replace_with_or_abort(&mut self.writer, |writer| match writer {
            State::Inactive(writer) => {
                let channel = channel::bounded::<Operation>(128);
                let handle = thread::spawn(move || Processor::new(channel.1, writer).process());

                State::Active(handle, channel.0)
            }
            writer @ State::Active(..) => writer,
        });
    }

    /// Returns the stop of this [`Server`].
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn stop(&mut self) {
        replace_with::replace_with_or_abort(&mut self.writer, |writer| match writer {
            State::Active(handle, sender) => {
                sender
                    .send(Operation::Exit)
                    .map_err(|_| Error::general("Server/Stop/Send"))
                    .expect("Server Send");

                let writer = handle
                    .join()
                    .map_err(|_| Error::general("Server/Stop/Join"))
                    .flatten()
                    .expect("Writer Join");

                State::Inactive(writer)
            }
            writer @ State::Inactive(..) => writer,
        });
    }
}

#[derive(Debug)]
enum State {
    Active(
        JoinHandle<Result<Writer, Error>>,
        channel::Sender<Operation>,
    ),
    Inactive(Writer),
}
