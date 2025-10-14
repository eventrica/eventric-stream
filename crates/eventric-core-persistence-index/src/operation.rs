pub mod descriptor;
pub mod tags;

use eventric_core_model::stream::Position;
use eventric_core_persistence::{
    model::{
        event::EventRef,
        query::{
            QueryItemRef,
            QueryRef,
        },
    },
    state::{
        Read,
        Write,
    },
};
use eventric_core_util::iter::{
    and,
    or,
};

use crate::iter::SequentialIterator;

// =================================================================================================
// Operation
// =================================================================================================

// Configuration

static ID_LEN: usize = size_of::<u8>();
static POSITION_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, position: Position, event: &EventRef<'_>) {
    descriptor::insert(write, position, &event.descriptor);
    tags::insert(write, position, &event.tags);
}

// -------------------------------------------------------------------------------------------------

// Query

#[must_use]
pub fn query(
    read: &Read<'_>,
    position: Option<Position>,
    query: &QueryRef<'_>,
) -> SequentialIterator {
    or::sequential_or(query.items().iter().map(|item| match item {
        QueryItemRef::Specifiers(specifiers) => descriptor::query(read, position, specifiers),
        QueryItemRef::SpecifiersAndTags(specifiers, tags) => and::sequential_and([
            descriptor::query(read, position, specifiers),
            tags::query(read, position, tags),
        ]),
        QueryItemRef::Tags(tags) => tags::query(read, position, tags),
    }))
}
