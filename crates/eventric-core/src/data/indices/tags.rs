use bytes::{
    Buf as _,
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
        indices::SequentialIterator,
    },
    model::{
        event::tag::{
            TagHash,
            TagHashRef,
        },
        stream::position::Position,
    },
    util::iter::SequentialAndIterator,
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
    pub fn put(&self, batch: &mut WriteBatch, position: Position, tags: &[TagHashRef<'_>]) {
        for tag in tags {
            let key: [u8; KEY_LEN] = PositionAndHash(position, tag.hash()).into();
            let value = [];

            batch.insert(&self.keyspace, key, value);
        }
    }
}

// Query

impl Tags {
    pub fn query<'a, T>(&self, tags: T, position: Option<Position>) -> SequentialIterator<'_>
    where
        T: Iterator<Item = &'a TagHash>,
    {
        SequentialAndIterator::combine(tags.map(|tag| self.query_tag(tag, position)))
    }

    fn query_tag(&self, tag: &TagHash, position: Option<Position>) -> SequentialIterator<'_> {
        let map = |key: Result<Slice, Error>| {
            let key = key.expect("iteration key error");

            let mut key = &key[..];

            key.advance(ID_LEN + HASH_LEN);

            Position::new(key.get_u64())
        };

        match position {
            Some(position) => self.query_tag_range(tag, position, map),
            None => self.query_tag_prefix(tag, map),
        }
    }

    fn query_tag_prefix<M>(&self, tag: &TagHash, map: M) -> SequentialIterator<'_>
    where
        M: Fn(Result<Slice, Error>) -> Position + 'static,
    {
        let hash = tag.hash();
        let prefix: [u8; PREFIX_LEN] = Hash(hash).into();

        SequentialIterator::Owned(Box::new(
            self.keyspace.prefix(prefix).map(Guard::key).map(map),
        ))
    }

    fn query_tag_range<M>(
        &self,
        tag: &TagHash,
        position: Position,
        map: M,
    ) -> SequentialIterator<'_>
    where
        M: Fn(Result<Slice, Error>) -> Position + 'static,
    {
        let hash = tag.hash();
        let lower: [u8; KEY_LEN] = PositionAndHash(position, hash).into();
        let upper: [u8; KEY_LEN] = PositionAndHash(Position::MAX, hash).into();

        SequentialIterator::Owned(Box::new(
            self.keyspace.range(lower..upper).map(Guard::key).map(map),
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
