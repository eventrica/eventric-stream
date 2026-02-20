pub(crate) mod identifiers;
pub(crate) mod tags;
pub(crate) mod timestamps;

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
        position::Position,
        timestamp::Timestamp,
    },
    stream::{
        data::indices::{
            identifiers::Identifiers,
            tags::Tags,
            timestamps::Timestamps,
        },
        select::{
            SelectionHash,
            selector::SelectorHash,
        },
    },
    utils::iteration::{
        and::AndIter,
        or::OrIter,
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
        let keyspace = database.keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default)?;

        let identifiers = Identifiers::new(keyspace.clone());
        let tags = Tags::new(keyspace.clone());
        let timestamps = Timestamps::new(keyspace);

        Ok(Self::new(identifiers, tags, timestamps))
    }
}

// Contains

impl Indices {
    #[must_use]
    pub fn contains(&self, selection: &SelectionHash, from: Option<Position>) -> bool {
        self.iterate(selection, from).any(|_| true)
    }
}

// Iterate

impl Indices {
    #[must_use]
    pub fn iterate(&self, selection: &SelectionHash, from: Option<Position>) -> PositionIter {
        OrIter::combine(selection.as_ref().iter().map(|selector| match selector {
            SelectorHash::Specifiers(specifiers) => {
                self.identifiers.iterate(specifiers.iter(), from)
            }
            SelectorHash::SpecifiersAndTags(specifiers, tags) => AndIter::combine([
                self.identifiers.iterate(specifiers.iter(), from),
                self.tags.iterate(tags.iter(), from),
            ]),
        }))
    }
}

// Put

impl Indices {
    #[rustfmt::skip]
    pub fn put(
        &self,
        batch: &mut OwnedWriteBatch,
        at: Position,
        event: &CandidateEventHashAndValue,
        timestamp: Timestamp,
    ) {
        self.identifiers.put(batch, at, &event.identifier_hash_and_value, event.version);
        self.tags.put(batch, at, &event.tags);
        self.timestamps.put(batch, at, timestamp);
    }
}

// -------------------------------------------------------------------------------------------------

// Iterators

#[derive(Debug)]
pub enum PositionIter {
    And(AndIter<PositionIter, Position>),
    Or(OrIter<PositionIter, Position>),
    Identifiers(identifiers::Iter),
    Tags(tags::Iter),
}

impl DoubleEndedIterator for PositionIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iter) => iter.next_back(),
            Self::Or(iter) => iter.next_back(),
            Self::Identifiers(iter) => iter.next_back(),
            Self::Tags(iter) => iter.next_back(),
        }
    }
}

impl Iterator for PositionIter {
    type Item = Result<Position, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iter) => iter.next(),
            Self::Or(iter) => iter.next(),
            Self::Identifiers(iter) => iter.next(),
            Self::Tags(iter) => iter.next(),
        }
    }
}

impl From<AndIter<PositionIter, Position>> for PositionIter {
    fn from(iter: AndIter<PositionIter, Position>) -> Self {
        Self::And(iter)
    }
}

impl From<OrIter<PositionIter, Position>> for PositionIter {
    fn from(iter: OrIter<PositionIter, Position>) -> Self {
        Self::Or(iter)
    }
}
