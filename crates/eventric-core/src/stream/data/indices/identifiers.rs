use std::sync::Arc;

use any_range::AnyRange;
use bytes::{
    Buf,
    BufMut as _,
};
use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Guard,
    Keyspace,
    Slice,
    WriteBatch,
};

use crate::{
    error::Error,
    event::{
        identifier::{
            IdentifierHash,
            IdentifierHashRef,
        },
        position::Position,
        specifier::SpecifierHash,
        version::Version,
    },
    stream::data::{
        HASH_LEN,
        ID_LEN,
        POSITION_LEN,
        indices::PositionIterator,
    },
    utils::iteration::or::SequentialOrIterator,
};

// =================================================================================================
// Identifiers
// =================================================================================================

// Configuration

static INDEX_ID: u8 = 0;

static KEY_LEN: usize = ID_LEN + HASH_LEN + POSITION_LEN;
static PREFIX_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Identifiers

#[derive(new, Clone, Debug)]
#[new(const_fn)]
pub(crate) struct Identifiers {
    #[debug("Keyspace(\"{}\")", keyspace.name)]
    keyspace: Keyspace,
}

// Put

impl Identifiers {
    pub fn put(
        &self,
        batch: &mut WriteBatch,
        at: Position,
        identifier: &IdentifierHashRef<'_>,
        version: Version,
    ) {
        let key: [u8; KEY_LEN] = PositionAndHash(at, identifier.hash()).into();
        let value = version.to_be_bytes();

        batch.insert(&self.keyspace, key, value);
    }
}

// Query

impl Identifiers {
    pub fn query<'a, S>(&self, specifiers: S, from: Option<Position>) -> PositionIterator<'_>
    where
        S: Iterator<Item = &'a SpecifierHash>,
    {
        SequentialOrIterator::combine(
            specifiers.map(|specifier| self.query_specifier(specifier, from)),
        )
    }

    fn query_specifier(
        &self,
        specifier: &SpecifierHash,
        from: Option<Position>,
    ) -> PositionIterator<'_> {
        let range = specifier.range.clone();

        match from {
            Some(position) => self.query_specifier_range(&specifier.identifier, position, range),
            None => self.query_specifier_prefix(&specifier.identifier, range),
        }
    }

    fn query_specifier_prefix(
        &self,
        identifier: &IdentifierHash,
        range: Option<AnyRange<Version>>,
    ) -> PositionIterator<'_> {
        let hash = identifier.hash();
        let prefix: [u8; PREFIX_LEN] = Hash(hash).into();

        let iter = self.keyspace.prefix(prefix).map(Guard::into_inner);
        let iter = IdentifierPositionIterator::new(iter, range);

        PositionIterator::Identifier(iter)
    }

    fn query_specifier_range(
        &self,
        identifier: &IdentifierHash,
        from: Position,
        range: Option<AnyRange<Version>>,
    ) -> PositionIterator<'_> {
        let hash = identifier.hash();
        let lower: [u8; KEY_LEN] = PositionAndHash(from, hash).into();
        let upper: [u8; KEY_LEN] = PositionAndHash(Position::MAX, hash).into();

        let iter = self.keyspace.range(lower..upper).map(Guard::into_inner);
        let iter = IdentifierPositionIterator::new(iter, range);

        PositionIterator::Identifier(iter)
    }
}

// -------------------------------------------------------------------------------------------------

// Iterators

#[derive(new, Debug)]
#[new(const_fn, name(new_inner), vis())]
pub(crate) struct IdentifierPositionIterator<'a> {
    #[debug("Boxed Iterator")]
    iter: Box<dyn DoubleEndedIterator<Item = Result<Position, Error>> + 'a>,
}

impl<'a> IdentifierPositionIterator<'a> {
    fn new<I>(iter: I, range: Option<AnyRange<Version>>) -> Self
    where
        I: DoubleEndedIterator<Item = Result<(Slice, Slice), fjall::Error>> + 'a,
    {
        let range = Arc::new(range);
        let iter = iter.filter_map(move |result| Self::filter_map(result, range.as_ref().as_ref()));
        let iter = Box::new(iter);

        Self::new_inner(iter)
    }
}

impl IdentifierPositionIterator<'_> {
    fn filter_map(
        result: Result<(Slice, Slice), fjall::Error>,
        range: Option<&AnyRange<Version>>,
    ) -> Option<<Self as Iterator>::Item> {
        match result {
            Ok((key, value)) => {
                if let Some(range) = range {
                    let version = Value(value).into();

                    if !range.contains(&version) {
                        return None;
                    }
                }

                Some(Ok(Key(key).into()))
            }
            Err(err) => Some(Err(Error::from(err))),
        }
    }
}

impl DoubleEndedIterator for IdentifierPositionIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl Iterator for IdentifierPositionIterator<'_> {
    type Item = Result<Position, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// -------------------------------------------------------------------------------------------------

// Conversions

// Hash -> Prefix Byte Array

struct Hash(u64);

impl From<Hash> for [u8; PREFIX_LEN] {
    fn from(Hash(hash): Hash) -> Self {
        let mut prefix = [0u8; PREFIX_LEN];

        {
            let mut prefix = &mut prefix[..];

            prefix.put_u8(INDEX_ID);
            prefix.put_u64(hash);
        }

        prefix
    }
}

// Key (Slice) -> Position

struct Key(Slice);

impl From<Key> for Position {
    fn from(Key(slice): Key) -> Self {
        let mut slice = &slice[..];

        slice.advance(ID_LEN + HASH_LEN);

        Position::new(slice.get_u64())
    }
}

// Position & Hash -> Key Byte Array

struct PositionAndHash(Position, u64);

impl From<PositionAndHash> for [u8; KEY_LEN] {
    fn from(PositionAndHash(position, hash): PositionAndHash) -> Self {
        let mut key = [0u8; KEY_LEN];

        {
            let mut key = &mut key[..];

            key.put_u8(INDEX_ID);
            key.put_u64(hash);
            key.put_u64(*position);
        }

        key
    }
}

// Value (Slice) -> Version

struct Value(Slice);

impl From<Value> for Version {
    fn from(Value(slice): Value) -> Self {
        Version::new(slice.as_ref().get_u8())
    }
}
