use error_stack::Report;
use fjall::OwnedWriteBatch as Batch;

use crate::{
    event::Event,
    stream::{
        Conflict,
        Error,
        Position,
        Result,
        operate::Condition,
        store::Store,
    },
};

// =================================================================================================
// Append
// =================================================================================================

pub trait Append {
    fn append<E>(&mut self, events: E, condition: Condition) -> Result<Position>
    where
        E: IntoIterator<Item = Event<(), String>>,
        E::IntoIter: Send + 'static;
}

impl<B> Append for (&mut B, &mut Position, &Store)
where
    B: FnMut() -> Batch,
{
    fn append<E>(&mut self, events: E, condition: Condition) -> Result<Position>
    where
        E: IntoIterator<Item = Event<(), String>>,
        E::IntoIter: Send + 'static,
    {
        let Condition {
            position,
            selections,
        } = condition;

        // Optimistic-concurrency (DCB) check: reject the append if any event
        // matching `selections` already exists at or after `position`. Empty
        // selections means no condition, so the append is unconditional. The
        // window starting at or after the head can never conflict, so skip the
        // index scan in that case.
        let conflict = match position {
            Some(from) if from >= *self.1 => false,
            _ => self.2.matches(&selections, position)?,
        };

        if conflict {
            return Err(Report::new(Error).attach(Conflict));
        }

        self.2.insert(self.0, events, self.1)
    }
}
