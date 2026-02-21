pub(crate) mod events;
pub(crate) mod indices;
pub(crate) mod new_events;
pub(crate) mod new_indices;
pub(crate) mod references;

use derive_more::Debug;
use fancy_constructor::new;
use fjall::Database;

use crate::{
    error::Error,
    stream::data::{
        events::Events,
        indices::Indices,
        new_events::NewEvents,
        new_indices::NewIndices,
        references::References,
    },
};

// =================================================================================================
// Data
// =================================================================================================

// Configuration

static HASH_LEN: usize = size_of::<u64>();
static ID_LEN: usize = size_of::<u8>();
static POSITION_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

#[derive(new, Clone, Debug)]
#[new(const_fn)]
pub struct NewData {
    pub events: NewEvents,
    pub indices: NewIndices,
}

impl NewData {
    pub fn open(database: &Database) -> Result<Self, Error> {
        let events = NewEvents::open(database)?;
        let indices = NewIndices::open(database)?;

        Ok(Self::new(events, indices))
    }
}

// Data

#[derive(new, Clone, Debug)]
#[new(const_fn)]
pub struct Data {
    pub events: Events,
    pub indices: Indices,
    pub references: References,
}

impl Data {
    pub fn open(database: &Database) -> Result<Self, Error> {
        let events = Events::open(database)?;
        let indices = Indices::open(database)?;
        let references = References::open(database)?;

        Ok(Self::new(events, indices, references))
    }
}
