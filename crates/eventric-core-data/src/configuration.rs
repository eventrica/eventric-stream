use std::error::Error;

use eventric_core_state::Context;
use fjall::{
    Keyspace,
    KeyspaceCreateOptions,
};

// =================================================================================================
// Configuration
// =================================================================================================

// Configuration

static KEYSPACE_NAME: &str = "data";

// -------------------------------------------------------------------------------------------------

// Keyspace

pub fn keyspace(database: &Context) -> Result<Keyspace, Box<dyn Error>> {
    Ok(database
        .as_ref()
        .keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())?)
}
