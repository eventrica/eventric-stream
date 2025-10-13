pub mod forward;

use crate::{
    model::Position,
    persistence::{
        Write,
        model::event::HashedTag,
    },
};

// =================================================================================================
// Tags
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, position: Position, tags: &[HashedTag]) {
    forward::insert(write, position, tags);
}
