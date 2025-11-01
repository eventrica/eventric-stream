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
    pub fn query(
        &self,
        query: &QueryHash,
        from: Option<Position>,
    ) -> SequentialPositionIterator<'_> {
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
pub enum SequentialPositionIterator<'a> {
    And(SequentialAndIterator<SequentialPositionIterator<'a>, Position>),
    Or(SequentialOrIterator<SequentialPositionIterator<'a>, Position>),
    Identifier(IdentifierPositionIterator<'a>),
    Boxed(#[debug("Boxed")] Box<dyn DoubleEndedIterator<Item = Result<Position, Error>> + 'a>),
}

impl DoubleEndedIterator for SequentialPositionIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iter) => iter.next_back(),
            Self::Or(iter) => iter.next_back(),
            Self::Identifier(iter) => iter.next_back(),
            Self::Boxed(iter) => iter.next_back(),
        }
    }
}

impl Iterator for SequentialPositionIterator<'_> {
    type Item = Result<Position, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iter) => iter.next(),
            Self::Or(iter) => iter.next(),
            Self::Identifier(iter) => iter.next(),
            Self::Boxed(iter) => iter.next(),
        }
    }
}

impl<'a> From<SequentialAndIterator<SequentialPositionIterator<'a>, Position>>
    for SequentialPositionIterator<'a>
{
    fn from(iter: SequentialAndIterator<SequentialPositionIterator<'a>, Position>) -> Self {
        Self::And(iter)
    }
}

impl<'a> From<SequentialOrIterator<SequentialPositionIterator<'a>, Position>>
    for SequentialPositionIterator<'a>
{
    fn from(iter: SequentialOrIterator<SequentialPositionIterator<'a>, Position>) -> Self {
        Self::Or(iter)
    }
}
