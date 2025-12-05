#![allow(clippy::multiple_crate_versions)]

use std::thread::{
    self,
    JoinHandle,
};

use crossbeam::channel;
use derive_more::Debug;
use eventric_stream::{
    error::Error,
    event::{
        CandidateEvent,
        Position,
    },
    stream::{
        Reader,
        Stream,
        Writer,
        append::Append,
    },
};
use fancy_constructor::new;

// =================================================================================================
// Eventric Stream Server
// =================================================================================================

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
}

#[derive(new, Debug)]
#[new(const_fn)]
struct Processor {
    receiver: channel::Receiver<Operation>,
    writer: Writer,
}

impl Processor {
    fn process(mut self) -> Writer {
        loop {
            match self.receiver.recv() {
                Ok(Operation::Append(events, after, sender)) => {
                    if let Err(_err) = sender.send(self.writer.append(events, after)) {
                        break;
                    }
                }
                Ok(Operation::AppendSelect) => {}
                Ok(Operation::Exit) => break,
                Err(_) => break,
            }
        }

        self.writer
    }
}

#[derive(Debug)]
enum Operation {
    Append(
        #[debug("Box<dyn Iterator<Item = CandidateEvent> + Send>")]
        Box<dyn Iterator<Item = CandidateEvent> + Send>,
        Option<Position>,
        oneshot::Sender<Result<Position, Error>>,
    ),
    AppendSelect,
    Exit,
}

#[derive(Debug)]
enum State {
    Active(JoinHandle<Writer>, channel::Sender<Operation>),
    Inactive(Writer),
}
