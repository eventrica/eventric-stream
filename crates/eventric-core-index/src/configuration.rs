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

static KEYSPACE_NAME: &str = "index";

// -------------------------------------------------------------------------------------------------

// Keyspace

pub fn keyspace(context: &Context) -> Result<Keyspace, Box<dyn Error>> {
    Ok(context
        .as_ref()
        .keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())?)
}
