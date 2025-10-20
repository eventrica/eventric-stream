pub mod forward;

use eventric_core_model::{
    DescriptorHashRef,
    Position,
    SpecifierHash,
};
use eventric_core_state::{
    Read,
    Write,
};
use eventric_core_util::iter;

use crate::iter::SequentialPositionIterator;

// =================================================================================================
// Descriptor
// =================================================================================================

// Configuration

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

//  Insert

pub fn insert(write: &mut Write<'_>, position: Position, descriptor: &DescriptorHashRef<'_>) {
    forward::insert(write, position, descriptor);
}

// -------------------------------------------------------------------------------------------------

// Query

#[must_use]
pub fn query<'a, S>(
    read: &Read<'_>,
    position: Option<Position>,
    specs: S,
) -> SequentialPositionIterator
where
    S: Iterator<Item = &'a SpecifierHash>,
{
    iter::sequential_or(specs.map(|spec| forward::iterate(read, position, spec)))
}
