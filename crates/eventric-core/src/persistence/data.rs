mod event;

use std::error::Error;

use bytes::Buf as _;
use fjall::{
    Keyspace,
    KeyspaceCreateOptions,
};

use crate::{
    model::Position,
    persistence::{
        Context,
        Read,
        Write,
        model::HashedEvent,
    },
};

// =================================================================================================
// Data
// =================================================================================================

static KEYSPACE_NAME: &str = "data";

// -------------------------------------------------------------------------------------------------

// Keyspace

pub fn keyspace(database: &Context) -> Result<Keyspace, Box<dyn Error>> {
    Ok(database
        .as_ref()
        .keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())?)
}

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

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, position: Position, event: &HashedEvent) {
    event::insert(write, position, event);
}
