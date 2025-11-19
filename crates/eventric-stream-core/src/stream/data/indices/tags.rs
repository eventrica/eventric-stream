use bytes::{
    Buf as _,
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
        position::Position,
        tag::{
            TagHash,
            TagHashRef,
        },
    },
    stream::data::{
        BoxedIterator,
        HASH_LEN,
        ID_LEN,
        POSITION_LEN,
        indices::PositionIterator,
    },
    utils::iteration::and::SequentialAndIterator,
};

// =================================================================================================
// Tags
// =================================================================================================

// Configuration

static INDEX_ID: u8 = 1;

static KEY_LEN: usize = ID_LEN + HASH_LEN + POSITION_LEN;
static PREFIX_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Tags

#[derive(new, Clone, Debug)]
#[new(const_fn)]
pub(crate) struct Tags {
    #[debug("Keyspace(\"{}\")", keyspace.name)]
    keyspace: Keyspace,
}

// Put

impl Tags {
    pub fn put(&self, batch: &mut WriteBatch, at: Position, tags: &[TagHashRef<'_>]) {
        for tag in tags {
            let key: [u8; KEY_LEN] = PositionAndHash(at, tag.hash()).into();
            let value = [];

            batch.insert(&self.keyspace, key, value);
        }
    }
}

// Query

impl Tags {
    pub fn query<'a, T>(&self, tags: T, from: Option<Position>) -> PositionIterator
    where
        T: Iterator<Item = &'a TagHash>,
    {
        SequentialAndIterator::combine(tags.map(|tag| self.query_tag(tag, from)))
    }

    fn query_tag(&self, tag: &TagHash, from: Option<Position>) -> PositionIterator {
        match from {
            Some(from) => PositionIterator::Iterator(self.query_range(tag, from)),
            None => PositionIterator::Iterator(self.query_prefix(tag)),
        }
    }

    fn query_prefix(&self, tag: &TagHash) -> BoxedIterator<Position> {
        let hash = tag.hash();
        let prefix: [u8; PREFIX_LEN] = Hash(hash).into();

        Box::new(
            self.keyspace
                .prefix(prefix)
                .map(Guard::key)
                .map(Self::query_map),
        )
    }

    fn query_range(&self, tag: &TagHash, from: Position) -> BoxedIterator<Position> {
        let hash = tag.hash();
        let lower: [u8; KEY_LEN] = PositionAndHash(from, hash).into();
        let upper: [u8; KEY_LEN] = PositionAndHash(Position::MAX, hash).into();

        Box::new(
            self.keyspace
                .range(lower..upper)
                .map(Guard::key)
                .map(Self::query_map),
        )
    }

    fn query_map(result: Result<Slice, fjall::Error>) -> Result<Position, Error> {
        match result {
            Ok(key) => Ok(Key(key).into()),
            Err(err) => Err(Error::from(err)),
        }
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
