mod descriptor;
mod tags;

use eventric_core_model::EventHashRef;
use fjall::{
    Keyspace,
    WriteBatch,
};

// =================================================================================================
// Reference
// =================================================================================================

// Configuration

static ID_LEN: usize = size_of::<u8>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(batch: &mut WriteBatch, reference: &Keyspace, event: &EventHashRef<'_>) {
    descriptor::insert(batch, reference, event.descriptor());
    tags::insert(batch, reference, event.tags());
}
