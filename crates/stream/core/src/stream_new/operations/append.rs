use error_stack::ResultExt;
use fjall::OwnedWriteBatch as Batch;

use crate::{
    event_new::Event,
    stream_new::{
        Error,
        Position,
        Result,
        storage::Storage,
    },
};

// =================================================================================================
// Append
// =================================================================================================

pub trait Append {
    fn append<E>(&mut self, events: E, after: Option<Position>) -> Result<Position>
    where
        E: IntoIterator<Item = Event<(), String>>,
        E::IntoIter: Send + 'static;
}

impl<B> Append for (&mut B, &mut Position, &Storage)
where
    B: FnMut() -> Batch,
{
    fn append<E>(&mut self, events: E, after: Option<Position>) -> Result<Position>
    where
        E: IntoIterator<Item = Event<(), String>>,
        E::IntoIter: Send + 'static,
    {
        after
            .is_none_or(|after| after >= *self.1)
            .ok_or(Error)
            .attach("failed to append (concurrency: after < current)")?;

        self.2.insert(self.0, events, self.1)
    }
}
