use bytes::{
    Buf as _,
    BufMut as _,
};
use derive_more::Debug;
use eventric_core_error::Error;
use eventric_core_event::{
    position::Position,
    tag::{
        TagHash,
        TagHashRef,
    },
};
use eventric_core_utils::iteration::and::SequentialAndIterator;
use fancy_constructor::new;
use fjall::{
    Guard,
    Keyspace,
    Slice,
    WriteBatch,
};

use crate::data::{
    HASH_LEN,
    ID_LEN,
    POSITION_LEN,
    indices::SequentialIterator,
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
pub struct Tags {
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
    pub fn query<'a, T>(&self, tags: T, from: Option<Position>) -> SequentialIterator<'_>
    where
        T: Iterator<Item = &'a TagHash>,
    {
        SequentialAndIterator::combine(tags.map(|tag| self.query_tag(tag, from)))
    }

    fn query_tag(&self, tag: &TagHash, from: Option<Position>) -> SequentialIterator<'_> {
        let map = |key: Result<Slice, fjall::Error>| match key {
            Ok(key) => {
                let mut key = &key[..];

                key.advance(ID_LEN + HASH_LEN);

                Ok(Position::new(key.get_u64()))
            }
            Err(err) => Err(Error::from(err)),
        };

        match from {
            Some(position) => self.query_tag_range(tag, position, map),
            None => self.query_tag_prefix(tag, map),
        }
    }

    fn query_tag_prefix<F>(&self, tag: &TagHash, f: F) -> SequentialIterator<'_>
    where
        F: Fn(Result<Slice, fjall::Error>) -> Result<Position, Error> + 'static,
    {
        let hash = tag.hash();
        let prefix: [u8; PREFIX_LEN] = Hash(hash).into();

        SequentialIterator::Boxed(Box::new(
            self.keyspace.prefix(prefix).map(Guard::key).map(f),
        ))
    }

    fn query_tag_range<F>(&self, tag: &TagHash, from: Position, f: F) -> SequentialIterator<'_>
    where
        F: Fn(Result<Slice, fjall::Error>) -> Result<Position, Error> + 'static,
    {
        let hash = tag.hash();
        let lower: [u8; KEY_LEN] = PositionAndHash(from, hash).into();
        let upper: [u8; KEY_LEN] = PositionAndHash(Position::MAX, hash).into();

        SequentialIterator::Boxed(Box::new(
            self.keyspace.range(lower..upper).map(Guard::key).map(f),
        ))
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
