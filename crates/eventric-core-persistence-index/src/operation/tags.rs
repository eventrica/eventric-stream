pub mod forward;

use eventric_core_model::stream::Position;
use eventric_core_persistence::{
    model::event::Tag,
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

pub fn insert(write: &mut Write<'_>, position: Position, tags: &[Tag]) {
    forward::insert(write, position, tags);
}

// -------------------------------------------------------------------------------------------------

// Query

#[must_use]
pub fn query(
    read: &Read<'_>,
    position: Option<Position>,
    tags: impl IntoIterator<Item = Tag>,
) -> SequentialIterator {
    and::sequential_and(
        tags.into_iter()
            .map(|tag| forward::iterate(read, position, &tag)),
    )
}
