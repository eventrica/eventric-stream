//! The [`append`][self] module contains types and functionality related to the
//! [`Stream::append`] operation, such as the append-specific [`Condition`]
//! type.

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

impl Stream {
    /// Appends new [`EphemeralEvent`]s to the [`Stream`], optionally performing
    /// a concurrency check based on a supplied [`Condition`].
    ///
    /// TODO: [Full append documentation + examples][issue]
    ///
    /// # Errors
    ///
    /// Returns an error if the optional concurrency checks fails, or in the
    /// case of underlying database/IO errors.
    ///
    /// [issue]: https://github.com/eventrica/eventric-core/issues/23
    pub fn append<'a, E>(
        &mut self,
        events: E,
        condition: Option<&Condition<'_>>,
    ) -> Result<Position, Error>
    where
        E: IntoIterator<Item = &'a EphemeralEvent>,
    {
        // Only apply the concurrent check if a condition has been provided, otherwise
        // the append should be unconditional.

        if let Some(condition) = condition {
            self.append_check(condition)?;
        }

        // Append the events, as the concurrency check did not return an error.

        self.append_put(events)
    }

    #[rustfmt::skip]
    fn append_check(&self, condition: &Condition<'_>) -> Result<(), Error> {

        // Shortcut the append concurrency check if the "after" position is at least the
        // current stream position. If it is, no events have been written after
        // the given position, so the condition will never match.

        if let Some(after) = condition.after && after >= self.next {
            return Ok(());
        }

        // Determine the query and from position. Note that queries internally are
        // always from, rather than after, a particular position, so we increment the
        // position here (if it exists) to ensure a correct from position.

        let query = condition.fail_if_matches.into();
        let from = condition.after.map(|after| after + 1);

        // We don't need to actually examine the events at all, the underlying
        // implementation only needs to check if there is any matching event in the
        // resultant query stream - contains avoids mapping positions to events, etc.

        if self.data.indices.contains(&query, from) {
            return Err(Error::Concurrency);
        }

        Ok(())
    }

    fn append_put<'a, E>(&mut self, events: E) -> Result<Position, Error>
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

        Ok(self.next - 1)
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::condition::Condition;
