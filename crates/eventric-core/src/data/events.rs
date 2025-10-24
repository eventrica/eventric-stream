use std::error::Error;

use bytes::{
    Buf as _,
    BufMut as _,
};
use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Database,
    Keyspace,
    KeyspaceCreateOptions,
    Slice,
    WriteBatch,
};
use itertools::Itertools;
use self_cell::self_cell;

use crate::model::{
    event::{
        EventHashRef,
        SequencedEventHash,
        data::Data,
        identifier::IdentifierHash,
        tag::TagHash,
        timestamp::Timestamp,
        version::Version,
    },
    stream::position::Position,
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
pub struct Events {
    #[debug("Keyspace(\"{}\")", keyspace.name)]
    keyspace: Keyspace,
}

impl Events {
    pub fn open(database: &Database) -> Result<Self, Box<dyn Error>> {
        let keyspace = database.keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())?;

        Ok(Self::new(keyspace))
    }
}

// Get/Put

impl Events {
    pub fn get(&self, position: Position) -> Result<Option<SequencedEventHash>, Box<dyn Error>> {
        let key = position.value().to_be_bytes();
        let value = self.keyspace.get(key)?;
        let event = value.map(|value| SliceAndPosition(value, position).into());

        Ok(event)
    }

    pub fn put(
        &self,
        batch: &mut WriteBatch,
        event: &EventHashRef<'_>,
        timestamp: Timestamp,
        position: Position,
    ) {
        let key = position.value().to_be_bytes();
        let value: Vec<u8> = EventAndTimestamp(event, timestamp).into();

        batch.insert(&self.keyspace, key, value);
    }
}

// Iterate

impl Events {
    pub fn iterate(&self, position: Option<Position>) -> SequencedEventHashIterator {
        SequencedEventHashIterator::new(match position {
            Some(position) => self.iterate_from(position),
            None => self.iterate_all(),
        })
    }

    fn iterate_all(&self) -> OwnedSequencedEventHashIterator {
        OwnedSequencedEventHashIterator::new(self.keyspace.clone(), |keyspace| {
            Box::new(
                keyspace
                    .iter()
                    .map(|guard| guard.into_inner().expect("iteration error"))
                    .map(|event| SliceAndPosition(event.1, event.0.into()).into()),
            )
        })
    }

    fn iterate_from(&self, position: Position) -> OwnedSequencedEventHashIterator {
        OwnedSequencedEventHashIterator::new(self.keyspace.clone(), |keyspace| {
            Box::new(
                keyspace
                    .range(position.value().to_be_bytes()..)
                    .map(|guard| guard.into_inner().expect("iteration error"))
                    .map(|event| SliceAndPosition(event.1, event.0.into()).into()),
            )
        })
    }
}

// Properties

impl Events {
    pub fn is_empty(&self) -> Result<bool, Box<dyn Error>> {
        self.len().map(|len| len == 0)
    }

    pub fn len(&self) -> Result<u64, Box<dyn Error>> {
        match self.keyspace.last_key_value()? {
            Some((key, _)) => Ok(key.as_ref().get_u64() + 1),
            _ => Ok(0),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Conversions

impl From<Slice> for Position {
    fn from(value: Slice) -> Self {
        Self::new(value.as_ref().get_u64())
    }
}

struct SliceAndPosition(Slice, Position);

impl From<SliceAndPosition> for SequencedEventHash {
    fn from(SliceAndPosition(value, position): SliceAndPosition) -> Self {
        let mut value = &value[..];

        let identifier = IdentifierHash::new(value.get_u64());
        let version = Version::new(value.get_u8());
        let tags = (0..value.get_u8())
            .map(|_| TagHash::new(value.get_u64()))
            .collect_vec();

        let timestamp = Timestamp::new(value.get_u64());
        let data = Data::new(value.iter().map(ToOwned::to_owned).collect::<Vec<_>>());

        Self::new(data, identifier, position, tags, timestamp, version)
    }
}

struct EventAndTimestamp<'a>(&'a EventHashRef<'a>, Timestamp);

impl From<EventAndTimestamp<'_>> for Vec<u8> {
    fn from(EventAndTimestamp(event, timestamp): EventAndTimestamp<'_>) -> Self {
        let mut value = Vec::new();

        value.put_u64(event.identifier().hash());
        value.put_u8(event.version().value());
        value.put_u8(u8::try_from(event.tags().len()).expect("max tag count exceeded"));

        for tag in event.tags() {
            value.put_u64(tag.hash());
        }

        value.put_u64(timestamp.nanos());
        value.put_slice(event.data().as_ref());

        value
    }
}

// -------------------------------------------------------------------------------------------------

// Iterator

// Sequenced Event Hash Iterator

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct SequencedEventHashIterator(#[debug("Iterator")] OwnedSequencedEventHashIterator);

impl Iterator for SequencedEventHashIterator {
    type Item = SequencedEventHash;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

// Boxed Sequenced Event Hash Iterator

type BoxedSequencedEventHashIterator<'a> = Box<dyn Iterator<Item = SequencedEventHash> + 'a>;

// Owned Sequenced Event Hash Iterator

self_cell!(
    struct OwnedSequencedEventHashIterator {
        owner: Keyspace,
        #[covariant]
        dependent: BoxedSequencedEventHashIterator,
    }
);

impl Iterator for OwnedSequencedEventHashIterator {
    type Item = SequencedEventHash;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_dependent_mut(|_, iterator| iterator.next())
    }
}
