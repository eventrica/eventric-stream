use std::collections::BTreeSet;

use bytes::{
    Buf as _,
    BufMut as _,
};
use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Guard,
    Keyspace,
    OwnedWriteBatch,
    Slice,
};

use crate::{
    error::Error,
    event::{
        position::Position,
        tag::{
            TagHash,
            TagHashAndValue,
        },
    },
    stream::data::{
        HASH_LEN,
        ID_LEN,
        POSITION_LEN,
        indices::PositionIter,
    },
    utils::iteration::and::AndIter,
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

// Iterate

impl Tags {
    pub fn iterate<'a, T>(&self, tags: T, from: Option<Position>) -> PositionIter
    where
        T: Iterator<Item = &'a TagHash>,
    {
        AndIter::combine(tags.map(|tag| {
            let iter = if let Some(from) = from {
                self.keyspace.range(
                    Into::<KeyBytes>::into(IntoKeyBytes(from, *tag))
                        ..Into::<KeyBytes>::into(IntoKeyBytes(Position::MAX, *tag)),
                )
            } else {
                self.keyspace
                    .prefix(Into::<PrefixBytes>::into(IntoPrefixBytes(*tag)))
            };

            PositionIter::Tags(Iter::new(iter))
        }))
    }
}

// Put

impl Tags {
    pub fn put(&self, batch: &mut OwnedWriteBatch, at: Position, tags: &BTreeSet<TagHashAndValue>) {
        for tag in tags {
            let key: KeyBytes = IntoKeyBytes(at, tag.0).into();
            let value = [];

            batch.insert(&self.keyspace, key, value);
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Iterator

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct Iter {
    #[debug("Iter")]
    iter: fjall::Iter,
}

impl Iter {
    #[rustfmt::skip]
    fn next_map(guard: Guard) -> <Self as Iterator>::Item {
        match guard.key() {
            Ok(key) => Ok(IntoPosition(key).into()),
            Err(err) => Err(Error::from(err)),
        }
    }
}

impl DoubleEndedIterator for Iter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Self::next_map)
    }
}

impl Iterator for Iter {
    type Item = Result<Position, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::next_map)
    }
}

impl Iter {}

// -------------------------------------------------------------------------------------------------

// Conversions

// Hash -> Prefix Byte Array

type PrefixBytes = [u8; PREFIX_LEN];

struct IntoPrefixBytes(TagHash);

impl From<IntoPrefixBytes> for PrefixBytes {
    fn from(IntoPrefixBytes(tag): IntoPrefixBytes) -> Self {
        let mut prefix = [0u8; PREFIX_LEN];

        {
            let mut prefix = &mut prefix[..];

            prefix.put_u8(INDEX_ID);
            prefix.put_u64(tag.0);
        }

        prefix
    }
}

// Key (Slice) -> Position

struct IntoPosition(Slice);

impl From<IntoPosition> for Position {
    fn from(IntoPosition(slice): IntoPosition) -> Self {
        let mut slice = &slice[..];

        slice.advance(ID_LEN + HASH_LEN);

        Position::new(slice.get_u64())
    }
}

// Position & Hash -> Key Byte Array

type KeyBytes = [u8; KEY_LEN];

struct IntoKeyBytes(Position, TagHash);

impl From<IntoKeyBytes> for KeyBytes {
    fn from(IntoKeyBytes(position, tag): IntoKeyBytes) -> Self {
        let mut key = [0u8; KEY_LEN];

        {
            let mut key = &mut key[..];

            key.put_u8(INDEX_ID);
            key.put_u64(tag.0);
            key.put_u64(*position);
        }

        key
    }
}
