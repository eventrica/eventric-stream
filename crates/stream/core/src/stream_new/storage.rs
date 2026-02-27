mod events;
mod indices;
mod references;

use std::sync::Exclusive;

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
        operations::Selector,
        storage::events::EventsIterMapped,
    },
};

// =================================================================================================
// Storage
// =================================================================================================

// Constants

static HASH_LEN: usize = size_of::<u64>();
static ID_LEN: usize = size_of::<u8>();
static POSITION_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Storage

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct Storage {
    pub(crate) events: Events,
    pub(crate) indices: Indices,
    pub(crate) references: References,
}

impl Storage {
    pub fn open(database: &Database) -> Result<Self> {
        let events = Events::open(database)?;
        let indices = Indices::open(database)?;
        let references = References::open(database)?;

        Ok(Self::new(events, indices, references))
    }
}

impl Storage {
    pub fn len(&self) -> Result<u64> {
        self.events.len()
    }
}

impl Storage {
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

impl Storage {
    pub fn iterate(
        &self,
        selection: &[Selector<u64>],
        from: Option<Position>,
    ) -> Exclusive<EventsIter> {
        Exclusive::new(
            EventsIterMapped::new(self.events.clone(), self.indices.iterate(selection, from))
                .into(),
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::{
    events::{
        Events,
        EventsIter,
    },
    indices::Indices,
    references::References,
};
