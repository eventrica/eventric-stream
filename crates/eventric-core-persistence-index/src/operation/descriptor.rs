pub mod forward;

use eventric_core_model::stream::Position;
use eventric_core_persistence::{
    model::{
        event::Descriptor,
        query::Specifier,
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

pub fn insert(write: &mut Write<'_>, position: Position, descriptor: &Descriptor) {
    forward::insert(write, position, descriptor);
}

// -------------------------------------------------------------------------------------------------

// Query

#[must_use]
pub fn query(
    read: &Read<'_>,
    position: Option<Position>,
    specifiers: impl IntoIterator<Item = Specifier>,
) -> SequentialIterator {
    or::sequential_or(
        specifiers
            .into_iter()
            .map(|specifier| forward::iterate(read, position, &specifier)),
    )
}
