pub mod forward;

use eventric_core_model::{
    DescriptorHashRef,
    Position,
    SpecifierHash,
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
    descriptor: &DescriptorHashRef<'_>,
) {
    forward::insert(batch, index, position, descriptor);
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
    iter::sequential_or(specs.map(|spec| forward::iterate(index, position, spec)))
}
