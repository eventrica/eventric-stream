//! See the `eventric-stream` crate for full documentation, including
//! module-level documentation.

pub(crate) mod condition;

use crate::{
    error::Error,
    event::{
        EphemeralEvent,
        position::Position,
        timestamp::Timestamp,
    },
    stream::Stream,
};

// =================================================================================================
// Append
// =================================================================================================

/// The [`Append`] trait defines the logical operation of appending (ephemeral)
/// events to a stream or stream-like type, with an optional condition to
/// determine behaviour related to concurrency, etc.
pub trait Append {
    /// Appends new [`EphemeralEvent`]s to the relevant stream or stream-like
    /// instance, optionally performing a concurrency check based on a supplied
    /// [`Condition`].
    ///
    /// If successful, returns the position of the last event appended, i.e. the
    /// effective head of the stream. This position can be used in concurrency
    /// checks as an "after" position.
    ///
    /// TODO: [Full append documentation + examples][issue]
    ///
    /// # Errors
    ///
    /// Returns an error if the optional concurrency checks fails, or in the
    /// case of underlying database/IO errors.
    ///
    /// [issue]: https://github.com/eventrica/eventric-stream/issues/23
    #[rustfmt::skip]
    fn append<'a, E>(&mut self, events: E, condition: Option<&Condition>) -> Result<Position, Error>
    where
        E: IntoIterator<Item = &'a EphemeralEvent>;
}

impl Append for Stream {
    #[rustfmt::skip]
    fn append<'a, E>(&mut self, events: E, condition: Option<&Condition>) -> Result<Position, Error>
    where
        E: IntoIterator<Item = &'a EphemeralEvent>,
    {
        // Only apply the concurrent check if a condition has been provided, otherwise
        // the append should be unconditional.

        if let Some(condition) = condition {
            self.check(condition)?;
        }

        // Append the events, as the concurrency check did not return an error.

        self.put(events)
    }
}

// -------------------------------------------------------------------------------------------------

// Stream Extension

impl Stream {
    #[rustfmt::skip]
    fn check(&self, condition: &Condition) -> Result<(), Error> {

        // Shortcut the append concurrency check if the "after" position is at least the
        // current stream position. If it is, no events have been written after
        // the given position, so the condition will never match.

        if let Some(after) = condition.after && after >= self.next {
            return Ok(());
        }

        // Determine the query and from position. Note that queries internally are
        // always from, rather than after, a particular position, so we increment the
        // position here (if it exists) to ensure a correct from position.

        let query = &condition.fail_if_matches;
        let from = condition.after.map(|after| after + 1);

        // We don't need to actually examine the events at all, the underlying
        // implementation only needs to check if there is any matching event in the
        // resultant query stream - contains avoids mapping positions to events, etc.

        if self.data.indices.contains(query, from) {
            return Err(Error::Concurrency);
        }

        Ok(())
    }

    fn put<'a, E>(&mut self, events: E) -> Result<Position, Error>
    where
        E: IntoIterator<Item = &'a EphemeralEvent>,
    {
        // Create a local copy of the "next" position here, so that it can be
        // incremented independently of the stream instance. As we only set the stream
        // next position to the incremented position after the batch has committed
        // successfully, this ensures that we don't create a gap in the sequence should
        // the batch commit fail.

        let mut next = self.next;
        let mut batch = self.database.batch();

        for event in events {
            let event = event.into();
            let timestamp = Timestamp::now()?;

            self.data.events.put(&mut batch, next, &event, timestamp);
            self.data.indices.put(&mut batch, next, &event, timestamp);
            self.data.references.put(&mut batch, &event);

            next += 1;
        }

        // Commit the batch...

        batch.commit()?;

        // ...and only update the stream next position if successful.

        self.next = next;

        // TODO: Deal with edge case of appending zero events to an empty stream!

        Ok(self.next - 1)
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::condition::Condition;
