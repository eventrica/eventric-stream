pub mod forward;

use eventric_core_model::Position;
use eventric_core_persistence::{
    Read,
    TagHash,
    TagHashRef,
    Write,
};
use eventric_core_util::iter;

use crate::iter::SequentialIterator;

// =================================================================================================
// Tags
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, position: Position, tags: &[TagHashRef<'_>]) {
    forward::insert(write, position, tags);
}

// -------------------------------------------------------------------------------------------------

// Query

#[must_use]
pub fn query<'a, T>(read: &Read<'_>, position: Option<Position>, tags: T) -> SequentialIterator
where
    T: Iterator<Item = &'a TagHash>,
{
    iter::sequential_and(tags.map(|tag| forward::iterate(read, position, tag)))
}
