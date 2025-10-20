mod lookup;

use eventric_core_model::{
    Identifier,
    IdentifierHashRef,
};
use fjall::{
    Keyspace,
    WriteBatch,
};

// =================================================================================================
// Identifier
// =================================================================================================

// Configuration

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Get

pub fn get(reference: &Keyspace, hash: u64) -> Option<Identifier> {
    lookup::get(reference, hash)
}

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(batch: &mut WriteBatch, reference: &Keyspace, identifier: &IdentifierHashRef<'_>) {
    lookup::insert(batch, reference, identifier);
}
