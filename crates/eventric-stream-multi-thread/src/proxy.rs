use crossbeam::channel;
use error_stack::Report;
use eventric_stream_core::{
    error::Error,
    event::Event,
    stream::{
        Append,
        Condition,
        Position,
        Reader,
        Select,
        SelectIter,
    },
};
use fancy_constructor::new;

use crate::processor::{
    AppendOperation,
    Operation,
};

// =================================================================================================
// Proxy
// =================================================================================================

#[derive(new, Clone, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct Proxy {
    reader: Reader,
    sender: channel::Sender<Operation>,
}

impl Proxy {
    fn sender<F, O, R>(&self, operation: F) -> Result<R, Report<Error>>
    where
        F: FnOnce(oneshot::Sender<Result<R, Report<Error>>>) -> O,
        O: Into<Operation>,
    {
        let channel = oneshot::channel();

        self.sender
            .send(operation(channel.0).into())
            .map_err(|_| Report::new(Error).attach("proxy/sender/send"))?;

        // Block on the reply: the writer thread answers via the paired sender
        // after it has processed the operation. (A non-blocking `try_recv` would
        // race the worker and usually observe an empty channel.)
        channel
            .1
            .recv()
            .map_err(|_| Report::new(Error).attach("proxy/sender/receive"))
            .flatten()
    }
}

impl Append for Proxy {
    fn append<E>(&mut self, events: E, condition: Condition) -> Result<Position, Report<Error>>
    where
        E: IntoIterator<Item = Event<(), String>>,
        E::IntoIter: Send + 'static,
    {
        let events = IntoIterator::into_iter(events);
        let events = Box::new(events);

        self.sender(|sender| AppendOperation::new(events, condition, sender))
    }
}

impl Select for Proxy {
    fn select(&self, condition: Condition) -> SelectIter {
        self.reader.select(condition)
    }
}
