mod iterate;
mod operate;
mod store;

use std::{
    path::Path,
    result,
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

use derive_more::{
    Debug,
    Display,
    Error,
    with_trait::{
        Add,
        AddAssign,
        Sub,
        SubAssign,
    },
};
use error_stack::{
    Report,
    ResultExt,
};
use fancy_constructor::new;
use fjall::Database;

use crate::{
    event_new::Event,
    stream_new::store::Store,
};

// =================================================================================================
// Stream
// =================================================================================================

// Builder

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
    #[must_use]
    pub fn temporary(mut self, temporary: bool) -> Self {
        self.temporary = Some(temporary);
        self
    }
}

// -------------------------------------------------------------------------------------------------

// Error

#[derive(Debug, Display, Error)]
#[display("stream error")]
pub struct Error;

// -------------------------------------------------------------------------------------------------

// Facets

#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct Facets(
    #[new(name(position))] pub(crate) Position,
    #[new(name(timestamp))] pub(crate) Timestamp,
);

// -------------------------------------------------------------------------------------------------

// Stream

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

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn len(&self) -> u64 {
        self.next.0
    }
}

impl Append for Stream {
    fn append<E>(&mut self, events: E, after: Option<Position>) -> Result<Position, Error>
    where
        E: IntoIterator<Item = Event<(), String>>,
        E::IntoIter: Send + 'static,
    {
        (&mut || self.database.batch(), &mut self.next, &self.store).append(events, after)
    }
}

impl Select for Stream {
    fn select(&self, condition: Condition) -> SelectIter {
        self.store.select(condition)
    }
}

// -------------------------------------------------------------------------------------------------

// Position

#[rustfmt::skip]
#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[derive(Add, AddAssign, Sub, SubAssign)]
#[new(const_fn)]
pub struct Position(#[new(name(position))] pub(crate) u64);

impl Position {
    pub const MAX: Self = Self::new(u64::MAX);
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

// Result

pub type Result<T, E = Error> = result::Result<T, Report<E>>;

// -------------------------------------------------------------------------------------------------

// Timestamp

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(#[new(name(nanos))] pub(crate) u64);

impl Timestamp {
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

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::operate::{
    Append,
    Condition,
    EventAndMask,
    Mask,
    Select,
    SelectIter,
    Selection,
    Selector,
    TypeSelector,
    VersionSelector,
};

// =================================================================================================
// Tests
// =================================================================================================

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        Append,
        Condition,
        Position,
        Select,
        Selection,
        Selector,
        Stream,
        TypeSelector,
    };
    use crate::{
        event_new::{
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

    // Phase 2: the masked, multi-selection query surface end to end via the
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
                None,
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
            .append(vec![event("A", 0, &[]), event("B", 0, &[])], None)
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
                None,
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
                None,
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
                None,
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
                None,
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
                None,
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
}
