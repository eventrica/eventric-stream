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
            identifiers::Identifiers,
            tags::Tags,
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
    pub fn query(&self, query: &QueryHash, from: Option<Position>) -> PositionIterator<'_> {
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

// Position Iterator

#[derive(Debug)]
#[rustfmt::skip]
pub enum PositionIterator<'a> {
    And(SequentialAndIterator<PositionIterator<'a>, Position>),
    Or(SequentialOrIterator<PositionIterator<'a>, Position>),
    Boxed(#[debug("Boxed Position Iterator")] Box<dyn DoubleEndedIterator<Item = Result<Position, Error>> + 'a>),
}

impl Iterator for PositionIterator<'_> {
    type Item = Result<Position, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iter) => iter.next(),
            Self::Or(iter) => iter.next(),
            Self::Boxed(iter) => iter.next(),
        }
    }
}

impl DoubleEndedIterator for PositionIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iter) => iter.next_back(),
            Self::Or(iter) => iter.next_back(),
            Self::Boxed(iter) => iter.next_back(),
        }
    }
}

impl<'a> From<SequentialAndIterator<PositionIterator<'a>, Position>> for PositionIterator<'a> {
    fn from(iter: SequentialAndIterator<PositionIterator<'a>, Position>) -> Self {
        Self::And(iter)
    }
}

impl<'a> From<SequentialOrIterator<PositionIterator<'a>, Position>> for PositionIterator<'a> {
    fn from(iter: SequentialOrIterator<PositionIterator<'a>, Position>) -> Self {
        Self::Or(iter)
    }
}

#[rustfmt::skip]
impl<'a> From<Box<dyn DoubleEndedIterator<Item = Result<Position, Error>> + 'a>> for PositionIterator<'a> {
    fn from(iter: Box<dyn DoubleEndedIterator<Item = Result<Position, Error>> + 'a>) -> Self {
        Self::Boxed(iter)
    }
}
