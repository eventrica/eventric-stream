use crossbeam::channel;
use derive_more::{
    Debug,
    From,
};
use eventric_stream_core::{
    error::Error,
    event::{
        CandidateEvent,
        Position,
    },
    stream::{
        Writer,
        append::{
            Append as _,
            AppendSelect as _,
        },
        select::{
            Prepared,
            PreparedMultiple,
        },
    },
};
use fancy_constructor::new;

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
    #[rustfmt::skip]
    pub fn process(mut self) -> Result<Writer, Error> {
        loop {
            match self.receiver.recv() {
                Ok(Operation::Append(append)) => self.append(append)?,
                Ok(Operation::AppendSelect(append)) => self.append_select(append)?,
                Ok(Operation::AppendSelectMultiple(append)) => self.append_select_multiple(append)?,
                Ok(Operation::Exit) => return Ok(self.writer),
                Err(_) => return Err(Error::general("processor/process/receive")),
            }
        }
    }
}

impl Processor {
    fn writer<F, R>(
        &mut self,
        operation: F,
        sender: oneshot::Sender<Result<R, Error>>,
    ) -> Result<(), Error>
    where
        F: FnOnce(&mut Writer) -> Result<R, Error>,
    {
        sender
            .send(operation(&mut self.writer))
            .map_err(|_| Error::general("processor/writer/send"))
    }
}

impl Processor {
    #[rustfmt::skip]
    fn append(&mut self, append: AppendOperation) -> Result<(), Error> {
        self.writer(
            |writer| writer.append(
                append.events,
                append.after
            ),
            append.sender,
        )
    }

    #[rustfmt::skip]
    fn append_select(&mut self, append: AppendSelectOperation) -> Result<(), Error> {
        self.writer(
            |writer| writer.append_select(
                append.events,
                append.selection,
                append.after
            ),
            append.sender,
        )
    }

    fn append_select_multiple(
        &mut self,
        append: AppendSelectMultipleOperation,
    ) -> Result<(), Error> {
        self.writer(
            |writer| writer.append_select_multiple(append.events, append.selections, append.after),
            append.sender,
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Operation

#[derive(Debug, From)]
pub enum Operation {
    Append(AppendOperation),
    AppendSelect(AppendSelectOperation),
    AppendSelectMultiple(AppendSelectMultipleOperation),
    Exit,
}

#[derive(new, Debug)]
#[new(const_fn)]
pub struct AppendOperation {
    #[debug("Box<dyn Iterator<Item = CandidateEvent> + Send>")]
    events: Box<dyn Iterator<Item = CandidateEvent> + Send>,
    after: Option<Position>,
    sender: oneshot::Sender<Result<Position, Error>>,
}

#[derive(new, Debug)]
#[new(const_fn)]
pub struct AppendSelectOperation {
    #[debug("Box<dyn Iterator<Item = CandidateEvent> + Send>")]
    events: Box<dyn Iterator<Item = CandidateEvent> + Send>,
    selection: Prepared,
    after: Option<Position>,
    sender: oneshot::Sender<Result<(Position, Prepared), Error>>,
}

#[derive(new, Debug)]
#[new(const_fn)]
pub struct AppendSelectMultipleOperation {
    #[debug("Box<dyn Iterator<Item = CandidateEvent> + Send>")]
    events: Box<dyn Iterator<Item = CandidateEvent> + Send>,
    selections: PreparedMultiple,
    after: Option<Position>,
    sender: oneshot::Sender<Result<(Position, PreparedMultiple), Error>>,
}
