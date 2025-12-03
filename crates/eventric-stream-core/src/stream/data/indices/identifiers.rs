use std::ops::{
    ControlFlow,
    Range,
};

use bytes::{
    Buf,
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
        indices::PositionIter,
    },
    utils::iteration::or::OrIter,
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

// Iterate

impl Identifiers {
    #[rustfmt::skip]
    pub fn iterate<'a, S>(&self, specifiers: S, from: Option<Position>) -> PositionIter
    where
        S: Iterator<Item = &'a SpecifierHash>,
    {
        OrIter::combine(specifiers.map(|specifier| {
            let identifier = specifier.0;
            let range = specifier.1.clone();

            let iter = if let Some(from) = from {
                self.keyspace
                    .range(Into::<KeyBytes>::into(IntoKeyBytes(from, identifier))
                         ..Into::<KeyBytes>::into(IntoKeyBytes(Position::MAX, identifier)),
                )
            } else {
                self.keyspace
                    .prefix(Into::<PrefixBytes>::into(IntoPrefixBytes(identifier)))
            };

            PositionIter::Identifiers(Iter::new(iter, range))
        }))
    }
}

// Put

impl Identifiers {
    pub fn put(
        &self,
        batch: &mut OwnedWriteBatch,
        at: Position,
        identifier: &IdentifierHashRef<'_>,
        version: Version,
    ) {
        let key: [u8; KEY_LEN] = IntoKeyBytes(at, identifier.into()).into();
        let value = version.to_be_bytes();

        batch.insert(&self.keyspace, key, value);
    }
}

// -------------------------------------------------------------------------------------------------

// Iterator

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct Iter {
    #[debug("Iter")]
    iter: fjall::Iter,
    range: Range<Version>,
}

impl Iter {
    fn next_map(guard: Guard, range: &Range<Version>) -> Option<<Self as Iterator>::Item> {
        match guard.into_inner() {
            Ok((key, value)) => range
                .contains::<Version>(&IntoVersion(value).into())
                .then(|| Ok(IntoPosition(key).into())),
            Err(err) => Some(Err(Error::from(err))),
        }
    }
}

impl DoubleEndedIterator for Iter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .try_rfold((), check(|x| Self::next_map(x, &self.range)))
            .break_value()
    }
}

impl Iterator for Iter {
    type Item = Result<Position, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .try_fold((), check(|x| Self::next_map(x, &self.range)))
            .break_value()
    }
}

#[inline]
fn check<T, U>(mut f: impl FnMut(T) -> Option<U>) -> impl FnMut((), T) -> ControlFlow<U> {
    move |(), x| match f(x) {
        Some(x) => ControlFlow::Break(x),
        None => ControlFlow::Continue(()),
    }
}

// -------------------------------------------------------------------------------------------------

// Conversions

// Hash -> Prefix Byte Array

type PrefixBytes = [u8; PREFIX_LEN];

struct IntoPrefixBytes(IdentifierHash);

impl From<IntoPrefixBytes> for PrefixBytes {
    fn from(IntoPrefixBytes(identifier): IntoPrefixBytes) -> Self {
        let mut prefix = [0u8; PREFIX_LEN];

        {
            let mut prefix = &mut prefix[..];

            prefix.put_u8(INDEX_ID);
            prefix.put_u64(identifier.0);
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

struct IntoKeyBytes(Position, IdentifierHash);

impl From<IntoKeyBytes> for KeyBytes {
    fn from(IntoKeyBytes(position, identifier): IntoKeyBytes) -> Self {
        let mut key = [0u8; KEY_LEN];

        {
            let mut key = &mut key[..];

            key.put_u8(INDEX_ID);
            key.put_u64(identifier.0);
            key.put_u64(*position);
        }

        key
    }
}

// Value (Slice) -> Version

struct IntoVersion(Slice);

impl From<IntoVersion> for Version {
    fn from(IntoVersion(slice): IntoVersion) -> Self {
        Version::new(slice.as_ref().get_u8())
    }
}
