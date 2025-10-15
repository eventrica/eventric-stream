pub mod descriptor;
pub mod tags;

use eventric_core_model::Position;
use eventric_core_persistence::{
    EventHashRef,
    QueryHashRef,
    QueryItemHashRef,
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

pub fn insert<'a>(write: &mut Write<'_>, position: Position, event: &'a EventHashRef<'a>) {
    descriptor::insert(write, position, &event.descriptor);
    tags::insert(write, position, &event.tags);
}

// -------------------------------------------------------------------------------------------------

// Query

pub fn query<'a>(
    read: &Read<'_>,
    position: Option<Position>,
    query: &'a QueryHashRef<'a>,
) -> impl Iterator<Item = u64> + use<> {
    iter::sequential_or(query.items().iter().map(|item| match item {
        QueryItemHashRef::Specifiers(specs) => descriptor::query(read, position, specs.iter()),
        QueryItemHashRef::SpecifiersAndTags(specs, tags) => iter::sequential_and([
            descriptor::query(read, position, specs.iter()),
            tags::query(read, position, tags.iter()),
        ]),
        QueryItemHashRef::Tags(tags) => tags::query(read, position, tags.iter()),
    }))
}
