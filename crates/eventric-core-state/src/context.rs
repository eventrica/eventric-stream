use std::{
    error::Error,
    path::Path,
};

use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Database,
    Keyspace,
};

// =================================================================================================
// Context
// =================================================================================================

#[derive(Debug)]
pub struct Context {
    #[debug("Database")]
    database: Database,
}

impl Context {
    pub fn new<P>(path: P, temporary: bool) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let database = Database::builder(path).temporary(temporary).open()?;

        Ok(Self { database })
    }
}

impl Context {
    #[must_use]
    pub fn database(&self) -> &Database {
        &self.database
    }
}

// -------------------------------------------------------------------------------------------------

// Keyspaces

#[derive(new, Clone, Debug)]
#[new(const_fn)]
pub struct Keyspaces {
    #[debug("Keyspace(\"{}\")", data.name)]
    pub data: Keyspace,
    #[debug("Keyspace(\"{}\")", index.name)]
    pub index: Keyspace,
    #[debug("Keyspace(\"{}\")", reference.name)]
    pub reference: Keyspace,
}
