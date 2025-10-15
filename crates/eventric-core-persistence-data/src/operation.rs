mod event;

use std::error::Error;

use bytes::Buf as _;
use eventric_core_model::Position;
use eventric_core_persistence::{
    EventHash,
    EventHashRef,
    Read,
    Write,
};

// =================================================================================================
// Operation
// =================================================================================================

// Get

pub fn get(read: &Read<'_>, position: Position) -> Result<Option<EventHash>, Box<dyn Error>> {
    event::get(read, position)
}

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert<'a>(write: &mut Write<'_>, position: Position, event: &'a EventHashRef<'a>) {
    event::insert(write, position, event);
}

// -------------------------------------------------------------------------------------------------

// Properties

pub fn is_empty(read: &Read<'_>) -> Result<bool, Box<dyn Error>> {
    len(read).map(|len| len == 0)
}

pub fn len(read: &Read<'_>) -> Result<u64, Box<dyn Error>> {
    let key_value = read.keyspaces.data.last_key_value()?;

    if let Some((key, _)) = key_value {
        let key = key.as_ref().get_u64();
        let len = key + 1;

        Ok(len)
    } else {
        Ok(0)
    }
}
