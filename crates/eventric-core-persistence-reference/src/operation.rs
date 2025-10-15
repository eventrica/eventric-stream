mod descriptor;
mod tags;

use eventric_core_persistence::{
    EventRef,
    Write,
};

// =================================================================================================
// Reference
// =================================================================================================

// Configuration

static ID_LEN: usize = size_of::<u8>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert<'a>(write: &mut Write<'_>, event: &'a EventRef<'a>) {
    descriptor::insert(write, &event.descriptor);
    tags::insert(write, &event.tags);
}
