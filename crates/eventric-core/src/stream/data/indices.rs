pub(crate) mod identifiers;
pub(crate) mod tags;
pub(crate) mod timestamps;

use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Database,
    KeyspaceCreateOptions,
    WriteBatch,
};

use crate::{
    error::Error,
    event::{
        EphemeralEventHashRef,
        position::Position,
        timestamp::Timestamp,
    },
    stream::{
        data::indices::{
            identifiers::{
                IdentifierPositionIterator,
                Identifiers,
            },
            tags::{
                TagPositionIterator,
                Tags,
            },
            timestamps::Timestamps,
        },
        query::{
            QueryHash,
            SelectorHash,
        },
    },
    utils::iteration::{
        and::SequentialAndIterator,
        or::SequentialOrIterator,
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
pub(crate) struct Indices {
    identifiers: Identifiers,
    tags: Tags,
    timestamps: Timestamps,
}

impl Indices {
    pub fn open(database: &Database) -> Result<Self, Error> {
        let keyspace = database.keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())?;

        let identifiers = Identifiers::new(keyspace.clone());
        let tags = Tags::new(keyspace.clone());
        let timestamps = Timestamps::new(keyspace);

        Ok(Self::new(identifiers, tags, timestamps))
    }
}

// Contains

impl Indices {
    #[must_use]
    pub fn contains(&self, query: &QueryHash, from: Option<Position>) -> bool {
        self.query(query, from).any(|_| true)
    }
}

// Put

impl Indices {
    #[rustfmt::skip]
    pub fn put(
        &self,
        batch: &mut WriteBatch,
        at: Position,
        event: &EphemeralEventHashRef<'_>,
        timestamp: Timestamp,
    ) {
        self.identifiers.put(batch, at, &event.identifier, event.version);
        self.tags.put(batch, at, &event.tags);
        self.timestamps.put(batch, at, timestamp);
    }
}

// Query

impl Indices {
    #[must_use]
    pub fn query(&self, query: &QueryHash, from: Option<Position>) -> PositionIterator {
        SequentialOrIterator::combine(query.as_ref().iter().map(|selector| match selector {
            SelectorHash::Specifiers(specifiers) => self.identifiers.query(specifiers.iter(), from),
            SelectorHash::SpecifiersAndTags(specifiers, tags) => SequentialAndIterator::combine([
                self.identifiers.query(specifiers.iter(), from),
                self.tags.query(tags.iter(), from),
            ]),
            SelectorHash::Tags(tags) => self.tags.query(tags.iter(), from),
        }))
    }
}

// -------------------------------------------------------------------------------------------------

// Iterators

#[derive(Debug)]
pub enum PositionIterator {
    And(SequentialAndIterator<PositionIterator, Position>),
    Or(SequentialOrIterator<PositionIterator, Position>),
    Identifier(#[debug("Identifier Position Iterator")] IdentifierPositionIterator),
    Tag(#[debug("Tag Position Iterator")] TagPositionIterator),
}

impl DoubleEndedIterator for PositionIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iter) => iter.next_back(),
            Self::Or(iter) => iter.next_back(),
            Self::Identifier(iter) => iter.next_back(),
            Self::Tag(iter) => iter.next_back(),
        }
    }
}

impl Iterator for PositionIterator {
    type Item = Result<Position, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iter) => iter.next(),
            Self::Or(iter) => iter.next(),
            Self::Identifier(iter) => iter.next(),
            Self::Tag(iter) => iter.next(),
        }
    }
}

impl From<SequentialAndIterator<PositionIterator, Position>> for PositionIterator {
    fn from(iter: SequentialAndIterator<PositionIterator, Position>) -> Self {
        Self::And(iter)
    }
}

impl From<SequentialOrIterator<PositionIterator, Position>> for PositionIterator {
    fn from(iter: SequentialOrIterator<PositionIterator, Position>) -> Self {
        Self::Or(iter)
    }
}
