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
    event_new::Event,
    stream_new::{
        Error,
        Facets,
        Position,
        Result,
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
            let event = event.into();

            let facets = Timestamp::now()
                .map(|timestamp| Facets::new(position, timestamp))
                .attach("failed to create timestamped facets")?;

            self.events.insert(&mut batch, &event, &facets);
            self.indices.insert(&mut batch, &event, &facets);

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
    pub fn iterate(&self, selections: &[Selection], from: Option<Position>) -> StoreIter {
        if selections.is_empty() {
            StoreIter::Events(self.events.iterate(from))
        } else {
            let events = self.events.clone();
            let iter = self.indices.iterate(
                selections
                    .iter()
                    .flat_map(|selection| selection.selectors.iter()),
                from,
            );

            StoreIter::Indices(events, iter)
        }
    }
}

impl Store {
    /// Whether any event matching `selections` exists at or after `from`. Used
    /// for the append concurrency (DCB) check; resolves index positions only,
    /// so it never materializes an event. Empty `selections` is vacuously
    /// `false`.
    pub fn matches(&self, selections: &[Selection], from: Option<Position>) -> Result<bool> {
        let mut iter = self.indices.iterate(
            selections
                .iter()
                .flat_map(|selection| selection.selectors.iter()),
            from,
        );

        match iter.next() {
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
    type Item = Result<Event<Facets, u64>>;

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
        event_new::{
            Data,
            Event,
            Facets,
            Name,
            Tag,
            Type,
            Version,
        },
        stream_new::Position,
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

    // Phase 1 is a data-format change to the new tree, which otherwise has no
    // tests. This drives the pub `Store` API directly (no dependency on the
    // not-yet-built public `Stream`/`Condition` surface) to prove the collapsed
    // single `String -> u64` insert hop and the events/indices round-trip still
    // work after the `references` keyspace was removed.
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
}
