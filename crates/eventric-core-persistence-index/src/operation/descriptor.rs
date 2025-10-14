pub mod forward;

use eventric_core_model::stream::Position;
use eventric_core_persistence::{
    model::{
        event::DescriptorRef,
        query::SpecifierRef,
    },
    state::{
        Read,
        Write,
    },
};
use eventric_core_util::iter::or;

use crate::iter::SequentialIterator;

// =================================================================================================
// Descriptor
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

//  Insert

pub fn insert<'a>(write: &mut Write<'_>, position: Position, descriptor: &'a DescriptorRef<'a>) {
    forward::insert(write, position, descriptor);
}

// -------------------------------------------------------------------------------------------------

// Query

#[must_use]
pub fn query<'a, S>(read: &Read<'_>, position: Option<Position>, specs: S) -> SequentialIterator
where
    S: Iterator<Item = &'a SpecifierRef<'a>>,
{
    or::sequential_or(specs.map(|spec| forward::iterate(read, position, spec)))
}
