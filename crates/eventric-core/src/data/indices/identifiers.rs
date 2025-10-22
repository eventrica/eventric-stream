use bytes::{
    Buf,
    BufMut as _,
};
use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Error,
    Guard,
    Keyspace,
    Slice,
    WriteBatch,
};

use crate::{
    data::{
        HASH_LEN,
        ID_LEN,
        POSITION_LEN,
        indices::{
            OwnedSequentialIterator,
            SequentialIterator,
        },
    },
    model::{
        event::{
            identifier::IdentifierHashRef,
            version::Version,
        },
        query::specifier::SpecifierHash,
        stream::position::Position,
    },
    util::iter::SequentialOrIterator,
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

#[derive(new, Debug)]
#[new(const_fn)]
pub struct Identifiers {
    #[debug("Keyspace(\"{}\")", keyspace.name)]
    keyspace: Keyspace,
}

// Put

impl Identifiers {
    pub fn put(
        &self,
        batch: &mut WriteBatch,
        position: Position,
        identifier: &IdentifierHashRef<'_>,
        version: Version,
    ) {
        let key: [u8; KEY_LEN] = PositionAndHash(position, identifier.hash()).into();
        let value = version.value().to_be_bytes();

        batch.insert(&self.keyspace, key, value);
    }
}

// Query

impl Identifiers {
    pub fn query<'a, S>(&self, specifiers: S, position: Option<Position>) -> SequentialIterator
    where
        S: Iterator<Item = &'a SpecifierHash>,
    {
        SequentialOrIterator::combine(specifiers.map(|specifier| self.iterate(specifier, position)))
    }

    fn iterate(&self, specifier: &SpecifierHash, position: Option<Position>) -> SequentialIterator {
        let version_range = specifier
            .range()
            .as_ref()
            .map_or(u8::MIN..u8::MAX, |r| r.start.value()..r.end.value());

        let predicate = move |key_value: Result<(Slice, Slice), Error>| {
            let (key, value) = key_value.expect("iteration key/value error");

            if !version_range.contains(&value.as_ref().get_u8()) {
                return None;
            }

            let mut key = &key[..];

            key.advance(ID_LEN + HASH_LEN);

            Some(Position::new(key.get_u64()))
        };

        let iterator = match position {
            Some(position) => self.iterate_range(specifier, position, predicate),
            None => self.iterate_prefix(specifier, predicate),
        };

        iterator.into()
    }

    fn iterate_prefix<P>(&self, specifier: &SpecifierHash, predicate: P) -> OwnedSequentialIterator
    where
        P: Fn(Result<(Slice, Slice), Error>) -> Option<Position> + 'static,
    {
        let hash = specifier.identifer().hash();
        let prefix: [u8; PREFIX_LEN] = Hash(hash).into();

        OwnedSequentialIterator::new(self.keyspace.clone(), |keyspace| {
            Box::new(
                keyspace
                    .prefix(prefix)
                    .map(Guard::into_inner)
                    .filter_map(predicate),
            )
        })
    }

    fn iterate_range<P>(
        &self,
        specifier: &SpecifierHash,
        position: Position,
        predicate: P,
    ) -> OwnedSequentialIterator
    where
        P: Fn(Result<(Slice, Slice), Error>) -> Option<Position> + 'static,
    {
        let hash = specifier.identifer().hash();
        let lower: [u8; KEY_LEN] = PositionAndHash(position, hash).into();
        let upper: [u8; KEY_LEN] = PositionAndHash(Position::new(u64::MAX), hash).into();

        OwnedSequentialIterator::new(self.keyspace.clone(), |keyspace| {
            Box::new(
                keyspace
                    .range(lower..upper)
                    .map(Guard::into_inner)
                    .filter_map(predicate),
            )
        })
    }
}

// -------------------------------------------------------------------------------------------------

// Conversions

struct PositionAndHash(Position, u64);

impl From<PositionAndHash> for [u8; KEY_LEN] {
    fn from(PositionAndHash(position, hash): PositionAndHash) -> Self {
        let mut key = [0u8; KEY_LEN];

        {
            let mut key = &mut key[..];

            key.put_u8(INDEX_ID);
            key.put_u64(hash);
            key.put_u64(position.value());
        }

        key
    }
}

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
