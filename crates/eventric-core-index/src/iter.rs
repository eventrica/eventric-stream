use derive_more::Debug;
use eventric_core_model::Position;
use eventric_core_util::iter::{
    SequentialAnd,
    SequentialOr,
};
use fjall::Keyspace;
use self_cell::self_cell;

// =================================================================================================
// Iterator
// =================================================================================================

// Sequential Position Iterator

#[derive(Debug)]
pub enum SequentialPositionIterator {
    And(SequentialAnd<SequentialPositionIterator, Position>),
    Or(SequentialOr<SequentialPositionIterator, Position>),
    Owned(#[debug("OwnedSequentialIterator")] OwnedSequentialPositionIterator),
}

impl Iterator for SequentialPositionIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::And(iterator) => iterator.next(),
            Self::Or(iterator) => iterator.next(),
            Self::Owned(iterator) => iterator.next(),
        }
    }
}

impl From<SequentialAnd<SequentialPositionIterator, Position>> for SequentialPositionIterator {
    fn from(value: SequentialAnd<SequentialPositionIterator, Position>) -> Self {
        Self::And(value)
    }
}

impl From<SequentialOr<SequentialPositionIterator, Position>> for SequentialPositionIterator {
    fn from(value: SequentialOr<SequentialPositionIterator, Position>) -> Self {
        Self::Or(value)
    }
}

impl From<OwnedSequentialPositionIterator> for SequentialPositionIterator {
    fn from(value: OwnedSequentialPositionIterator) -> Self {
        Self::Owned(value)
    }
}

// Boxed Sequential Position Iterator

type BoxedSequentialPositionIterator<'a> = Box<dyn Iterator<Item = Position> + 'a>;

// Owned Sequential Position Iterator

self_cell!(
    pub struct OwnedSequentialPositionIterator {
        owner: Keyspace,
        #[covariant]
        dependent: BoxedSequentialPositionIterator,
    }
);

impl Iterator for OwnedSequentialPositionIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_dependent_mut(|_, iterator| iterator.next())
    }
}
