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

#[derive(new, Debug)]
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
        position: Position,
        timestamp: Timestamp,
    ) {
        let key = position.value().to_be_bytes();
        let value: Vec<u8> = EventAndTimestamp(event, timestamp).into();

        batch.insert(&self.keyspace, key, value);
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

        SequencedEventHash::new(data, identifier, position, tags, timestamp, version)
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
