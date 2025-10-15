pub mod descriptor;
pub mod tags;

use eventric_core_model::Position;
use eventric_core_persistence::{
    EventRef,
    QueryItemRef,
    QueryRef,
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

pub fn insert<'a>(write: &mut Write<'_>, position: Position, event: &'a EventRef<'a>) {
    descriptor::insert(write, position, &event.descriptor);
    tags::insert(write, position, &event.tags);
}

// -------------------------------------------------------------------------------------------------

// Query

pub fn query<'a>(
    read: &Read<'_>,
    position: Option<Position>,
    query: &'a QueryRef<'a>,
) -> impl Iterator<Item = u64> + use<> {
    iter::sequential_or(query.items().iter().map(|item| match item {
        QueryItemRef::Specifiers(specs) => descriptor::query(read, position, specs.iter()),
        QueryItemRef::SpecifiersAndTags(specs, tags) => iter::sequential_and([
            descriptor::query(read, position, specs.iter()),
            tags::query(read, position, tags.iter()),
        ]),
        QueryItemRef::Tags(tags) => tags::query(read, position, tags.iter()),
    }))
}
