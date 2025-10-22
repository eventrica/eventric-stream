use std::error::Error;

use fjall::{
    Database,
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

pub fn keyspace(database: &Database) -> Result<Keyspace, Box<dyn Error>> {
    Ok(database.keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())?)
}
