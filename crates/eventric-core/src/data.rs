pub mod events;
pub mod indices;
pub mod references;

use derive_more::Debug;
use fancy_constructor::new;
use fjall::Database;

use crate::data::{
    events::Events,
    indices::Indices,
    references::References,
};

// =================================================================================================
// Data
// =================================================================================================

// Configuration

static HASH_LEN: usize = size_of::<u64>();
static ID_LEN: usize = size_of::<u8>();
static POSITION_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Data

#[derive(new, Debug)]
#[new(const_fn)]
pub struct Data {
    pub(crate) events: Events,
    pub(crate) indices: Indices,
    pub(crate) references: References,
}

impl Data {
    pub fn open(database: &Database) -> Self {
        let events = Events::open(database);
        let indices = Indices::open(database);
        let references = References::open(database);

        Self::new(events, indices, references)
    }
}
