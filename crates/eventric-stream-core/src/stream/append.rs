//! See the `eventric-stream` crate for full documentation, including
//! module-level documentation.

use fjall::Database;

use crate::{
    error::Error,
    event::{
        CandidateEvent,
        position::Position,
        timestamp::Timestamp,
    },
    stream::{
        data::Data,
        select::{
            Prepared,
            Selection,
            SelectionHash,
            Selections,
        },
    },
};

// =================================================================================================
// Append
// =================================================================================================

// Append

/// The [`Append`] trait defines the logical operation of appending (candidate)
/// events to a stream or stream-like type, with an optional condition to
/// determine behaviour related to concurrency, etc.
pub trait Append {
    /// Appends new [`CandidateEvent`]s to the relevant stream or stream-like
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
    //#[rustfmt::skip]
    fn append<E>(&mut self, events: E, after: Option<Position>) -> Result<Position, Error>
    where
        E: IntoIterator<Item = CandidateEvent>;

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn append_select<E, S>(
        &mut self,
        events: E,
        selection: S,
        after: Option<Position>,
    ) -> Result<(Position, Prepared<Selection>), Error>
    where
        E: IntoIterator<Item = CandidateEvent>,
        S: Into<Prepared<Selection>>;

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn append_select_multi<E, S>(
        &mut self,
        events: E,
        selections: S,
        after: Option<Position>,
    ) -> Result<(Position, Prepared<Selections>), Error>
    where
        E: IntoIterator<Item = CandidateEvent>,
        S: Into<Prepared<Selections>>;
}

// Implementations

pub(crate) fn append<E>(
    database: &Database,
    data: &Data,
    next: &mut Position,
    events: E,
    after: Option<Position>,
) -> Result<Position, Error>
where
    E: IntoIterator<Item = CandidateEvent>,
{
    check(data, *next, None, after).and_then(|()| put(database, data, next, events))
}

pub(crate) fn append_select<E, S>(
    database: &Database,
    data: &Data,
    next: &mut Position,
    events: E,
    selection: S,
    after: Option<Position>,
) -> Result<(Position, Prepared<Selection>), Error>
where
    E: IntoIterator<Item = CandidateEvent>,
    S: Into<Prepared<Selection>>,
{
    let prepared = selection.into();

    check(data, *next, Some(prepared.as_ref()), after)
        .and_then(|()| put(database, data, next, events))
        .map(|position| (position, prepared))
}

pub(crate) fn append_select_multi<E, S>(
    database: &Database,
    data: &Data,
    next: &mut Position,
    events: E,
    selection: S,
    after: Option<Position>,
) -> Result<(Position, Prepared<Selections>), Error>
where
    E: IntoIterator<Item = CandidateEvent>,
    S: Into<Prepared<Selections>>,
{
    let prepared = selection.into();

    check(data, *next, Some(prepared.as_ref()), after)
        .and_then(|()| put(database, data, next, events))
        .map(|position| (position, prepared))
}

fn check(
    data: &Data,
    next: Position,
    selection: Option<&SelectionHash>,
    after: Option<Position>,
) -> Result<(), Error> {
    // Shortcut the append concurrency check if the "after" position is at least the
    // current stream position. If it is, no events have been written after
    // the given position, so the condition will never match.

    let from = after.map(|after| after + 1);

    if let Some(from) = from
        && from >= next
    {
        return Ok(());
    }

    // Determine the query and from position. Note that queries internally are
    // always from, rather than after, a particular position, so we increment the
    // position here (if it exists) to ensure a correct from position.

    if let Some(selection) = selection {
        // We don't need to actually examine the events at all, the underlying
        // implementation only needs to check if there is any matching event in the
        // resultant query stream - contains avoids mapping positions to events, etc.

        if data.indices.contains(selection, from) {
            return Err(Error::Concurrency);
        }
    } else if let Some(from) = from
        && from < next
    {
        return Err(Error::Concurrency);
    }

    Ok(())
}

fn put<E>(
    database: &Database,
    data: &Data,
    next: &mut Position,
    events: E,
) -> Result<Position, Error>
where
    E: IntoIterator<Item = CandidateEvent>,
{
    // Create a local copy of the "next" position here, so that it can be
    // incremented independently of the stream instance. As we only set the stream
    // next position to the incremented position after the batch has committed
    // successfully, this ensures that we don't create a gap in the sequence should
    // the batch commit fail.

    let mut local_next = *next;
    let mut batch = database.batch();

    for event in events {
        let event = event.into();
        let timestamp = Timestamp::now()?;

        data.events.put(&mut batch, local_next, &event, timestamp);
        data.indices.put(&mut batch, local_next, &event, timestamp);
        data.references.put(&mut batch, &event);

        local_next += 1;
    }

    // Commit the batch...

    batch.commit()?;

    // ...and only update the stream next position if successful.

    *next = local_next;

    // TODO: Deal with edge case of appending zero events to an empty stream!

    Ok(*next - 1)
}
