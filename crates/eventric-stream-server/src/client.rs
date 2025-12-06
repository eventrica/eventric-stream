use crossbeam::channel;
use eventric_stream::{
    error::Error,
    event::{
        CandidateEvent,
        Position,
    },
    stream::{
        Reader,
        append::{
            Append,
            AppendSelect,
        },
        iterate::{
            Iter,
            Iterate,
        },
        select::{
            IterSelect,
            IterSelectMultiple,
            Prepared,
            PreparedMultiple,
            Select,
        },
    },
};
use fancy_constructor::new;

use crate::processor::{
    AppendOperation,
    AppendSelectMultipleOperation,
    AppendSelectOperation,
    Operation,
};

// =================================================================================================
// Client
// =================================================================================================

#[derive(new, Clone, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct Client {
    reader: Reader,
    writer: channel::Sender<Operation>,
}

impl Client {
    #[rustfmt::skip]
    fn writer<F, O, R>(&self, operation: F) -> Result<R, Error>
    where
        F: FnOnce(oneshot::Sender<Result<R, Error>>) -> O,
        O: Into<Operation>,
    {
        let channel = oneshot::channel();

        self.writer
            .send(operation(channel.0).into())
            .map_err(|_| Error::general("Client/Writer/Send"))?;

        channel.1
            .recv()
            .map_err(|_| Error::general("Client/Writer/Receive"))
            .flatten()
    }
}

impl Append for Client {
    fn append<E>(&mut self, events: E, after: Option<Position>) -> Result<Position, Error>
    where
        E: IntoIterator<Item = CandidateEvent>,
        E::IntoIter: Send + 'static,
    {
        let events = IntoIterator::into_iter(events);
        let events = Box::new(events);

        self.writer(|sender| AppendOperation::new(events, after, sender))
    }
}

impl AppendSelect for Client {
    fn append_select<E, S>(
        &mut self,
        events: E,
        selection: S,
        after: Option<Position>,
    ) -> Result<(Position, Prepared), Error>
    where
        E: IntoIterator<Item = CandidateEvent>,
        E::IntoIter: Send + 'static,
        S: Into<Prepared>,
    {
        let events = IntoIterator::into_iter(events);
        let events = Box::new(events);
        let selection = selection.into();

        self.writer(|sender| AppendSelectOperation::new(events, selection, after, sender))
    }

    fn append_select_multiple<E, S>(
        &mut self,
        events: E,
        selections: S,
        after: Option<Position>,
    ) -> Result<(Position, PreparedMultiple), Error>
    where
        E: IntoIterator<Item = CandidateEvent>,
        E::IntoIter: Send + 'static,
        S: Into<PreparedMultiple>,
    {
        let events = IntoIterator::into_iter(events);
        let events = Box::new(events);
        let selections = selections.into();

        self.writer(|sender| AppendSelectMultipleOperation::new(events, selections, after, sender))
    }
}

impl Iterate for Client {
    fn iter(&self, from: Option<Position>) -> Iter {
        self.reader.iter(from)
    }
}

impl Select for Client {
    fn select<S>(&self, selection: S, from: Option<Position>) -> (IterSelect, Prepared)
    where
        S: Into<Prepared>,
    {
        self.reader.select(selection, from)
    }

    fn select_multiple<S>(
        &self,
        selections: S,
        from: Option<Position>,
    ) -> (IterSelectMultiple, PreparedMultiple)
    where
        S: Into<PreparedMultiple>,
    {
        self.reader.select_multiple(selections, from)
    }
}
