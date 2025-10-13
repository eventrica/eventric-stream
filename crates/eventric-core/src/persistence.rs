pub mod data;
pub mod index;
pub mod model;
pub mod operation;
pub mod reference;

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

use crate::{
    model::{
        event::insertion::Event,
        stream::Position,
    },
    persistence::operation::Write,
};

// =================================================================================================
// Persistence
// =================================================================================================

// Context

#[derive(new, Debug)]
#[new(vis())]
pub struct Context {
    #[debug("Database")]
    database: Database,
}

impl AsRef<Database> for Context {
    fn as_ref(&self) -> &Database {
        &self.database
    }
}

pub fn context<P>(path: P) -> Result<Context, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    Ok(Database::builder(path).open().map(Context::new)?)
}

// -------------------------------------------------------------------------------------------------

// Keyspaces

#[derive(new, Clone, Debug)]
#[new(vis())]
pub struct Keyspaces {
    #[debug("Keyspace(\"{}\")", data.name)]
    data: Keyspace,
    #[debug("Keyspace(\"{}\")", index.name)]
    index: Keyspace,
    #[debug("Keyspace(\"{}\")", reference.name)]
    reference: Keyspace,
}

pub fn keyspaces(context: &Context) -> Result<Keyspaces, Box<dyn Error>> {
    Ok(Keyspaces::new(
        data::keyspace(context)?,
        index::keyspace(context)?,
        reference::keyspace(context)?,
    ))
}

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, position: Position, event: Event) {
    let event = event.into();

    data::insert(write, position, &event);
    index::insert(write, position, &event);
    reference::insert(write, &event);
}
