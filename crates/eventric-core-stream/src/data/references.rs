mod identifiers;
mod tags;

use derive_more::Debug;
use eventric_core_error::Error;
use eventric_core_event::{
    EventHashRef,
    identifier::Identifier,
    tag::Tag,
};
use fancy_constructor::new;
use fjall::{
    Database,
    KeyspaceCreateOptions,
    WriteBatch,
};

use crate::data::references::{
    identifiers::Identifiers,
    tags::Tags,
};

// =================================================================================================
// Events
// =================================================================================================

// Configuration

static KEYSPACE_NAME: &str = "references";

// -------------------------------------------------------------------------------------------------

// Data

#[derive(new, Clone, Debug)]
#[new(const_fn, vis())]
pub(crate) struct References {
    identifiers: Identifiers,
    tags: Tags,
}

impl References {
    pub fn open(database: &Database) -> Result<Self, Error> {
        let keyspace = database.keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())?;

        let identifiers = Identifiers::new(keyspace.clone());
        let tags = Tags::new(keyspace);

        Ok(Self::new(identifiers, tags))
    }
}

// Get/Put

impl References {
    pub fn get_identifier(&self, hash: u64) -> Result<Option<Identifier>, Error> {
        self.identifiers.get(hash)
    }

    pub fn get_tag(&self, hash: u64) -> Result<Option<Tag>, Error> {
        self.tags.get(hash)
    }

    pub fn put(&self, batch: &mut WriteBatch, event: &EventHashRef<'_>) {
        self.identifiers.put(batch, &event.identifier);
        self.tags.put(batch, &event.tags);
    }
}
