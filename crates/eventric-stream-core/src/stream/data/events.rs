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
    OwnedWriteBatch,
    Slice,
};

use crate::{
    error::Error,
    event::{
        CandidateEventHashAndValue,
        EventHash,
        data::Data,
        identifier::IdentifierHash,
        position::Position,
        tag::TagHash,
        timestamp::Timestamp,
        version::Version,
    },
    stream::data::indices::PositionIter,
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
    pub fn get(&self, at: Position) -> Result<Option<EventHash>, Error> {
        let key = at.to_be_bytes();
        let value = self.keyspace.get(key)?;

        Ok(value.map(|value| IntoEventHash(at, value).into()))
    }

    pub fn put(
        &self,
        batch: &mut OwnedWriteBatch,
        at: Position,
        event: &CandidateEventHashAndValue,
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
    pub fn iterate(&self, from: Option<Position>) -> DirectEventHashIter {
        let iter = if let Some(from) = from {
            self.keyspace.range(from.to_be_bytes()..)
        } else {
            self.keyspace.iter()
        };

        DirectEventHashIter::new(iter)
    }
}

// Properties

impl Events {
    pub fn len(&self) -> Result<u64, Error> {
        match self.keyspace.last_key_value() {
            Some(guard) => Ok(guard.key().map(|key| key.as_ref().get_u64() + 1)?),
            _ => Ok(0),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Iterators

#[derive(Debug)]
pub(crate) enum EventHashIter {
    Direct(DirectEventHashIter),
    Mapped(MappedEventHashIter),
}

impl DoubleEndedIterator for EventHashIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Direct(iter) => iter.next_back(),
            Self::Mapped(iter) => iter.next_back(),
        }
    }
}

impl Iterator for EventHashIter {
    type Item = Result<EventHash, Error>;

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
pub(crate) struct DirectEventHashIter {
    #[debug("Iter")]
    iter: fjall::Iter,
}

impl DirectEventHashIter {
    fn next_map(guard: Guard) -> <Self as Iterator>::Item {
        match guard.into_inner() {
            Ok((key, value)) => Ok(IntoEventHash(IntoPosition(key).into(), value).into()),
            Err(err) => Err(Error::from(err)),
        }
    }
}

impl DoubleEndedIterator for DirectEventHashIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Self::next_map)
    }
}

impl Iterator for DirectEventHashIter {
    type Item = Result<EventHash, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::next_map)
    }
}

// Mapped

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct MappedEventHashIter {
    events: Events,
    iter: PositionIter,
}

impl MappedEventHashIter {
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

impl DoubleEndedIterator for MappedEventHashIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .and_then(|position| self.map(position))
    }
}

impl Iterator for MappedEventHashIter {
    type Item = Result<EventHash, Error>;

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

// Slice & Position -> EventHash

struct IntoEventHash(Position, Slice);

impl From<IntoEventHash> for EventHash {
    fn from(IntoEventHash(position, value): IntoEventHash) -> Self {
        let mut value = &value[..];

        let identifier = IdentifierHash::new(value.get_u64());
        let version = Version::new(value.get_u8());
        let tags = (0..value.get_u8())
            .map(|_| TagHash::new(value.get_u64()))
            .collect();

        let timestamp = Timestamp::new(value.get_u64());
        let data = Data::new_unvalidated(value.iter().map(ToOwned::to_owned).collect::<Vec<_>>());

        Self::new(data, identifier, position, tags, timestamp, version)
    }
}

// Event & Timestamp -> Value Byte Vector

struct IntoValueBytes<'a>(&'a CandidateEventHashAndValue, Timestamp);

impl From<IntoValueBytes<'_>> for Vec<u8> {
    fn from(IntoValueBytes(event, timestamp): IntoValueBytes<'_>) -> Self {
        let mut value = Vec::new();

        value.put_u64(event.identifier.0.0);
        value.put_u8(*event.version);
        value.put_u8(u8::try_from(event.tags.len()).expect("max tag count exceeded"));

        for tag in &event.tags {
            value.put_u64(tag.0.0);
        }

        value.put_u64(*timestamp);
        value.put_slice(event.data.as_ref());

        value
    }
}
