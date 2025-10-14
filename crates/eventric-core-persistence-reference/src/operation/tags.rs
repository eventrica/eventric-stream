mod lookup;

use eventric_core_persistence::{
    model::event::TagRef,
    state::Write,
};

// =================================================================================================
// Tags
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert<'a>(write: &mut Write<'_>, tags: &'a [TagRef<'a>]) {
    lookup::insert(write, tags);
}
