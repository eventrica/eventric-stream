mod events;
mod indices;
mod references;

use fancy_constructor::new;
use fjall::{
    Database,
    OwnedWriteBatch as Batch,
};

use crate::{
    error::Error,
    event_new::Event,
    stream_new::{
        Facets,
        storage::{
            events::Events,
            indices::Indices,
            references::References,
        },
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
    events: Events,
    indices: Indices,
    references: References,
}

impl Storage {
    pub fn open(database: &Database) -> Result<Self, Error> {
        let events = Events::open(database)?;
        let indices = Indices::open(database)?;
        let references = References::open(database)?;

        Ok(Self::new(events, indices, references))
    }
}

impl Storage {
    pub fn insert(&self, batch: &mut Batch, event: Event<(), String>, facets: &Facets) {
        let event = event.into();

        self.references.insert(batch, &event);

        let event = event.into();

        self.events.insert(batch, &event, facets);
        self.indices.insert(batch, &event, facets);
    }
}
