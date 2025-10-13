pub mod forward;

use eventric_core_model::stream::Position;
use eventric_core_persistence::{
    model::event::Descriptor,
    state::Write,
};

// =================================================================================================
// Descriptor
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

//  Insert

pub fn insert(write: &mut Write<'_>, position: Position, descriptor: &Descriptor) {
    forward::insert(write, position, descriptor);
}
