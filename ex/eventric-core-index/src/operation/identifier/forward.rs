use bytes::{
    Buf as _,
    BufMut as _,
};
use eventric_core_model::{
    IdentifierHashRef,
    Position,
    SpecifierHash,
    Version,
};
use fjall::{
    Error,
    Guard,
    Keyspace,
    Slice,
    WriteBatch,
};

use crate::{
    iter::{
        OwnedSequentialPositionIterator,
        SequentialPositionIterator,
    },
    operation::{
        ID_LEN,
        POSITION_LEN,
        identifier::HASH_LEN,
    },
};

// =================================================================================================
// Forward
// =================================================================================================

// Configuration

static INDEX_ID: u8 = 0;
static KEY_LEN: usize = ID_LEN + HASH_LEN + POSITION_LEN;
static PREFIX_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

//  Insert

pub fn insert(
    batch: &mut WriteBatch,
    index: &Keyspace,
    position: Position,
    identifier: &IdentifierHashRef<'_>,
    version: Version,
) {
    let mut key = [0u8; KEY_LEN];

    write_key(&mut key, position, identifier.hash());

    let value = version.value().to_be_bytes();

    batch.insert(index, key, value);
}

// -------------------------------------------------------------------------------------------------

// Iterate

#[must_use]
pub fn iterate(
    index: Keyspace,
    position: Option<Position>,
    specifier: &SpecifierHash,
) -> SequentialPositionIterator {
    let version_bounds = specifier
        .range()
        .as_ref()
        .map_or((u8::MIN, u8::MAX), |r| (r.start.value(), r.end.value()));

    let version_min = version_bounds.0;
    let version_max = version_bounds.1;
    let version_filter = version_min > u8::MIN || version_max < u8::MAX;

    let filter_map = move |key_value: Result<(Slice, Slice), Error>| {
        let (key, value) = key_value.expect("invalid key/value during iteration");

        if version_filter {
            let mut value = &value[..];

            let version = value.get_u8();

            if !(version_min..version_max).contains(&version) {
                return None;
            }
        }

        let mut key = &key[..];

        key.advance(ID_LEN + HASH_LEN);

        let position = key.get_u64();
        let position = Position::new(position);

        Some(position)
    };

    let iterator = match position {
        Some(position) => range(index, position, specifier, filter_map),
        None => prefix(index, specifier, filter_map),
    };

    iterator.into()
}

fn prefix<F>(
    index: Keyspace,
    specification: &SpecifierHash,
    filter_map: F,
) -> OwnedSequentialPositionIterator
where
    F: Fn(Result<(Slice, Slice), Error>) -> Option<Position> + 'static,
{
    let mut prefix = [0u8; PREFIX_LEN];

    let identifier = specification.identifer();

    write_prefix(&mut prefix, identifier.hash());

    OwnedSequentialPositionIterator::new(index, |keyspace| {
        Box::new(
            keyspace
                .prefix(prefix)
                .map(Guard::into_inner)
                .filter_map(filter_map),
        )
    })
}

fn range<F>(
    index: Keyspace,
    position: Position,
    specifier: &SpecifierHash,
    filter_map: F,
) -> OwnedSequentialPositionIterator
where
    F: Fn(Result<(Slice, Slice), Error>) -> Option<Position> + 'static,
{
    let mut lower = [0u8; KEY_LEN];

    let identifier = specifier.identifer();

    write_key(&mut lower, position, identifier.hash());

    let mut upper = [0u8; KEY_LEN];

    let position = Position::new(u64::MAX);

    write_key(&mut upper, position, identifier.hash());

    let range = lower..=upper;

    OwnedSequentialPositionIterator::new(index, |keyspace| {
        Box::new(
            keyspace
                .range(range)
                .map(Guard::into_inner)
                .filter_map(filter_map),
        )
    })
}

// -------------------------------------------------------------------------------------------------

// Keys/Prefixes

fn write_key(key: &mut [u8; KEY_LEN], position: Position, identifier: u64) {
    let mut key = &mut key[..];

    let index_id = INDEX_ID;
    let position = position.value();

    key.put_u8(index_id);
    key.put_u64(identifier);
    key.put_u64(position);
}

fn write_prefix(prefix: &mut [u8; PREFIX_LEN], identifier: u64) {
    let mut prefix = &mut prefix[..];

    let index_id = INDEX_ID;

    prefix.put_u8(index_id);
    prefix.put_u64(identifier);
}
