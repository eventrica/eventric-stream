use bytes::{
    Buf as _,
    BufMut as _,
};
use eventric_core_model::{
    Position,
    TagHash,
    TagHashRef,
};
use eventric_core_state::{
    Read,
    Write,
};
use fjall::{
    Error,
    Guard,
    Keyspace,
    Slice,
};

use crate::{
    iter::{
        OwnedSequentialIterator,
        SequentialIterator,
    },
    operation::{
        ID_LEN,
        POSITION_LEN,
        tags::HASH_LEN,
    },
};

// =================================================================================================
// Forward
// =================================================================================================

static INDEX_ID: u8 = 1;
static KEY_LEN: usize = ID_LEN + HASH_LEN + POSITION_LEN;
static PREFIX_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Insert

pub fn insert(write: &mut Write<'_>, position: Position, tags: &[TagHashRef<'_>]) {
    let mut key = [0u8; KEY_LEN];

    for tag in tags {
        write_key(&mut key, position, tag.hash());

        write.batch.insert(&write.keyspaces.index, key, []);
    }
}

// -------------------------------------------------------------------------------------------------

// Iterate

#[must_use]
pub fn iterate(read: &Read<'_>, position: Option<Position>, tag: &TagHash) -> SequentialIterator {
    let map = |key: Result<Slice, Error>| {
        let key = key.expect("invalid key/value during iteration");

        let mut key = &key[..];

        key.advance(ID_LEN + HASH_LEN);
        key.get_u64()
    };

    let index = read.keyspaces.index.clone();

    match position {
        Some(position) => range(index, position, tag, map),
        None => prefix(index, tag, map),
    }
}

fn prefix<F>(index: Keyspace, tag: &TagHash, map: F) -> SequentialIterator
where
    F: Fn(Result<Slice, Error>) -> u64 + 'static,
{
    let mut prefix = [0u8; PREFIX_LEN];

    write_prefix(&mut prefix, tag.hash());

    OwnedSequentialIterator::new(index, |keyspace| {
        Box::new(keyspace.prefix(prefix).map(Guard::key).map(map))
    })
    .into()
}

fn range<F>(index: Keyspace, position: Position, tag: &TagHash, map: F) -> SequentialIterator
where
    F: Fn(Result<Slice, Error>) -> u64 + 'static,
{
    let mut lower = [0u8; KEY_LEN];

    write_key(&mut lower, position, tag.hash());

    let mut upper = [0u8; KEY_LEN];

    let position = Position::new(u64::MAX);

    write_key(&mut upper, position, tag.hash());

    let range = lower..=upper;

    OwnedSequentialIterator::new(index, |keyspace| {
        Box::new(keyspace.range(range).map(Guard::key).map(map))
    })
    .into()
}

// -------------------------------------------------------------------------------------------------

// Keys/Prefixes

fn write_key(key: &mut [u8; KEY_LEN], position: Position, tag: u64) {
    let mut key = &mut key[..];

    let index_id = INDEX_ID;
    let position = position.value();

    key.put_u8(index_id);
    key.put_u64(tag);
    key.put_u64(position);
}

fn write_prefix(prefix: &mut [u8; PREFIX_LEN], tag: u64) {
    let mut prefix = &mut prefix[..];

    let index_id = INDEX_ID;

    prefix.put_u8(index_id);
    prefix.put_u64(tag);
}
