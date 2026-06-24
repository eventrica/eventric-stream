use crossbeam::channel;
use derive_more::{
    Debug,
    From,
};
use error_stack::Report;
use fancy_constructor::new;

use crate::{
    error::Error,
    event::Event,
    stream::{
        Position,
        Writer,
        operate::{
            Condition,
            append::Append as _,
        },
    },
};

// =================================================================================================
// Processor
// =================================================================================================

#[derive(new, Debug)]
#[new(const_fn)]
pub struct Processor {
    receiver: channel::Receiver<Operation>,
    writer: Writer,
}

impl Processor {
    pub fn process(mut self) -> Result<Writer, Report<Error>> {
        loop {
            match self.receiver.recv() {
                Ok(Operation::Append(append)) => self.append(append)?,
                Ok(Operation::Exit) => return Ok(self.writer),
                Err(_) => return Err(Report::new(Error).attach("processor/process/receive")),
            }
        }
    }
}

impl Processor {
    fn writer<F, R>(
        &mut self,
        operation: F,
        sender: oneshot::Sender<Result<R, Report<Error>>>,
    ) -> Result<(), Report<Error>>
    where
        F: FnOnce(&mut Writer) -> Result<R, Report<Error>>,
    {
        sender
            .send(operation(&mut self.writer))
            .map_err(|_| Report::new(Error).attach("processor/writer/send"))
    }
}

impl Processor {
    fn append(&mut self, append: AppendOperation) -> Result<(), Report<Error>> {
        self.writer(
            |writer| writer.append(append.events, append.condition),
            append.sender,
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Operation

#[derive(Debug, From)]
pub enum Operation {
    Append(AppendOperation),
    Exit,
}

#[derive(new, Debug)]
#[new(const_fn)]
pub struct AppendOperation {
    #[debug("Box<dyn Iterator<Item = Event<(), String>> + Send>")]
    events: Box<dyn Iterator<Item = Event<(), String>> + Send>,
    condition: Condition,
    sender: oneshot::Sender<Result<Position, Report<Error>>>,
}
