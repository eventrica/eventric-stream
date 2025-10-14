pub mod descriptor;
pub mod tags;

use eventric_core_model::stream::Position;
use eventric_core_persistence::{
    model::{
        event::Event,
        query::{
            Query,
            QueryItem,
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

pub fn insert(write: &mut Write<'_>, position: Position, event: &Event) {
    descriptor::insert(write, position, &event.descriptor);
    tags::insert(write, position, &event.tags);
}

// -------------------------------------------------------------------------------------------------

// Query

#[must_use]
pub fn query(read: &Read<'_>, position: Option<Position>, query: Query) -> SequentialIterator {
    let items: Vec<QueryItem> = query.into();

    or::sequential_or(items.into_iter().map(|item| match item {
        QueryItem::Specifiers(specifiers) => descriptor::query(read, position, specifiers),
        QueryItem::SpecifiersAndTags(specifiers, tags) => and::sequential_and([
            descriptor::query(read, position, specifiers),
            tags::query(read, position, tags),
        ]),
        QueryItem::Tags(tags) => tags::query(read, position, tags),
    }))
}
