mod identifiers;
mod tags;
mod timestamps;

use std::error::Error;

use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Database,
    Keyspace,
    KeyspaceCreateOptions,
    WriteBatch,
};
use self_cell::self_cell;

use crate::{
    data::indices::{
        identifiers::Identifiers,
        tags::Tags,
        timestamps::Timestamps,
    },
    model::{
        event::{
            EventHashRef,
            timestamp::Timestamp,
        },
        query::{
            QueryHash,
            QueryItemHash,
        },
        stream::position::Position,
    },
    util::iter::{
        SequentialAndIterator,
        SequentialOrIterator,
    },
};

// =================================================================================================
// Indicies
// =================================================================================================

// Configuration

static KEYSPACE_NAME: &str = "indices";

// -------------------------------------------------------------------------------------------------

// Indices

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct Indices {
    identifiers: Identifiers,
    tags: Tags,
    timestamps: Timestamps,
}

impl Indices {
    pub fn open(database: &Database) -> Result<Self, Box<dyn Error>> {
        let keyspace = database.keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())?;

        let identifiers = Identifiers::new(keyspace.clone());
        let tags = Tags::new(keyspace.clone());
        let timestamps = Timestamps::new(keyspace);

        Ok(Self::new(identifiers, tags, timestamps))
    }
}

// Put

impl Indices {
    #[rustfmt::skip]
    pub fn put(
        &self,
        batch: &mut WriteBatch,
        position: Position,
        event: &EventHashRef<'_>,
        timestamp: Timestamp,
    ) {
        self.identifiers.put(batch, position, event.identifier(), *event.version());
        self.tags.put(batch, position, event.tags());
        self.timestamps.put(batch, position, timestamp);
    }
}

// Query

impl Indices {
    #[must_use]
    pub fn query(&self, query: &QueryHash, position: Option<Position>) -> SequentialIterator {
        SequentialOrIterator::combine(query.items().iter().map(|item| match item {
            QueryItemHash::Specifiers(specifiers) => {
                self.identifiers.query(specifiers.iter(), position)
            }
            QueryItemHash::SpecifiersAndTags(specifiers, tags) => SequentialAndIterator::combine([
                self.identifiers.query(specifiers.iter(), position),
                self.tags.query(tags.iter(), position),
            ]),
            QueryItemHash::Tags(tags) => self.tags.query(tags.iter(), position),
        }))
    }
}

// -------------------------------------------------------------------------------------------------

// Iterators

// Sequential Iterator

#[derive(Debug)]
pub enum SequentialIterator {
    And(SequentialAndIterator<SequentialIterator, Position>),
    Or(SequentialOrIterator<SequentialIterator, Position>),
    Owned(#[debug("OwnedSequentialIterator")] OwnedSequentialIterator),
}

impl Iterator for SequentialIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iterator) => iterator.next(),
            Self::Or(iterator) => iterator.next(),
            Self::Owned(iterator) => iterator.next(),
        }
    }
}

impl From<SequentialAndIterator<SequentialIterator, Position>> for SequentialIterator {
    fn from(value: SequentialAndIterator<SequentialIterator, Position>) -> Self {
        Self::And(value)
    }
}

impl From<SequentialOrIterator<SequentialIterator, Position>> for SequentialIterator {
    fn from(value: SequentialOrIterator<SequentialIterator, Position>) -> Self {
        Self::Or(value)
    }
}

impl From<OwnedSequentialIterator> for SequentialIterator {
    fn from(value: OwnedSequentialIterator) -> Self {
        Self::Owned(value)
    }
}

// Boxed Sequential Iterator

type BoxedSequentialIterator<'a> = Box<dyn Iterator<Item = Position> + 'a>;

// Owned Sequential Position Iterator

self_cell!(
    pub struct OwnedSequentialIterator {
        owner: Keyspace,
        #[covariant]
        dependent: BoxedSequentialIterator,
    }
);

impl Iterator for OwnedSequentialIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_dependent_mut(|_, iterator| iterator.next())
    }
}
