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
use self_cell::self_cell;

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
        let keyspace = database.keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())?;

        Ok(Self::new(keyspace))
    }
}

// Get/Put

impl Events {
    pub fn get(&self, at: Position) -> Result<Option<PersistentEventHash>, Error> {
        let key = at.to_be_bytes();
        let value = self.keyspace.get(key)?;

        Ok(value.map(|value| SliceAndPosition(value, at).into()))
    }

    pub fn put(
        &self,
        batch: &mut WriteBatch,
        at: Position,
        event: &EphemeralEventHashRef<'_>,
        timestamp: Timestamp,
    ) {
        let key = at.to_be_bytes();
        let value: Vec<u8> = EventAndTimestamp(event, timestamp).into();

        batch.insert(&self.keyspace, key, value);
    }
}

// Iterate

impl Events {
    #[must_use]
    pub fn iterate(&self, from: Option<Position>) -> DirectPersistentEventHashIterator {
        match from {
            Some(position) => self.iterate_from(position),
            None => self.iterate_all(),
        }
    }

    fn iterate_all(&self) -> DirectPersistentEventHashIterator {
        DirectPersistentEventHashIterator::new(self.keyspace.clone(), |keyspace| {
            Box::new(
                keyspace
                    .iter()
                    .map(Guard::into_inner)
                    .map(DirectPersistentEventHashIterator::map),
            )
        })
    }

    fn iterate_from(&self, from: Position) -> DirectPersistentEventHashIterator {
        let range = from.to_be_bytes()..;

        DirectPersistentEventHashIterator::new(self.keyspace.clone(), |keyspace| {
            Box::new(
                keyspace
                    .range(range)
                    .map(Guard::into_inner)
                    .map(DirectPersistentEventHashIterator::map),
            )
        })
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
    Direct(#[debug("Peristent Event Hash Iterator")] DirectPersistentEventHashIterator),
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

#[rustfmt::skip]
type BoxedPersistentEventHashIterator<'a> = Box<dyn DoubleEndedIterator<Item = Result<PersistentEventHash, Error>> + Send + 'a>;

self_cell!(
    pub(crate) struct DirectPersistentEventHashIterator {
        owner: Keyspace,
        #[covariant]
        dependent: BoxedPersistentEventHashIterator,
    }
);

impl DirectPersistentEventHashIterator {
    fn map(result: Result<(Slice, Slice), fjall::Error>) -> <Self as Iterator>::Item {
        match result {
            Ok((key, value)) => Ok(SliceAndPosition(value, Key(key).into()).into()),
            Err(err) => Err(Error::from(err)),
        }
    }
}

impl DoubleEndedIterator for DirectPersistentEventHashIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.with_dependent_mut(|_, iter| iter.next_back())
    }
}

impl Iterator for DirectPersistentEventHashIterator {
    type Item = Result<PersistentEventHash, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_dependent_mut(|_, iter| iter.next())
    }
}

#[allow(unsafe_code)]
unsafe impl Sync for DirectPersistentEventHashIterator {}

// Mapped

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct MappedPersistentEventHashIterator {
    events: Events,
    iter: PositionIterator,
}

impl MappedPersistentEventHashIterator {
    fn map(&mut self, position: Result<Position, Error>) -> <Self as Iterator>::Item {
        match position {
            Ok(position) => match self.events.get(position) {
                Ok(Some(event)) => Ok(event),
                Ok(None) => Err(Error::data("event not found")),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }
}

impl DoubleEndedIterator for MappedPersistentEventHashIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|position| self.map(position))
    }
}

impl Iterator for MappedPersistentEventHashIterator {
    type Item = Result<PersistentEventHash, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|position| self.map(position))
    }
}

// -------------------------------------------------------------------------------------------------

// Conversions

// Key (Slice) -> Position

struct Key(Slice);

impl From<Key> for Position {
    fn from(Key(slice): Key) -> Self {
        Self::new(slice.as_ref().get_u64())
    }
}

// Slice & Position -> PersistentEventHash

struct SliceAndPosition(Slice, Position);

impl From<SliceAndPosition> for PersistentEventHash {
    fn from(SliceAndPosition(value, position): SliceAndPosition) -> Self {
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

struct EventAndTimestamp<'a>(&'a EphemeralEventHashRef<'a>, Timestamp);

impl From<EventAndTimestamp<'_>> for Vec<u8> {
    fn from(EventAndTimestamp(event, timestamp): EventAndTimestamp<'_>) -> Self {
        let mut value = Vec::new();

        value.put_u64(event.identifier.hash());
        value.put_u8(*event.version);
        value.put_u8(u8::try_from(event.tags.len()).expect("max tag count exceeded"));

        for tag in &event.tags {
            value.put_u64(tag.hash());
        }

        value.put_u64(*timestamp);
        value.put_slice(event.data.as_ref());

        value
    }
}
