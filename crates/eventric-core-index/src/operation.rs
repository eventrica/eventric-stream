pub mod identifier;
pub mod tags;
pub mod timestamp;

use eventric_core_model::{
    EventHashRef,
    Position,
    QueryHash,
    QueryItemHash,
};
use eventric_core_util::iter;
use fjall::{
    Keyspace,
    WriteBatch,
};

use crate::iter::SequentialPositionIterator;

// =================================================================================================
// Operation
// =================================================================================================

// Configuration

static ID_LEN: usize = size_of::<u8>();
static POSITION_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(
    batch: &mut WriteBatch,
    index: &Keyspace,
    event: &EventHashRef<'_>,
    position: Position,
) {
    identifier::insert(batch, index, position, event.identifier(), *event.version());
    tags::insert(batch, index, position, event.tags());
    timestamp::insert(batch, index, position, *event.timestamp());
}

// -------------------------------------------------------------------------------------------------

// Query

#[must_use]
pub fn query(
    index: &Keyspace,
    position: Option<Position>,
    query: &QueryHash,
) -> SequentialPositionIterator {
    iter::sequential_or(query.items().iter().map(|item| match item {
        QueryItemHash::Specifiers(specs) => identifier::query(index, position, specs.iter()),
        QueryItemHash::SpecifiersAndTags(specs, tags) => iter::sequential_and([
            identifier::query(index, position, specs.iter()),
            tags::query(index, position, tags.iter()),
        ]),
        QueryItemHash::Tags(tags) => tags::query(index, position, tags.iter()),
    }))
}
