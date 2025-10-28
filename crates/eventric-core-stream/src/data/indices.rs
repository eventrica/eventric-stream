mod identifiers;
mod tags;
mod timestamps;

use derive_more::Debug;
use eventric_core_error::Error;
use eventric_core_event::{
    NewEventHashRef,
    position::Position,
    timestamp::Timestamp,
};
use eventric_core_utils::iteration::{
    and::SequentialAndIterator,
    or::SequentialOrIterator,
};
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
    query::{
        QueryHash,
        QueryItemHash,
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
        event: &NewEventHashRef<'_>,
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
    pub fn query(&self, query: &QueryHash, from: Option<Position>) -> SequentialIterator<'_> {
        SequentialOrIterator::combine(query.as_ref().iter().map(|item| match item {
            QueryItemHash::Specifiers(specifiers) => {
                self.identifiers.query(specifiers.iter(), from)
            }
            QueryItemHash::SpecifiersAndTags(specifiers, tags) => SequentialAndIterator::combine([
                self.identifiers.query(specifiers.iter(), from),
                self.tags.query(tags.iter(), from),
            ]),
            QueryItemHash::Tags(tags) => self.tags.query(tags.iter(), from),
        }))
    }
}

// -------------------------------------------------------------------------------------------------

// Iterators

// Sequential Iterator

#[derive(Debug)]
#[rustfmt::skip]
pub enum SequentialIterator<'a> {
    And(SequentialAndIterator<SequentialIterator<'a>, Position>),
    Or(SequentialOrIterator<SequentialIterator<'a>, Position>),
    Boxed(#[debug("BoxedIterator")] Box<dyn Iterator<Item = Result<Position, Error>> + 'a>),
}

impl Iterator for SequentialIterator<'_> {
    type Item = Result<Position, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iterator) => iterator.next(),
            Self::Or(iterator) => iterator.next(),
            Self::Boxed(iterator) => iterator.next(),
        }
    }
}

impl<'a> From<SequentialAndIterator<SequentialIterator<'a>, Position>> for SequentialIterator<'a> {
    fn from(iter: SequentialAndIterator<SequentialIterator<'a>, Position>) -> Self {
        Self::And(iter)
    }
}

impl<'a> From<SequentialOrIterator<SequentialIterator<'a>, Position>> for SequentialIterator<'a> {
    fn from(iter: SequentialOrIterator<SequentialIterator<'a>, Position>) -> Self {
        Self::Or(iter)
    }
}

impl<'a> From<Box<dyn Iterator<Item = Result<Position, Error>> + 'a>> for SequentialIterator<'a> {
    fn from(iter: Box<dyn Iterator<Item = Result<Position, Error>> + 'a>) -> Self {
        Self::Boxed(iter)
    }
}
