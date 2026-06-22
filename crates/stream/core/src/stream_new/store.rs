mod events;
mod indices;
mod references;

use error_stack::ResultExt as _;
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

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct Store {
    pub(crate) events: Events,
    pub(crate) indices: Indices,
    pub(crate) references: References,
}

impl Store {
    pub fn open(database: &Database) -> Result<Self> {
        let events = Events::open(database)?;
        let indices = Indices::open(database)?;
        let references = References::open(database)?;

        Ok(Self::new(events, indices, references))
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

            self.references.insert(&mut batch, &event);

            let event = event.into();
            let facets = Timestamp::now()
                .map(|timestamp| Facets::new(position, timestamp))
                .attach("failed to create timestamped facets")?;

            self.events.insert(&mut batch, &event, &facets);
            self.indices.insert(&mut batch, &event, &facets);

            position += 1;
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
    pub fn iterate(&self, selection: Option<Selection>, from: Option<Position>) -> StoreIter {
        if let Some(selection) = selection {
            let events = self.events.clone();
            let iter = self.indices.iterate(&selection.selectors, from);

            StoreIter::Indices(events, iter)
        } else {
            let iter = self.events.iterate(from);

            StoreIter::Events(iter)
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
    references::References,
};
