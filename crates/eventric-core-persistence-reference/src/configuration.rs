use std::error::Error;

use eventric_core_persistence::Context;
use fjall::{
    Keyspace,
    KeyspaceCreateOptions,
};

// =================================================================================================
// Configuration
// =================================================================================================

// Configuration

static KEYSPACE_NAME: &str = "reference";

// -------------------------------------------------------------------------------------------------

// Keyspace

pub fn keyspace(context: &Context) -> Result<Keyspace, Box<dyn Error>> {
    Ok(context
        .as_ref()
        .keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())?)
}
