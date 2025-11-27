use bytes::{
    Buf as _,
    BufMut as _,
};
use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Database,
    Guard,
    Keyspace,
    KeyspaceCreateOptions,
    Slice,
    WriteBatch,
};
use itertools::Itertools as _;

use crate::{
    error::Error,
    event::{
        EphemeralEventHashRef,
        PersistentEventHash,
        data::Data,
        identifier::IdentifierHash,
        position::Position,
        tag::TagHash,
        timestamp::Timestamp,
        version::Version,
    },
    stream::data::indices::PositionIterator,
};

// =================================================================================================
// Events
// =================================================================================================

// Configuration

static KEYSPACE_NAME: &str = "events";

// -------------------------------------------------------------------------------------------------

// Data

#[derive(new, Clone, Debug)]
#[new(const_fn, vis())]
pub(crate) struct Events {
    #[debug("Keyspace(\"{}\")", keyspace.name)]
    keyspace: Keyspace,
}

impl Events {
    pub fn open(database: &Database) -> Result<Self, Error> {
        let keyspace = database.keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default)?;

        Ok(Self::new(keyspace))
    }
}

// Get/Put

impl Events {
    pub fn get(&self, at: Position) -> Result<Option<PersistentEventHash>, Error> {
        let key = at.to_be_bytes();
        let value = self.keyspace.get(key)?;

        Ok(value.map(|value| IntoPersistentEventHash(at, value).into()))
    }

    pub fn put(
        &self,
        batch: &mut WriteBatch,
        at: Position,
        event: &EphemeralEventHashRef<'_>,
        timestamp: Timestamp,
    ) {
        let key = at.to_be_bytes();
        let value: Vec<u8> = IntoValueBytes(event, timestamp).into();

        batch.insert(&self.keyspace, key, value);
    }
}

// Iterate

impl Events {
    #[must_use]
    #[rustfmt::skip]
    pub fn iterate(&self, from: Option<Position>) -> Iter {
        let iter = if let Some(from) = from {
            self.keyspace.range(from.to_be_bytes()..)
        } else {
            self.keyspace.iter()
        };

        Iter::new(iter)
    }
}

// Properties

impl Events {
    pub fn len(&self) -> Result<u64, Error> {
        match self.keyspace.last_key_value()? {
            Some((key, _)) => Ok(key.as_ref().get_u64() + 1),
            _ => Ok(0),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Iterators

#[derive(Debug)]
pub(crate) enum PersistentEventHashIterator {
    Direct(Iter),
    Mapped(MappedPersistentEventHashIterator),
}

impl DoubleEndedIterator for PersistentEventHashIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Direct(iter) => iter.next_back(),
            Self::Mapped(iter) => iter.next_back(),
        }
    }
}

impl Iterator for PersistentEventHashIterator {
    type Item = Result<PersistentEventHash, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Direct(iter) => iter.next(),
            Self::Mapped(iter) => iter.next(),
        }
    }
}

// Direct

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct Iter {
    #[debug("Iter")]
    iter: fjall::Iter,
}

impl Iter {
    fn next_map(guard: Guard) -> <Self as Iterator>::Item {
        match guard.into_inner() {
            Ok((key, value)) => Ok(IntoPersistentEventHash(IntoPosition(key).into(), value).into()),
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
    type Item = Result<PersistentEventHash, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::next_map)
    }
}

// Mapped

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct MappedPersistentEventHashIterator {
    events: Events,
    iter: PositionIterator,
}

impl MappedPersistentEventHashIterator {
    fn map(&mut self, position: Result<Position, Error>) -> Option<<Self as Iterator>::Item> {
        match position {
            Ok(position) => match self.events.get(position) {
                Ok(Some(event)) => Some(Ok(event)),
                Ok(None) => None,
                Err(err) => Some(Err(err)),
            },
            Err(err) => Some(Err(err)),
        }
    }
}

impl DoubleEndedIterator for MappedPersistentEventHashIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .and_then(|position| self.map(position))
    }
}

impl Iterator for MappedPersistentEventHashIterator {
    type Item = Result<PersistentEventHash, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().and_then(|position| self.map(position))
    }
}

// -------------------------------------------------------------------------------------------------

// Conversions

// Key (Slice) -> Position

struct IntoPosition(Slice);

impl From<IntoPosition> for Position {
    fn from(IntoPosition(slice): IntoPosition) -> Self {
        Self::new(slice.as_ref().get_u64())
    }
}

// Slice & Position -> PersistentEventHash

struct IntoPersistentEventHash(Position, Slice);

impl From<IntoPersistentEventHash> for PersistentEventHash {
    fn from(IntoPersistentEventHash(position, value): IntoPersistentEventHash) -> Self {
        let mut value = &value[..];

        let identifier = IdentifierHash::new(value.get_u64());
        let version = Version::new(value.get_u8());
        let tags = (0..value.get_u8())
            .map(|_| TagHash::new(value.get_u64()))
            .collect();

        let timestamp = Timestamp::new(value.get_u64());
        let data = Data::new_unvalidated(value.iter().map(ToOwned::to_owned).collect_vec());

        Self::new(data, identifier, position, tags, timestamp, version)
    }
}

// Event & Timestamp -> Value Byte Vector

struct IntoValueBytes<'a>(&'a EphemeralEventHashRef<'a>, Timestamp);

impl From<IntoValueBytes<'_>> for Vec<u8> {
    fn from(IntoValueBytes(event, timestamp): IntoValueBytes<'_>) -> Self {
        let mut value = Vec::new();

        value.put_u64(event.identifier.hash_val());
        value.put_u8(*event.version);
        value.put_u8(u8::try_from(event.tags.len()).expect("max tag count exceeded"));

        for tag in &event.tags {
            value.put_u64(tag.hash_val());
        }

        value.put_u64(*timestamp);
        value.put_slice(event.data.as_ref());

        value
    }
}
