use std::{
    error::Error,
    path::Path,
};

use derive_more::Debug;
use fjall::Database;

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
