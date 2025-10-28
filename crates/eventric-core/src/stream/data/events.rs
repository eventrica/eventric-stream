use std::iter;

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

use crate::{
    error::Error,
    event::{
        Data,
        EventHashRef,
        Position,
        SequencedEventHash,
        Timestamp,
        Version,
        identifier::IdentifierHash,
        tag::TagHash,
    },
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
    pub fn get(&self, at: Position) -> Result<Option<SequencedEventHash>, Error> {
        let key = at.to_be_bytes();
        let value = self.keyspace.get(key)?;

        Ok(value.map(|value| SliceAndPosition(value, at).into()))
    }

    pub fn put(
        &self,
        batch: &mut WriteBatch,
        at: Position,
        event: &EventHashRef<'_>,
        timestamp: Timestamp,
    ) {
        let key = at.to_be_bytes();
        let value: Vec<u8> = EventAndTimestamp(event, timestamp).into();

        batch.insert(&self.keyspace, key, value);
    }
}

// Iterate

impl Events {
    pub fn iterate(&self, from: Option<Position>) -> Iterator<'_> {
        match from {
            Some(position) => self.iterate_from(position),
            None => self.iterate_all(),
        }
    }

    fn iterate_all(&self) -> Iterator<'_> {
        Iterator {
            iter: Box::new(self.keyspace.iter().map(|guard| match guard.into_inner() {
                Ok((key, value)) => Ok(SliceAndPosition(value, key.into()).into()),
                Err(err) => Err(Error::from(err)),
            })),
        }
    }

    fn iterate_from(&self, from: Position) -> Iterator<'_> {
        Iterator {
            iter: Box::new(self.keyspace.range(from.to_be_bytes()..).map(|guard| {
                match guard.into_inner() {
                    Ok((key, value)) => Ok(SliceAndPosition(value, key.into()).into()),
                    Err(err) => Err(Error::from(err)),
                }
            })),
        }
    }
}

// Properties

impl Events {
    pub fn is_empty(&self) -> Result<bool, Error> {
        self.len().map(|len| len == 0)
    }

    pub fn len(&self) -> Result<u64, Error> {
        match self.keyspace.last_key_value()? {
            Some((key, _)) => Ok(key.as_ref().get_u64() + 1),
            _ => Ok(0),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Conversions

impl From<Slice> for Position {
    fn from(slice: Slice) -> Self {
        Self::new(slice.as_ref().get_u64())
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
            .collect();

        let timestamp = Timestamp::new(value.get_u64());
        let data = Data::new_unvalidated(value.iter().map(ToOwned::to_owned).collect::<Vec<_>>());

        Self::new(data, identifier, position, tags, timestamp, version)
    }
}

struct EventAndTimestamp<'a>(&'a EventHashRef<'a>, Timestamp);

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

// -------------------------------------------------------------------------------------------------

// Iterator

#[derive(new, Debug)]
pub(crate) struct Iterator<'a> {
    #[debug("Iterator")]
    iter: Box<dyn iter::Iterator<Item = Result<SequencedEventHash, Error>> + 'a>,
}

impl iter::Iterator for Iterator<'_> {
    type Item = Result<SequencedEventHash, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
