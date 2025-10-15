mod lookup;

use eventric_core_persistence::TagHashRef;
use eventric_core_state::Write;

// =================================================================================================
// Tags
// =================================================================================================

static HASH_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, tags: &[TagHashRef<'_>]) {
    lookup::insert(write, tags);
}
