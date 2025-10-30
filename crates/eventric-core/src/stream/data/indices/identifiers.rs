use bytes::{
    Buf,
    BufMut as _,
};
use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
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
        let version_range = specifier
            .range
            .as_ref()
            .map_or(u8::MIN..u8::MAX, |r| *r.start..*r.end);

        let f = move |key: Slice, value: Slice| {
            if !version_range.contains(&value.as_ref().get_u8()) {
                return None;
            }

            let mut key = &key[..];

            key.advance(ID_LEN + HASH_LEN);

            Some(Position::new(key.get_u64()))
        };

        match from {
            Some(position) => self.query_specifier_range(&specifier.identifier, position, f),
            None => self.query_specifier_prefix(&specifier.identifier, f),
        }
    }

    fn query_specifier_prefix<F>(&self, identifier: &IdentifierHash, f: F) -> PositionIterator<'_>
    where
        F: Fn(Slice, Slice) -> Option<Position> + 'static,
    {
        let hash = identifier.hash();
        let prefix: [u8; PREFIX_LEN] = Hash(hash).into();

        PositionIterator::Boxed(Box::new(self.keyspace.prefix(prefix).filter_map(
            move |guard| match guard.into_inner() {
                Ok((key, value)) => f(key, value).map(Ok),
                Err(err) => Some(Err(Error::from(err))),
            },
        )))
    }

    fn query_specifier_range<F>(
        &self,
        identifier: &IdentifierHash,
        from: Position,
        f: F,
    ) -> PositionIterator<'_>
    where
        F: Fn(Slice, Slice) -> Option<Position> + 'static,
    {
        let hash = identifier.hash();
        let lower: [u8; KEY_LEN] = PositionAndHash(from, hash).into();
        let upper: [u8; KEY_LEN] = PositionAndHash(Position::MAX, hash).into();

        PositionIterator::Boxed(Box::new(self.keyspace.range(lower..upper).filter_map(
            move |guard| match guard.into_inner() {
                Ok((key, value)) => f(key, value).map(Ok),
                Err(err) => Some(Err(Error::from(err))),
            },
        )))
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
            key.put_u64(*position);
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
