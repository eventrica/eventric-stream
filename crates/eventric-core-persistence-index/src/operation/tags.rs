pub mod forward;

use eventric_core_model::stream::Position;
use eventric_core_persistence::{
    model::event::TagRef,
    state::{
        Read,
        Write,
    },
};
use eventric_core_util::iter::and;

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
    and::sequential_and(tags.map(|tag| forward::iterate(read, position, tag)))
}
