pub mod forward;

use eventric_core_model::{
    Position,
    TagHash,
    TagHashRef,
};
use eventric_core_util::iter;
use fjall::{
    Keyspace,
    WriteBatch,
};

use crate::iter::SequentialPositionIterator;

// =================================================================================================
// Tags
// =================================================================================================

// Configuration

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(
    batch: &mut WriteBatch,
    index: &Keyspace,
    position: Position,
    tags: &[TagHashRef<'_>],
) {
    forward::insert(batch, index, position, tags);
}

// -------------------------------------------------------------------------------------------------

// Query

#[must_use]
pub fn query<'a, T>(
    index: &Keyspace,
    position: Option<Position>,
    tags: T,
) -> SequentialPositionIterator
where
    T: Iterator<Item = &'a TagHash>,
{
    iter::sequential_and(tags.map(|tag| forward::iterate(index, position, tag)))
}
