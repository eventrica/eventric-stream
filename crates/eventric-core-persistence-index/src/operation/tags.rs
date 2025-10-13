pub mod forward;

use eventric_core_model::stream::Position;
use eventric_core_persistence::{
    model::event::Tag,
    state::Write,
};

// =================================================================================================
// Tags
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, position: Position, tags: &[Tag]) {
    forward::insert(write, position, tags);
}
