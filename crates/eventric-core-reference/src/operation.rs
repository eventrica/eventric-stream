mod descriptor;
mod tags;

use eventric_core_model::AppendEventHashRef;
use eventric_core_state::Write;

// =================================================================================================
// Reference
// =================================================================================================

// Configuration

static ID_LEN: usize = size_of::<u8>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, event: &AppendEventHashRef<'_>) {
    descriptor::insert(write, &event.descriptor);
    tags::insert(write, &event.tags);
}
