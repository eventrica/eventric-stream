mod identifiers;
mod tags;

use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Database,
    KeyspaceCreateOptions,
    WriteBatch,
};

use crate::{
    data::references::{
        identifiers::Identifiers,
        tags::Tags,
    },
    error::Error,
    model::event::{
        EventHashRef,
        identifier::Identifier,
        tag::Tag,
    },
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
pub struct References {
    identifiers: Identifiers,
    tags: Tags,
}

impl References {
    pub fn open(database: &Database) -> Self {
        let keyspace = database
            .keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())
            .map_err(Error::from)
            .expect("references keyspace open: database error");

        let identifiers = Identifiers::new(keyspace.clone());
        let tags = Tags::new(keyspace);

        Self::new(identifiers, tags)
    }
}

// Get/Put

impl References {
    pub fn get_identifier(&self, hash: u64) -> Option<Identifier> {
        self.identifiers.get(hash)
    }

    pub fn get_tag(&self, hash: u64) -> Option<Tag> {
        self.tags.get(hash)
    }

    pub fn put(&self, batch: &mut WriteBatch, event: &EventHashRef<'_>) {
        self.identifiers.put(batch, event.identifier());
        self.tags.put(batch, event.tags());
    }
}
