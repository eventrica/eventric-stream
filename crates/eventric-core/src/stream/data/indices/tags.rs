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
use self_cell::self_cell;

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
    pub fn query<'a, T>(&self, tags: T, from: Option<Position>) -> PositionIterator
    where
        T: Iterator<Item = &'a TagHash>,
    {
        SequentialAndIterator::combine(tags.map(|tag| self.query_tag(tag, from)))
    }

    fn query_tag(&self, tag: &TagHash, from: Option<Position>) -> PositionIterator {
        match from {
            Some(position) => self.query_tag_range(tag, position),
            None => self.query_tag_prefix(tag),
        }
    }

    fn query_tag_prefix(&self, tag: &TagHash) -> PositionIterator {
        let hash = tag.hash();
        let prefix: [u8; PREFIX_LEN] = Hash(hash).into();

        let iter = TagPositionIterator::new(self.keyspace.clone(), |keyspace| {
            Box::new(
                keyspace
                    .prefix(prefix)
                    .map(Guard::key)
                    .map(TagPositionIterator::map),
            )
        });

        PositionIterator::Tag(iter)
    }

    fn query_tag_range(&self, tag: &TagHash, from: Position) -> PositionIterator {
        let hash = tag.hash();
        let lower: [u8; KEY_LEN] = PositionAndHash(from, hash).into();
        let upper: [u8; KEY_LEN] = PositionAndHash(Position::MAX, hash).into();

        let iter = TagPositionIterator::new(self.keyspace.clone(), |keyspace| {
            Box::new(
                keyspace
                    .range(lower..upper)
                    .map(Guard::key)
                    .map(TagPositionIterator::map),
            )
        });

        PositionIterator::Tag(iter)
    }
}

// -------------------------------------------------------------------------------------------------

// Iterators

#[rustfmt::skip]
type BoxedTagPositionIterator<'a> = Box<dyn DoubleEndedIterator<Item = Result<Position, Error>> + Send + 'a>;

self_cell!(
    pub(crate) struct TagPositionIterator {
        owner: Keyspace,
        #[covariant]
        dependent: BoxedTagPositionIterator,
    }
);

impl TagPositionIterator {
    fn map(result: Result<Slice, fjall::Error>) -> <Self as Iterator>::Item {
        match result {
            Ok(key) => Ok(Key(key).into()),
            Err(err) => Err(Error::from(err)),
        }
    }
}

impl DoubleEndedIterator for TagPositionIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.with_dependent_mut(|_, iter| iter.next_back())
    }
}

impl Iterator for TagPositionIterator {
    type Item = Result<Position, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_dependent_mut(|_, iter| iter.next())
    }
}

#[allow(unsafe_code)]
unsafe impl Sync for TagPositionIterator {}

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
