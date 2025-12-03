pub(crate) mod identifiers;
pub(crate) mod tags;

use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Database,
    KeyspaceCreateOptions,
    OwnedWriteBatch,
};

use crate::{
    error::Error,
    event::{
        CandidateEventHashAndValue,
        identifier::{
            Identifier,
            IdentifierHash,
        },
        tag::{
            Tag,
            TagHash,
        },
    },
    stream::data::references::{
        identifiers::Identifiers,
        tags::Tags,
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
pub(crate) struct References {
    identifiers: Identifiers,
    tags: Tags,
}

impl References {
    pub fn open(database: &Database) -> Result<Self, Error> {
        let keyspace = database.keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default)?;

        let identifiers = Identifiers::new(keyspace.clone());
        let tags = Tags::new(keyspace);

        Ok(Self::new(identifiers, tags))
    }
}

// Get/Put

impl References {
    pub fn get_identifier(&self, identifier: IdentifierHash) -> Result<Option<Identifier>, Error> {
        self.identifiers.get(identifier)
    }

    pub fn get_tag(&self, tag: TagHash) -> Result<Option<Tag>, Error> {
        self.tags.get(tag)
    }

    pub fn put(&self, batch: &mut OwnedWriteBatch, event: &CandidateEventHashAndValue) {
        self.identifiers.put(batch, &event.identifier);
        self.tags.put(batch, &event.tags);
    }
}
