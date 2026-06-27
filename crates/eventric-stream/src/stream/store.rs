mod events;
mod indices;

use error_stack::{
    Report,
    ResultExt as _,
};
use fancy_constructor::new;
use fjall::{
    Database,
    OwnedWriteBatch as Batch,
};

use crate::{
    error::{
        Error,
        Result,
    },
    event::Event,
    stream::{
        Metadata,
        Position,
        Timestamp,
        operate::Selection,
        store::{
            events::EventsIter,
            indices::IndicesIter,
        },
    },
};

// =================================================================================================
// Store
// =================================================================================================

// Constants

static HASH_LEN: usize = size_of::<u64>();
static ID_LEN: usize = size_of::<u8>();
static POSITION_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Store

#[derive(new, Clone, Debug)]
#[new(const_fn, vis())]
pub struct Store {
    pub(crate) events: Events,
    pub(crate) indices: Indices,
}

impl Store {
    pub fn open(database: &Database) -> Result<Self> {
        let events = Events::open(database)?;
        let indices = Indices::open(database)?;

        Ok(Self::new(events, indices))
    }
}

impl Store {
    pub fn len(&self) -> Result<u64> {
        self.events.len()
    }
}

impl Store {
    pub fn insert<B, E>(&self, batch: &mut B, events: E, next: &mut Position) -> Result<Position>
    where
        B: FnMut() -> Batch,
        E: IntoIterator<Item = Event<(), String>>,
        E::IntoIter: Send + 'static,
    {
        let mut batch = batch();
        let mut position = *next;

        for event in events {
            let event: Event<(), u64> = event.into();

            // The events keyspace prefixes the tag list with a `u8` count, so an
            // event cannot carry more than `u8::MAX` tags. Reject gracefully here
            // rather than letting the serializer panic on the cast.
            if event.facets().tags().len() > usize::from(u8::MAX) {
                return Err(Report::new(Error).attach("event exceeds the maximum of 255 tags"));
            }

            let meta = Timestamp::now()
                .map(|timestamp| Metadata::new(position, timestamp))
                .attach("failed to create timestamped metadata")?;

            self.events.insert(&mut batch, &event, &meta);
            self.indices.insert(&mut batch, &event, &meta);

            position += 1;
        }

        // Appending zero events has no "last position" to return (and would
        // underflow `*next - 1` on an empty stream), so treat it as a usage
        // error rather than committing an empty batch.
        if position == *next {
            return Err(Report::new(Error).attach("cannot append zero events"));
        }

        batch
            .commit()
            .change_context(Error)
            .attach("failed to commit append batch")?;

        *next = position;

        Ok(*next - 1)
    }
}

impl Store {
    /// The candidate positions matching `selections` at or after `from`,
    /// OR-unioned across selections (an index-only scan that resolves no
    /// event bodies). Shared by `iterate` and `matches`.
    fn positions(&self, selections: &[Selection], from: Option<Position>) -> IndicesIter {
        self.indices.iterate(
            selections
                .iter()
                .flat_map(|selection| selection.selectors.iter()),
            from,
        )
    }

    pub fn iterate(&self, selections: &[Selection], from: Option<Position>) -> StoreIter {
        if selections.is_empty() {
            StoreIter::Events(self.events.iterate(from))
        } else {
            StoreIter::Indices(self.events.clone(), self.positions(selections, from))
        }
    }
}

impl Store {
    /// Whether any event matching `selections` exists at or after `from`. Used
    /// for the append concurrency (DCB) check; resolves index positions only,
    /// so it never materializes an event. Empty `selections` is vacuously
    /// `false`.
    pub fn matches(&self, selections: &[Selection], from: Option<Position>) -> Result<bool> {
        match self.positions(selections, from).next() {
            Some(result) => result.map(|_| true),
            None => Ok(false),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Iterators

#[derive(Debug)]
pub enum StoreIter {
    Events(EventsIter),
    Indices(Events, IndicesIter),
}

impl StoreIter {
    fn next_map(events: &Events, position: Result<Position>) -> Option<<Self as Iterator>::Item> {
        match position {
            Ok(position) => match events.get(position) {
                Ok(Some(event)) => Some(Ok(event)),
                Ok(None) => None,
                Err(err) => Some(Err(err)),
            },
            Err(err) => Some(Err(err)),
        }
    }
}

impl DoubleEndedIterator for StoreIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Events(iter) => iter.next_back(),
            Self::Indices(events, iter) => iter
                .next_back()
                .and_then(|position| Self::next_map(events, position)),
        }
    }
}

impl Iterator for StoreIter {
    type Item = Result<Event<Metadata, u64>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Events(iter) => iter.next(),
            Self::Indices(events, iter) => iter
                .next()
                .and_then(|position| Self::next_map(events, position)),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::{
    events::Events,
    indices::Indices,
};

// =================================================================================================
// Tests
// =================================================================================================

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use fjall::Database;

    use super::Store;
    use crate::{
        event::{
            Data,
            Event,
            Facets,
            Name,
            Tag,
            Type,
            Version,
        },
        stream::{
            Position,
            operate::{
                Selection,
                select::{
                    Selector,
                    TypeSelector,
                },
            },
        },
        utils::temp_path,
    };

    fn event(identifier: &str, tags: &[&str]) -> Event<(), String> {
        let ty = Type::new(Name::new(identifier).unwrap(), Version::new(0));
        let tags = tags
            .iter()
            .map(|t| Tag::new(*t).unwrap())
            .collect::<BTreeSet<_>>();

        Event::new(
            Data::new(b"payload".to_vec()).unwrap(),
            Facets::new(ty, tags),
            (),
        )
    }

    fn event_v(identifier: &str, version: u8, tags: &[&str]) -> Event<(), String> {
        let ty = Type::new(Name::new(identifier).unwrap(), Version::new(version));
        let tags = tags
            .iter()
            .map(|t| Tag::new(*t).unwrap())
            .collect::<BTreeSet<_>>();

        Event::new(
            Data::new(b"payload".to_vec()).unwrap(),
            Facets::new(ty, tags),
            (),
        )
    }

    // A low-level round-trip driven directly against the `Store` API,
    // independent of the higher-level `Stream`/`Condition` surface: proves the
    // single `String -> u64` insert hop and the events/indices round-trip (with
    // no `references` keyspace) preserve positions.
    #[test]
    fn insert_then_iterate_round_trips_with_positions() {
        let database = Database::builder(temp_path())
            .temporary(true)
            .open()
            .unwrap();
        let store = Store::open(&database).unwrap();

        let events = vec![
            event("StudentSubscribedToCourse", &["student:1", "course:1"]),
            event("StudentSubscribedToCourse", &["student:2", "course:1"]),
            event("CourseCreated", &["course:1"]),
        ];

        let mut next = Position::new(0);
        let last = store
            .insert(&mut || database.batch(), events, &mut next)
            .unwrap();

        assert_eq!(last, Position::new(2));
        assert_eq!(next, Position::new(3));

        let read = store
            .iterate(&[], None)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(read.len(), 3);

        let mut expected = Position::new(0);
        for event in &read {
            assert_eq!(event.2.0, expected);
            expected += 1u64;
        }
    }

    // A reverse query with a `from` lower bound must keep that bound while
    // leapfrogging: `seek_back` re-ranges each leaf to `[from ..= target]`, so a
    // sub-`from` index entry can never be resurrected. Type `evt` is at {2, 7, 8}
    // and tag `k:1` at {2, 7, 9}, so their intersection is {2, 7} — but `from(5)`
    // excludes 2. Both leaves leapfrog down here, so a `seek_back` that dropped the
    // lower bound would re-admit position 2.
    #[test]
    fn reverse_query_with_from_bound_keeps_the_lower_bound() {
        let database = Database::builder(temp_path())
            .temporary(true)
            .open()
            .unwrap();
        let store = Store::open(&database).unwrap();

        let events = vec![
            event("other", &["t:x"]), // 0
            event("other", &["t:x"]), // 1
            event("evt", &["k:1"]),   // 2  type + tag, but below `from`
            event("other", &["t:x"]), // 3
            event("other", &["t:x"]), // 4
            event("other", &["t:x"]), // 5
            event("other", &["t:x"]), // 6
            event("evt", &["k:1"]),   // 7  type + tag, at/above `from`
            event("evt", &["t:x"]),   // 8  type only
            event("other", &["k:1"]), // 9  tag only
        ];

        let mut next = Position::new(0);
        store
            .insert(&mut || database.batch(), events, &mut next)
            .unwrap();

        let selection = Selection::new([Selector::types_and_tags(
            [TypeSelector::new("evt").unwrap()],
            [Tag::new("k:1").unwrap()],
        )]);

        let positions = store
            .iterate(&[selection], Some(Position::new(5)))
            .rev()
            .map(|event| event.unwrap().meta().position())
            .collect::<Vec<_>>();

        // Only the match at 7 (>= the `from` bound of 5), descending; position 2
        // (a genuine type+tag match) stays excluded.
        assert_eq!(positions, vec![Position::new(7)]);
    }

    // A version-range selection filters by version during the type-index scan: one
    // type at v0/v1/v2, queried with versions `0..2` (half-open), matches v0 and v1
    // but not v2.
    #[test]
    fn selects_a_version_range() {
        let database = Database::builder(temp_path())
            .temporary(true)
            .open()
            .unwrap();
        let store = Store::open(&database).unwrap();

        let events = vec![
            event_v("Evt", 0, &["k:1"]), // 0
            event_v("Evt", 1, &["k:1"]), // 1
            event_v("Evt", 2, &["k:1"]), // 2
        ];

        let mut next = Position::new(0);
        store
            .insert(&mut || database.batch(), events, &mut next)
            .unwrap();

        let selection = Selection::new([Selector::types([TypeSelector::with_versions(
            "Evt",
            Version::new(0)..Version::new(2),
        )
        .unwrap()])]);

        let positions = store
            .iterate(&[selection], None)
            .map(|event| event.unwrap().meta().position())
            .collect::<Vec<_>>();

        assert_eq!(positions, vec![Position::new(0), Position::new(1)]);
    }

    // The events keyspace prefixes the tag list with a `u8` count, so more than 255
    // tags is rejected at append (rather than panicking in the serializer), and
    // nothing is committed.
    #[test]
    fn rejects_an_event_with_too_many_tags() {
        let database = Database::builder(temp_path())
            .temporary(true)
            .open()
            .unwrap();
        let store = Store::open(&database).unwrap();

        let ty = Type::new(Name::new("Tagged").unwrap(), Version::new(0));
        let tags = (0u16..256)
            .map(|i| Tag::new(format!("t:{i}")).unwrap())
            .collect::<BTreeSet<_>>();
        let event = Event::new(Data::new(b"x".to_vec()).unwrap(), Facets::new(ty, tags), ());

        let mut next = Position::new(0);
        let result = store.insert(&mut || database.batch(), vec![event], &mut next);

        assert!(result.is_err());
        assert_eq!(next, Position::new(0)); // nothing committed
    }
}
