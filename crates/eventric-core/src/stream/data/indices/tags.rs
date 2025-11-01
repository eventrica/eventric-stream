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
    pub fn query<'a, T>(&self, tags: T, from: Option<Position>) -> PositionIterator<'_>
    where
        T: Iterator<Item = &'a TagHash>,
    {
        SequentialAndIterator::combine(tags.map(|tag| self.query_tag(tag, from)))
    }

    fn query_tag(&self, tag: &TagHash, from: Option<Position>) -> PositionIterator<'_> {
        match from {
            Some(position) => self.query_tag_range(tag, position),
            None => self.query_tag_prefix(tag),
        }
    }

    fn query_tag_prefix(&self, tag: &TagHash) -> PositionIterator<'_> {
        let hash = tag.hash();
        let prefix: [u8; PREFIX_LEN] = Hash(hash).into();

        let iter = Box::new(self.keyspace.prefix(prefix).map(Guard::key));
        let iter = TagPositionIterator::new(iter);

        PositionIterator::Tag(iter)
    }

    fn query_tag_range(&self, tag: &TagHash, from: Position) -> PositionIterator<'_> {
        let hash = tag.hash();
        let lower: [u8; KEY_LEN] = PositionAndHash(from, hash).into();
        let upper: [u8; KEY_LEN] = PositionAndHash(Position::MAX, hash).into();

        let iter = Box::new(self.keyspace.range(lower..upper).map(Guard::key));
        let iter = TagPositionIterator::new(iter);

        PositionIterator::Tag(iter)
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

// -------------------------------------------------------------------------------------------------

// Iterators

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub(crate) struct TagPositionIterator<'a> {
    #[debug("BoxedIterator")]
    iter: Box<dyn DoubleEndedIterator<Item = Result<Slice, fjall::Error>> + 'a>,
}

impl TagPositionIterator<'_> {
    fn map(key: &Slice) -> Position {
        let mut key = &key[..];

        key.advance(ID_LEN + HASH_LEN);

        Position::new(key.get_u64())
    }
}

impl DoubleEndedIterator for TagPositionIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.iter.next()? {
            Ok(key) => Some(Ok(Self::map(&key))),
            Err(err) => Some(Err(Error::from(err))),
        }
    }
}

impl Iterator for TagPositionIterator<'_> {
    type Item = Result<Position, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next()? {
            Ok(key) => Some(Ok(Self::map(&key))),
            Err(err) => Some(Err(Error::from(err))),
        }
    }
}
