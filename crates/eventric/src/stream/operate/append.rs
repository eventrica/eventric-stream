//! Appending events to the stream under an optimistic-concurrency (DCB)
//! `Condition`.

use error_stack::Report;
use fancy_constructor::new;
use fjall::OwnedWriteBatch as Batch;

use crate::{
    error::{
        Conflict,
        Error,
        Result,
    },
    event::Event,
    stream::{
        Position,
        operate::Condition,
        store::Store,
    },
};

// =================================================================================================
// Append
// =================================================================================================

/// Appends candidate events to the stream, subject to a `Condition`.
pub trait Append {
    /// Appends `events`, rejecting with a `Conflict` if `condition`'s DCB
    /// concurrency check fails, and returns the `Position` of the last appended
    /// event.
    fn append<E>(&mut self, events: E, condition: Condition) -> Result<Position>
    where
        E: IntoIterator<Item = Event<(), String>>,
        E::IntoIter: Send + 'static;
}

// -------------------------------------------------------------------------------------------------

// Appender

/// The shared append worker behind [`Stream`](crate::stream::Stream) and
/// [`Writer`](crate::stream::Writer): a batch source, the `next`-position
/// cursor, and the `Store`. Both handles construct one and delegate to its
/// `append`, so the DCB check and the insert live in a single place.
#[derive(new)]
#[new(vis(pub(crate)))]
pub(crate) struct Appender<'a, B> {
    batch: &'a mut B,
    next: &'a mut Position,
    store: &'a Store,
}

impl<B> Appender<'_, B>
where
    B: FnMut() -> Batch,
{
    pub(crate) fn append<E>(&mut self, events: E, condition: Condition) -> Result<Position>
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
            Some(from) if from >= *self.next => false,
            _ => self.store.matches(&selections, position)?,
        };

        if conflict {
            return Err(Report::new(Error).attach(Conflict));
        }

        self.store.insert(self.batch, events, self.next)
    }
}
