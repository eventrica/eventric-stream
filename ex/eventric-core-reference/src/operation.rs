mod identifier;
mod tags;

use eventric_core_model::{
    EventHashRef,
    Identifier,
    Tag,
};
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

// Get

#[must_use]
pub fn get_identifier(reference: &Keyspace, hash: u64) -> Option<Identifier> {
    identifier::get(reference, hash)
}

#[must_use]
pub fn get_tag(reference: &Keyspace, hash: u64) -> Option<Tag> {
    tags::get(reference, hash)
}

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(batch: &mut WriteBatch, reference: &Keyspace, event: &EventHashRef<'_>) {
    identifier::insert(batch, reference, event.identifier());
    tags::insert(batch, reference, event.tags());
}

// -------------------------------------------------------------------------------------------------
