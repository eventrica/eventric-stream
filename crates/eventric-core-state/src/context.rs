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
    pub fn new<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let database = Database::builder(path).open()?;

        Ok(Self { database })
    }
}

impl AsRef<Database> for Context {
    fn as_ref(&self) -> &Database {
        &self.database
    }
}

// -------------------------------------------------------------------------------------------------

// Keyspaces

#[derive(new, Clone, Debug)]
#[new(vis(pub))]
pub struct Keyspaces {
    #[debug("Keyspace(\"{}\")", data.name)]
    pub data: Keyspace,
    #[debug("Keyspace(\"{}\")", index.name)]
    pub index: Keyspace,
    #[debug("Keyspace(\"{}\")", reference.name)]
    pub reference: Keyspace,
}
