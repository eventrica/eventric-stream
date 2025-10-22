pub mod forward;

use eventric_core_model::{
    IdentifierHashRef,
    Position,
    SpecifierHash,
    Version,
};
use eventric_core_util::iter;
use fjall::{
    Keyspace,
    WriteBatch,
};

use crate::iter::SequentialPositionIterator;

// =================================================================================================
// Descriptor
// =================================================================================================

// Configuration

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

//  Insert

pub fn insert(
    batch: &mut WriteBatch,
    index: &Keyspace,
    position: Position,
    identifier: &IdentifierHashRef<'_>,
    version: Version,
) {
    forward::insert(batch, index, position, identifier, version);
}

// -------------------------------------------------------------------------------------------------

// Query

#[must_use]
pub fn query<'a, S>(
    index: &Keyspace,
    position: Option<Position>,
    specs: S,
) -> SequentialPositionIterator
where
    S: Iterator<Item = &'a SpecifierHash>,
{
    iter::sequential_or(specs.map(|spec| forward::iterate(index.clone(), position, spec)))
}
