pub mod forward;

use eventric_core_model::{
    Position,
    Timestamp,
};
use fjall::{
    Keyspace,
    WriteBatch,
};

// =================================================================================================
// Timestamp
// =================================================================================================

// Configuration

static TIMESTAMP_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

//  Insert

pub fn insert(batch: &mut WriteBatch, index: &Keyspace, position: Position, timestamp: Timestamp) {
    forward::insert(batch, index, position, timestamp);
}
