pub mod forward;

use eventric_core_model::{
    Position,
    Timestamp,
};
use eventric_core_state::Write;

// =================================================================================================
// Timestamp
// =================================================================================================

// Configuration

static TIMESTAMP_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

//  Insert

pub fn insert(write: &mut Write<'_>, position: Position, timestamp: Timestamp) {
    forward::insert(write, position, timestamp);
}
