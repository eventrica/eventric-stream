pub mod descriptor;
pub mod tags;

use eventric_core_model::{
    EventHashRef,
    Position,
    QueryHash,
    QueryItemHash,
};
use eventric_core_state::{
    Read,
    Write,
};
use eventric_core_util::iter;

// =================================================================================================
// Operation
// =================================================================================================

// Configuration

static ID_LEN: usize = size_of::<u8>();
static POSITION_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, position: Position, event: &EventHashRef<'_>) {
    descriptor::insert(write, position, &event.descriptor);
    tags::insert(write, position, &event.tags);
}

// -------------------------------------------------------------------------------------------------

// Query

pub fn query(
    read: &Read<'_>,
    position: Option<Position>,
    query: &QueryHash,
) -> impl Iterator<Item = u64> + use<> {
    iter::sequential_or(query.items().iter().map(|item| match item {
        QueryItemHash::Specifiers(specs) => descriptor::query(read, position, specs.iter()),
        QueryItemHash::SpecifiersAndTags(specs, tags) => iter::sequential_and([
            descriptor::query(read, position, specs.iter()),
            tags::query(read, position, tags.iter()),
        ]),
        QueryItemHash::Tags(tags) => tags::query(read, position, tags.iter()),
    }))
}
