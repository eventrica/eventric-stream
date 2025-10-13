pub mod descriptor;
pub mod tags;

use eventric_core_model::stream::Position;
use eventric_core_persistence::{
    model::event::Event,
    state::Write,
};

// =================================================================================================
// Operation
// =================================================================================================

// Configuration

static ID_LEN: usize = size_of::<u8>();
static POSITION_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, position: Position, event: &Event) {
    descriptor::insert(write, position, &event.descriptor);
    tags::insert(write, position, &event.tags);
}
