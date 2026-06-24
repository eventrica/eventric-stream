//! The top-level [`Stream`], its [`Reader`]/[`Writer`] split, and the shared
//! value types ([`Position`], [`Timestamp`], [`Metadata`]). The error model
//! ([`Error`], [`Conflict`](crate::error::Conflict), [`Result`]) lives in
//! [`crate::error`].

pub mod concurrent;
pub mod operate;
mod store;

use std::{
    path::Path,
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

use derive_more::{
    Debug,
    with_trait::{
        Add,
        AddAssign,
        Sub,
        SubAssign,
    },
};
use error_stack::ResultExt;
use fancy_constructor::new;
use fjall::Database;

use crate::{
    error::{
        Error,
        Result,
    },
    event::Event,
    stream::{
        operate::{
            Condition,
            append::Append,
            select::{
                Select,
                SelectIter,
            },
        },
        store::Store,
    },
};

// =================================================================================================
// Stream
// =================================================================================================

// Builder

/// Configures and opens a [`Stream`] at a given path. Obtained from
/// [`Stream::builder`].
#[derive(new, Debug)]
#[new(vis())]
pub struct Builder<P>
where
    P: AsRef<Path>,
{
    path: P,
    #[new(default)]
    temporary: Option<bool>,
}

impl<P> Builder<P>
where
    P: AsRef<Path>,
{
    /// Open the stream, recovering the `next` position cursor from the existing
    /// `events` keyspace.
    pub fn open(self) -> Result<Stream> {
        let database = Database::builder(self.path)
            .temporary(self.temporary.unwrap_or_default())
            .open()
            .change_context(Error)
            .attach("failed to open database")?;

        let storage = Store::open(&database)?;
        let next = storage.len().map(Position::new)?;

        Ok(Stream::new(database, next, storage))
    }
}

impl<P> Builder<P>
where
    P: AsRef<Path>,
{
    /// Whether the stream is temporary (its on-disk data is cleaned up on
    /// drop). Defaults to `false`.
    #[must_use]
    pub fn temporary(mut self, temporary: bool) -> Self {
        self.temporary = Some(temporary);
        self
    }
}

// -------------------------------------------------------------------------------------------------

// Metadata

/// The metadata a persisted event carries (its `M` in `Event<M, T>`): the
/// [`Position`] and [`Timestamp`] assigned when it was appended.
#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct Metadata(
    #[new(name(position))] pub(crate) Position,
    #[new(name(timestamp))] pub(crate) Timestamp,
);

impl Metadata {
    /// The position the event was appended at.
    #[must_use]
    pub fn position(&self) -> Position {
        self.0
    }

    /// The timestamp the event was appended at.
    #[must_use]
    pub fn timestamp(&self) -> Timestamp {
        self.1
    }
}

// -------------------------------------------------------------------------------------------------

// Stream

/// An append/query event stream consistent with Dynamic Consistency Boundaries
/// (DCB). Owns the database, the store, and the `next` position cursor;
/// [`split`](Stream::split) into a [`Reader`] and [`Writer`] for concurrent
/// access.
#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct Stream {
    #[debug("Database")]
    database: Database,
    next: Position,
    store: Store,
}

impl Stream {
    /// Begin opening a stream at `path`. Configure with
    /// [`temporary`](Builder::temporary) and finish with
    /// [`open`](Builder::open).
    pub fn builder<P>(path: P) -> Builder<P>
    where
        P: AsRef<Path>,
    {
        Builder::new(path)
    }

    /// Whether the stream holds no events.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The number of events appended to the stream (also the `next` position).
    #[must_use]
    pub fn len(&self) -> u64 {
        self.next.0
    }

    /// Split into a cloneable, read-only [`Reader`] and the unique [`Writer`].
    /// Reads scale across `Reader` clones; writes serialize through the single
    /// `Writer`. Recombine into a `Stream` with `Stream::from`.
    #[must_use]
    pub fn split(self) -> (Reader, Writer) {
        let reader = Reader::new(self.store.clone());
        let writer = Writer::new(self.database, self.next, self.store);

        (reader, writer)
    }
}

impl Append for Stream {
    fn append<E>(&mut self, events: E, condition: Condition) -> Result<Position, Error>
    where
        E: IntoIterator<Item = Event<(), String>>,
        E::IntoIter: Send + 'static,
    {
        operate::Appender::new(&mut || self.database.batch(), &mut self.next, &self.store)
            .append(events, condition)
    }
}

impl Select for Stream {
    fn select(&self, condition: Condition) -> SelectIter {
        self.store.select(condition)
    }
}

// -------------------------------------------------------------------------------------------------

// Reader

/// A cloneable, read-only handle to a stream, obtained from [`Stream::split`].
/// Reads run concurrently across clones; a `Reader` cannot append.
#[derive(new, Clone, Debug)]
#[new(const_fn, vis())]
pub struct Reader {
    store: Store,
}

impl Select for Reader {
    fn select(&self, condition: Condition) -> SelectIter {
        self.store.select(condition)
    }
}

// -------------------------------------------------------------------------------------------------

// Writer

/// The unique write handle to a stream, obtained from [`Stream::split`]. It
/// owns the write side (database + position cursor); fold it back into a
/// [`Stream`] with `Stream::from`.
#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct Writer {
    #[debug("Database")]
    database: Database,
    next: Position,
    store: Store,
}

impl Append for Writer {
    fn append<E>(&mut self, events: E, condition: Condition) -> Result<Position, Error>
    where
        E: IntoIterator<Item = Event<(), String>>,
        E::IntoIter: Send + 'static,
    {
        operate::Appender::new(&mut || self.database.batch(), &mut self.next, &self.store)
            .append(events, condition)
    }
}

impl From<Writer> for Stream {
    fn from(writer: Writer) -> Self {
        Self::new(writer.database, writer.next, writer.store)
    }
}

// -------------------------------------------------------------------------------------------------

// Position

/// A `u64` ordinal identifying an event's place in the stream. Also the unit of
/// the position-based (DCB) append concurrency check.
#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(const_fn)]
pub struct Position(#[new(name(position))] pub(crate) u64);

impl Position {
    /// The largest possible position.
    pub const MAX: Self = Self::new(u64::MAX);
    /// The smallest possible position (the head of an empty stream).
    pub const MIN: Self = Self::new(u64::MIN);
}

impl Add<u64> for Position {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<u64> for Position {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::MIN
    }
}

impl Sub<u64> for Position {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<u64> for Position {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 -= rhs;
    }
}

// -------------------------------------------------------------------------------------------------

// Timestamp

/// The wall-clock time an event was appended, in nanoseconds since the Unix
/// epoch. **Not monotonic** — its order may not match position order.
#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(#[new(name(nanos))] pub(crate) u64);

impl Timestamp {
    /// The current time as a `Timestamp`.
    pub fn now() -> Result<Self> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .change_context(Error)
            .attach("failed to get epoch duration")?;

        let nanos = u64::try_from(duration.as_nanos())
            .change_context(Error)
            .attach("failed to get epoch duration as nanos")?;

        Ok(Self::new(nanos))
    }
}

// =================================================================================================
// Tests
// =================================================================================================

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        Position,
        Reader,
        Stream,
        Writer,
        operate::{
            Condition,
            Selection,
            append::Append,
            select::{
                Select,
                Selector,
                TypeSelector,
            },
        },
    };
    use crate::{
        error::Conflict,
        event::{
            Data,
            Event,
            Facets,
            Name,
            Tag,
            Type,
            Version,
        },
        utils::temp_path,
    };

    fn stream() -> Stream {
        Stream::builder(temp_path()).temporary(true).open().unwrap()
    }

    fn event(identifier: &str, version: u8, tags: &[&str]) -> Event<(), String> {
        let ty = Type::new(Name::new(identifier).unwrap(), Version::new(version));
        let tags = tags
            .iter()
            .map(|tag| Tag::new(*tag).unwrap())
            .collect::<BTreeSet<_>>();

        Event::new(
            Data::new(b"payload".to_vec()).unwrap(),
            Facets::new(ty, tags),
            (),
        )
    }

    // The masked, multi-selection query surface end to end via the
    // public Stream API. Each selection is one mask bit, in order.
    #[test]
    fn select_masks_events_by_selection() {
        let mut stream = stream();

        stream
            .append(
                vec![
                    event("Enrolled", 0, &["student:1", "course:1"]),
                    event("Enrolled", 0, &["student:2", "course:1"]),
                    event("Dropped", 0, &["student:1", "course:1"]),
                ],
                Condition::new(),
            )
            .unwrap();

        // selection 0: any "Enrolled"; selection 1: "Dropped" carrying student:1
        let condition = Condition::new().selections([
            Selection::new([Selector::types([TypeSelector::new("Enrolled").unwrap()])]),
            Selection::new([Selector::types_and_tags(
                [TypeSelector::new("Dropped").unwrap()],
                [Tag::new("student:1").unwrap()],
            )]),
        ]);

        let results = stream
            .select(condition)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].mask.as_ref(), [true, false].as_slice()); // Enrolled
        assert_eq!(results[1].mask.as_ref(), [true, false].as_slice()); // Enrolled
        assert_eq!(results[2].mask.as_ref(), [false, true].as_slice()); // Dropped+student:1
    }

    #[test]
    fn select_with_no_selections_scans_all_with_empty_mask() {
        let mut stream = stream();

        stream
            .append(
                vec![event("A", 0, &[]), event("B", 0, &[])],
                Condition::new(),
            )
            .unwrap();

        let results = stream
            .select(Condition::new())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|result| result.mask.as_ref().is_empty()));
        assert_eq!(results[0].event.2.0, Position::new(0));
        assert_eq!(results[1].event.2.0, Position::new(1));
    }

    #[test]
    fn select_filters_by_version_range() {
        let mut stream = stream();

        stream
            .append(
                vec![event("T", 0, &[]), event("T", 1, &[]), event("T", 2, &[])],
                Condition::new(),
            )
            .unwrap();

        // versions [1, 3) — matches v1 and v2 only.
        let condition = Condition::new().selections([Selection::new([Selector::types([
            TypeSelector::with_versions("T", Version::new(1)..Version::new(3)).unwrap(),
        ])])]);

        let results = stream
            .select(condition)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results.len(), 2);
        for result in &results {
            assert_eq!(result.mask.as_ref(), [true].as_slice());
        }
    }

    // An event matching several selections is emitted once with multiple mask
    // bits set; bits stay independent across events.
    #[test]
    fn select_overlapping_selections_set_multiple_mask_bits() {
        let mut stream = stream();

        stream
            .append(
                vec![
                    event("Enrolled", 0, &["student:1"]),
                    event("Enrolled", 0, &[]),
                ],
                Condition::new(),
            )
            .unwrap();

        let condition = Condition::new().selections([
            Selection::new([Selector::types([TypeSelector::new("Enrolled").unwrap()])]),
            Selection::new([Selector::types_and_tags(
                [TypeSelector::new("Enrolled").unwrap()],
                [Tag::new("student:1").unwrap()],
            )]),
        ]);

        let results = stream
            .select(condition)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].mask.as_ref(), [true, true].as_slice()); // both selections
        assert_eq!(results[1].mask.as_ref(), [true, false].as_slice()); // only selection 0
    }

    // next_back yields events in descending position order, masks still paired
    // to the right event.
    #[test]
    fn select_reverse_iteration_pairs_masks() {
        let mut stream = stream();

        stream
            .append(
                vec![
                    event("Enrolled", 0, &["student:1", "course:1"]),
                    event("Enrolled", 0, &["student:2", "course:1"]),
                    event("Dropped", 0, &["student:1", "course:1"]),
                ],
                Condition::new(),
            )
            .unwrap();

        let condition = Condition::new().selections([
            Selection::new([Selector::types([TypeSelector::new("Enrolled").unwrap()])]),
            Selection::new([Selector::types_and_tags(
                [TypeSelector::new("Dropped").unwrap()],
                [Tag::new("student:1").unwrap()],
            )]),
        ]);

        let results = stream
            .select(condition)
            .rev()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].event.2.0, Position::new(2));
        assert_eq!(results[0].mask.as_ref(), [false, true].as_slice());
        assert_eq!(results[1].event.2.0, Position::new(1));
        assert_eq!(results[1].mask.as_ref(), [true, false].as_slice());
        assert_eq!(results[2].event.2.0, Position::new(0));
        assert_eq!(results[2].mask.as_ref(), [true, false].as_slice());
    }

    // The `from` lower bound is inclusive and applies to the indexed path.
    #[test]
    fn select_from_position_lower_bound() {
        let mut stream = stream();

        stream
            .append(
                vec![
                    event("Enrolled", 0, &[]),
                    event("Enrolled", 0, &[]),
                    event("Dropped", 0, &[]),
                ],
                Condition::new(),
            )
            .unwrap();

        let condition = Condition::new()
            .from(Position::new(1))
            .selections([Selection::new([Selector::types([TypeSelector::new(
                "Enrolled",
            )
            .unwrap()])])]);

        let results = stream
            .select(condition)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].event.2.0, Position::new(1));
    }

    // Selectors within one selection are OR-combined and contribute one mask bit.
    #[test]
    fn select_multiple_selectors_in_one_selection_or() {
        let mut stream = stream();

        stream
            .append(
                vec![event("A", 0, &[]), event("B", 0, &[]), event("C", 0, &[])],
                Condition::new(),
            )
            .unwrap();

        let condition = Condition::new().selections([Selection::new([
            Selector::types([TypeSelector::new("A").unwrap()]),
            Selector::types([TypeSelector::new("B").unwrap()]),
        ])]);

        let results = stream
            .select(condition)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results.len(), 2); // A and B, not C
        assert_eq!(results[0].mask.as_ref(), [true].as_slice());
        assert_eq!(results[1].mask.as_ref(), [true].as_slice());
    }

    // Conditional (DCB) append. A condition rejects the append iff a
    // matching event already exists at or after the condition's position.

    #[test]
    fn append_with_empty_condition_is_unconditional() {
        let mut stream = stream();

        stream
            .append(vec![event("A", 0, &[])], Condition::new())
            .unwrap();
        // No selections => no concurrency check, even though events exist.
        assert!(
            stream
                .append(vec![event("B", 0, &[])], Condition::new())
                .is_ok()
        );
    }

    #[test]
    fn append_is_rejected_when_a_matching_event_exists() {
        let mut stream = stream();

        stream
            .append(vec![event("Enrolled", 0, &[])], Condition::new())
            .unwrap();

        // Append only if no "Enrolled" event exists from position 0 — one does.
        let condition = Condition::new()
            .from(Position::new(0))
            .selections([Selection::new([Selector::types([TypeSelector::new(
                "Enrolled",
            )
            .unwrap()])])]);
        let result = stream.append(vec![event("Dropped", 0, &[])], condition);

        let report = result.unwrap_err();
        assert!(report.downcast_ref::<Conflict>().is_some());
        // The rejected append did not write anything.
        assert_eq!(stream.len(), 1);
    }

    #[test]
    fn append_is_allowed_when_no_matching_event_exists() {
        let mut stream = stream();

        stream
            .append(vec![event("Enrolled", 0, &[])], Condition::new())
            .unwrap();

        // Watch for "Dropped" from position 0 — none exists, so the append
        // proceeds (this exercises the real index check, not the head shortcut).
        let condition = Condition::new()
            .from(Position::new(0))
            .selections([Selection::new([Selector::types([TypeSelector::new(
                "Dropped",
            )
            .unwrap()])])]);

        assert!(
            stream
                .append(vec![event("Dropped", 0, &[])], condition)
                .is_ok()
        );
        assert_eq!(stream.len(), 2);
    }

    #[test]
    fn append_condition_window_starts_at_position() {
        let mut stream = stream();

        stream
            .append(
                vec![event("Enrolled", 0, &[]), event("Dropped", 0, &[])],
                Condition::new(),
            )
            .unwrap();

        // The conflicting "Enrolled" is at position 0; a window starting at
        // position 1 does not see it, so the append is allowed.
        let condition = Condition::new()
            .from(Position::new(1))
            .selections([Selection::new([Selector::types([TypeSelector::new(
                "Enrolled",
            )
            .unwrap()])])]);

        assert!(
            stream
                .append(vec![event("Enrolled", 0, &[])], condition)
                .is_ok()
        );
    }

    // A window starting at or after the head short-circuits the index scan: a
    // caller anchored at the head gets no spurious conflict, even though a
    // matching event exists below the window.
    #[test]
    fn append_condition_window_at_head_never_conflicts() {
        let mut stream = stream();

        stream
            .append(vec![event("Enrolled", 0, &[])], Condition::new())
            .unwrap();

        let condition = Condition::new()
            .from(Position::new(1)) // == next; the Enrolled at position 0 is below it
            .selections([Selection::new([Selector::types([
                TypeSelector::new("Enrolled").unwrap(),
            ])])]);

        assert!(
            stream
                .append(vec![event("Dropped", 0, &[])], condition)
                .is_ok()
        );
    }

    // With no position the condition checks the whole stream.
    #[test]
    fn append_with_no_position_checks_whole_stream() {
        let mut stream = stream();

        stream
            .append(vec![event("Enrolled", 0, &[])], Condition::new())
            .unwrap();

        // Watching "Enrolled" with no `from` — one exists anywhere => conflict.
        let conflicting =
            Condition::new().selections([Selection::new([Selector::types([TypeSelector::new(
                "Enrolled",
            )
            .unwrap()])])]);
        assert!(
            stream
                .append(vec![event("Dropped", 0, &[])], conflicting)
                .is_err()
        );
        assert_eq!(stream.len(), 1);

        // Watching "Dropped" with no `from` — none exists => allowed.
        let clear =
            Condition::new().selections([Selection::new([Selector::types([TypeSelector::new(
                "Dropped",
            )
            .unwrap()])])]);
        assert!(stream.append(vec![event("Dropped", 0, &[])], clear).is_ok());
        assert_eq!(stream.len(), 2);
    }

    // A tag-scoped selector conflicts only when both the type and the tag match.
    #[test]
    fn append_conflict_via_tag_scoped_selector() {
        let mut stream = stream();

        stream
            .append(vec![event("Enrolled", 0, &["student:1"])], Condition::new())
            .unwrap();

        let condition = |tag: &str| {
            Condition::new()
                .from(Position::new(0))
                .selections([Selection::new([Selector::types_and_tags(
                    [TypeSelector::new("Enrolled").unwrap()],
                    [Tag::new(tag).unwrap()],
                )])])
        };

        // Same type + same tag => conflict.
        assert!(
            stream
                .append(vec![event("X", 0, &[])], condition("student:1"))
                .is_err()
        );
        // Same type, different tag => no matching event, allowed.
        assert!(
            stream
                .append(vec![event("X", 0, &[])], condition("student:2"))
                .is_ok()
        );
    }

    // A multi-selection condition honors every selection, not just the first.
    #[test]
    fn append_multi_selection_condition_honors_all_selections() {
        let mut stream = stream();

        stream
            .append(vec![event("B", 0, &[])], Condition::new())
            .unwrap();

        // Two selections: type "A" (no match) and type "B" (matches the event).
        // A regression that only checked the first selection would miss this.
        let condition = Condition::new().from(Position::new(0)).selections([
            Selection::new([Selector::types([TypeSelector::new("A").unwrap()])]),
            Selection::new([Selector::types([TypeSelector::new("B").unwrap()])]),
        ]);

        assert!(stream.append(vec![event("C", 0, &[])], condition).is_err());
    }

    // Split into a cloneable Reader (reads) and the unique Writer
    // (writes). The Reader (and clones of it) sees the Writer's committed
    // appends, and the Writer folds back into a Stream.
    #[test]
    fn split_reader_reads_writer_writes_then_recombines() {
        let (reader, mut writer) = stream().split();

        writer
            .append(vec![event("Enrolled", 0, &["student:1"])], Condition::new())
            .unwrap();

        let condition = || {
            Condition::new().selections([Selection::new([Selector::types([TypeSelector::new(
                "Enrolled",
            )
            .unwrap()])])])
        };

        for handle in [reader.clone(), reader] {
            let results = handle
                .select(condition())
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            assert_eq!(results.len(), 1);
            assert_eq!(results[0].mask.as_ref(), [true].as_slice());
        }

        let stream = Stream::from(writer);
        assert_eq!(stream.len(), 1);
    }

    // The concurrency contract the multi-thread wrapper relies on: a `Reader` is
    // shareable across threads and cloneable, and a `Writer` can be moved to the
    // dedicated writer thread. Compile-time assertion only.
    #[test]
    fn handles_satisfy_thread_bounds() {
        const fn assert_send_sync_clone<T: Send + Sync + Clone>() {}
        const fn assert_send<T: Send>() {}

        assert_send_sync_clone::<Reader>();
        assert_send::<Writer>();
    }

    // Data written before a (non-temporary) stream is dropped is
    // recovered on re-open, including the `next` position cursor.
    #[test]
    fn data_persists_across_reopen() {
        let path = temp_path();

        {
            let mut stream = Stream::builder(&path).open().unwrap();
            stream
                .append(
                    vec![
                        event("Enrolled", 0, &["student:1"]),
                        event("Dropped", 0, &[]),
                    ],
                    Condition::new(),
                )
                .unwrap();
            assert_eq!(stream.len(), 2);
        }

        // Re-open the same path; `temporary(true)` cleans it up on drop.
        let stream = Stream::builder(&path).temporary(true).open().unwrap();

        assert_eq!(stream.len(), 2); // cursor recovered from the events keyspace
        let results = stream
            .select(Condition::new())
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(results.len(), 2);
    }

    // A full scan (no selections) honors the `from` lower bound.
    #[test]
    fn full_scan_respects_from_position() {
        let mut stream = stream();

        stream
            .append(
                vec![event("A", 0, &[]), event("B", 0, &[]), event("C", 0, &[])],
                Condition::new(),
            )
            .unwrap();

        let results = stream
            .select(Condition::new().from(Position::new(1)))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results.len(), 2); // positions 1 and 2
        assert_eq!(results[0].event.2.0, Position::new(1));
        assert_eq!(results[1].event.2.0, Position::new(2));
    }

    // Appending zero events is a usage error, not a panic/underflow.
    #[test]
    fn append_with_no_events_is_an_error() {
        let mut stream = stream();

        assert!(
            stream
                .append(Vec::<Event<(), String>>::new(), Condition::new())
                .is_err()
        );
        assert_eq!(stream.len(), 0);
    }

    // A queried event is fully readable through the public accessors a
    // consumer (e.g. the model layer) needs — payload, metadata, and type.
    #[test]
    fn queried_event_exposes_public_accessors() {
        let mut stream = stream();
        stream
            .append(vec![event("Enrolled", 0, &["student:1"])], Condition::new())
            .unwrap();

        let condition =
            Condition::new().selections([Selection::new([Selector::types([TypeSelector::new(
                "Enrolled",
            )
            .unwrap()])])]);
        let results = stream
            .select(condition)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let event = &results[0].event;
        assert_eq!(event.meta().position(), Position::new(0));
        assert_eq!(event.data().as_ref(), &b"payload"[..]);
        assert_eq!(event.facets().ty().version(), Version::new(0));

        // The type-name is the same `Name<u64>` derived from the identifier
        // string via the stable hash — how the model will match event types.
        let expected: Name<u64> = Name::<String>::new("Enrolled").unwrap().into();
        assert_eq!(event.facets().ty().name(), &expected);
    }
}
