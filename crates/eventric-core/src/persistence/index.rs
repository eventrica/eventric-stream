pub mod descriptor;
pub mod tags;

use std::error::Error;

use derive_more::Debug;
use fjall::{
    Keyspace,
    KeyspaceCreateOptions,
};
use self_cell::self_cell;

use crate::{
    model::Position,
    persistence::{
        Context,
        Write,
        model::HashedEvent,
    },
    utility::iter::{
        SequentialAnd,
        SequentialOr,
    },
};

// =================================================================================================
// Index
// =================================================================================================

static ID_LEN: usize = size_of::<u8>();
static KEYSPACE_NAME: &str = "index";

// -------------------------------------------------------------------------------------------------

// Keyspace

pub fn keyspace(context: &Context) -> Result<Keyspace, Box<dyn Error>> {
    Ok(context
        .as_ref()
        .keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())?)
}

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, position: Position, event: &HashedEvent) {
    descriptor::insert(write, position, &event.descriptor);
    tags::insert(write, position, &event.tags);
}

// -------------------------------------------------------------------------------------------------

// Iterator

// Sequential Iterator

#[derive(Debug)]
pub enum SequentialIterator {
    And(SequentialAnd<SequentialIterator, u64>),
    Or(SequentialOr<SequentialIterator, u64>),
    Owned(#[debug("OwnedPositionIterator")] OwnedSequentialIterator),
}

impl Iterator for SequentialIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iterator) => iterator.next(),
            Self::Or(iterator) => iterator.next(),
            Self::Owned(iterator) => iterator.next(),
        }
    }
}

impl From<SequentialAnd<SequentialIterator, u64>> for SequentialIterator {
    fn from(value: SequentialAnd<SequentialIterator, u64>) -> Self {
        Self::And(value)
    }
}

impl From<SequentialOr<SequentialIterator, u64>> for SequentialIterator {
    fn from(value: SequentialOr<SequentialIterator, u64>) -> Self {
        Self::Or(value)
    }
}

impl From<OwnedSequentialIterator> for SequentialIterator {
    fn from(value: OwnedSequentialIterator) -> Self {
        Self::Owned(value)
    }
}

// Boxed Sequential Iterator

type BoxedSequentialIterator<'a> = Box<dyn Iterator<Item = u64> + 'a>;

// Owned Sequential Iterator

self_cell!(
    pub struct OwnedSequentialIterator {
        owner: Keyspace,
        #[covariant]
        dependent: BoxedSequentialIterator,
    }
);

impl Iterator for OwnedSequentialIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_dependent_mut(|_, iterator| iterator.next())
    }
}
