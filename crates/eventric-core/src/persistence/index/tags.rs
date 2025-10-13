pub mod forward;

use crate::{
    model::stream::Position,
    persistence::{
        model::event::Tag,
        operation::Write,
    },
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
