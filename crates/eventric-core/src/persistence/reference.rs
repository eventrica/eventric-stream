mod descriptor;
mod tags;

use std::error::Error;

use fjall::{
    Keyspace,
    KeyspaceCreateOptions,
};

use crate::persistence::{
    Context,
    Write,
    model::HashedEvent,
};

// =================================================================================================
// Reference
// =================================================================================================

static ID_LEN: usize = size_of::<u8>();
static KEYSPACE_NAME: &str = "reference";

// -------------------------------------------------------------------------------------------------

// Keyspace

pub fn keyspace(context: &Context) -> Result<Keyspace, Box<dyn Error>> {
    Ok(context
        .as_ref()
        .keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())?)
}

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, event: &HashedEvent) {
    descriptor::insert(write, &event.descriptor);
    tags::insert(write, &event.tags);
}
