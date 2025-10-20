mod event;

use std::error::Error;

use bytes::Buf as _;
use eventric_core_model::{
    EventHashRef,
    Position,
    SequencedEventHash,
};
use fjall::{
    Keyspace,
    WriteBatch,
};

// =================================================================================================
// Operation
// =================================================================================================

// Get

pub fn get(
    data: &Keyspace,
    position: Position,
) -> Result<Option<SequencedEventHash>, Box<dyn Error>> {
    event::get(data, position)
}

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(
    batch: &mut WriteBatch,
    data: &Keyspace,
    event: &EventHashRef<'_>,
    position: Position,
) {
    event::insert(batch, data, event, position);
}

// -------------------------------------------------------------------------------------------------

// Properties

pub fn is_empty(data: &Keyspace) -> Result<bool, Box<dyn Error>> {
    len(data).map(|len| len == 0)
}

pub fn len(data: &Keyspace) -> Result<u64, Box<dyn Error>> {
    let key_value = data.last_key_value()?;

    if let Some((key, _)) = key_value {
        let key = key.as_ref().get_u64();
        let len = key + 1;

        Ok(len)
    } else {
        Ok(0)
    }
}
