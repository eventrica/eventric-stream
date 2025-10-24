mod identifiers;
mod tags;
mod timestamps;

use std::error::Error;

use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Database,
    KeyspaceCreateOptions,
    WriteBatch,
};

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

#[derive(new, Clone, Debug)]
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

// Contains

impl Indices {
    pub fn contains(&self, query: &QueryHash, position: Option<Position>) -> bool {
        self.query(query, position).any(|_| true)
    }
}

// Put

impl Indices {
    #[rustfmt::skip]
    pub fn put(
        &self,
        batch: &mut WriteBatch,
        event: &EventHashRef<'_>,
        timestamp: Timestamp,
        position: Position,
    ) {
        self.identifiers.put(batch, position, event.identifier(), *event.version());
        self.tags.put(batch, position, event.tags());
        self.timestamps.put(batch, position, timestamp);
    }
}

// Query

impl Indices {
    #[must_use]
    pub fn query(&self, query: &QueryHash, position: Option<Position>) -> SequentialIterator<'_> {
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
pub enum SequentialIterator<'a> {
    And(SequentialAndIterator<SequentialIterator<'a>, Position>),
    Or(SequentialOrIterator<SequentialIterator<'a>, Position>),
    Owned(#[debug("OwnedSequentialIterator")] Box<dyn Iterator<Item = Position> + 'a>),
}

impl Iterator for SequentialIterator<'_> {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iterator) => iterator.next(),
            Self::Or(iterator) => iterator.next(),
            Self::Owned(iterator) => iterator.next(),
        }
    }
}

impl<'a> From<SequentialAndIterator<SequentialIterator<'a>, Position>> for SequentialIterator<'a> {
    fn from(value: SequentialAndIterator<SequentialIterator<'a>, Position>) -> Self {
        Self::And(value)
    }
}

impl<'a> From<SequentialOrIterator<SequentialIterator<'a>, Position>> for SequentialIterator<'a> {
    fn from(value: SequentialOrIterator<SequentialIterator<'a>, Position>) -> Self {
        Self::Or(value)
    }
}

impl<'a> From<Box<dyn Iterator<Item = Position> + 'a>> for SequentialIterator<'a> {
    fn from(value: Box<dyn Iterator<Item = Position> + 'a>) -> Self {
        Self::Owned(value)
    }
}
