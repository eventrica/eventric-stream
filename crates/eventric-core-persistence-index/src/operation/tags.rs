pub mod forward;

use eventric_core_model::Position;
use eventric_core_persistence::{
    Read,
    TagRef,
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

pub fn insert<'a>(write: &mut Write<'_>, position: Position, tags: &'a [TagRef<'a>]) {
    forward::insert(write, position, tags);
}

// -------------------------------------------------------------------------------------------------

// Query

#[must_use]
pub fn query<'a, T>(read: &Read<'_>, position: Option<Position>, tags: T) -> SequentialIterator
where
    T: Iterator<Item = &'a TagRef<'a>>,
{
    iter::sequential_and(tags.map(|tag| forward::iterate(read, position, tag)))
}
