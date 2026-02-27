use std::result;

use bytes::{
    Buf as _,
    BufMut as _,
};
use derive_more::{
    Debug,
    From,
};
use error_stack::ResultExt;
use fancy_constructor::new;
use fjall::{
    Database,
    Guard,
    Keyspace,
    KeyspaceCreateOptions,
    OwnedWriteBatch as Batch,
    Slice,
};

use crate::{
    event_new::{
        self,
        Data,
        Event,
        Name,
        Tag,
        Type,
        Version,
    },
    stream_new::{
        self,
        Error,
        Position,
        Result,
        Timestamp,
        storage::indices::IndicesIter,
    },
};

// =================================================================================================
// Events
// =================================================================================================

#[derive(new, Clone, Debug)]
pub struct Events {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Events {
    pub fn open(database: &Database) -> Result<Self> {
        database
            .keyspace("events", KeyspaceCreateOptions::default)
            .map(Self::new)
            .change_context(Error)
            .attach("failed to open events keyspace")
    }
}

impl Events {
    pub fn len(&self) -> Result<u64> {
        let len = match self.keyspace.last_key_value() {
            Some(guard) => guard
                .key()
                .map(|key| key.as_ref().get_u64() + 1)
                .change_context(Error)
                .attach("failed to get last key value")?,
            _ => 0,
        };

        Ok(len)
    }
}

impl Events {
    pub fn get(
        &self,
        position: Position,
    ) -> result::Result<Option<Event<stream_new::Facets, u64>>, crate::error::Error> {
        let key = position.0.to_be_bytes(); // Position
        let value = self.keyspace.get(key)?;

        Ok(value.map(|value| EventConverter(position, &value).into()))
    }
}

impl Events {
    pub fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, facets: &stream_new::Facets) {
        let key = facets.0.0.to_be_bytes(); // Position
        let value: Vec<u8> = ValueConverter(event, &facets.1).into(); // Event & Timestamp

        batch.insert(&self.keyspace, key, value);
    }
}

// -------------------------------------------------------------------------------------------------

// Converters

struct EventConverter<'a>(Position, &'a Slice);

impl From<EventConverter<'_>> for Event<stream_new::Facets, u64> {
    fn from(EventConverter(position, value): EventConverter<'_>) -> Self {
        let mut value = &value[..];

        let name = Name(value.get_u64());
        let version = Version(value.get_u8());
        let ty = Type::new(name, version);
        let tags = (0..value.get_u8()).map(|_| Tag(value.get_u64())).collect();
        let facets = event_new::Facets::new(ty, tags);

        let timestamp = Timestamp(value.get_u64());
        let meta = stream_new::Facets::new(position, timestamp);

        let data = Data(value.iter().map(ToOwned::to_owned).collect::<Vec<_>>());

        Self::new(data, facets, meta)
    }
}

struct PositionConverter<'a>(&'a Slice);

impl From<PositionConverter<'_>> for Position {
    fn from(PositionConverter(slice): PositionConverter<'_>) -> Self {
        Self::new(slice.as_ref().get_u64())
    }
}

struct ValueConverter<'a>(&'a Event<(), u64>, &'a Timestamp);

impl From<ValueConverter<'_>> for Vec<u8> {
    fn from(ValueConverter(event, timestamp): ValueConverter<'_>) -> Self {
        let mut value = Vec::new();

        value.put_u64(event.1.0.0.0); // Event Type Name
        value.put_u8(event.1.0.1.0); // Event Type Version
        value.put_u8(u8::try_from(event.1.1.len()).expect("max tag count exceeded")); // Tags Len

        for tag in &event.1.1 {
            value.put_u64(tag.0); // Tag
        }

        value.put_u64(timestamp.0); // Timestamp
        value.put_slice(event.0.as_ref()); // Data

        value
    }
}

// -------------------------------------------------------------------------------------------------

// Iterators

#[derive(Debug, From)]
pub enum EventsIter {
    Direct(EventsIterDirect),
    Mapped(EventsIterMapped),
}

impl DoubleEndedIterator for EventsIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Direct(iter) => iter.next_back(),
            Self::Mapped(iter) => iter.next_back(),
        }
    }
}

impl Iterator for EventsIter {
    type Item = Result<Event<stream_new::Facets, u64>>;

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
pub struct EventsIterDirect {
    #[debug("Iter")]
    iter: fjall::Iter,
}

impl EventsIterDirect {
    fn next_map(guard: Guard) -> <Self as Iterator>::Item {
        match guard.into_inner() {
            Ok((key, value)) => Ok(EventConverter(PositionConverter(&key).into(), &value).into()),
            Err(err) => Err(err)
                .change_context(Error)
                .attach("failed to map next event"),
        }
    }
}

impl DoubleEndedIterator for EventsIterDirect {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Self::next_map)
    }
}

impl Iterator for EventsIterDirect {
    type Item = Result<Event<stream_new::Facets, u64>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::next_map)
    }
}

// Mapped

#[derive(new, Debug)]
#[new(const_fn)]
pub struct EventsIterMapped {
    events: Events,
    iter: IndicesIter,
}

impl EventsIterMapped {
    fn map(&mut self, position: Result<Position>) -> Option<<Self as Iterator>::Item> {
        match position {
            Ok(position) => match self.events.get(position) {
                Ok(Some(event)) => Some(Ok(event)),
                Ok(None) => None,
                Err(err) => Some(
                    Err(err)
                        .change_context(Error)
                        .attach("failed to get next event"),
                ),
            },
            Err(err) => Some(
                Err(err)
                    .change_context(Error)
                    .attach("failed to map next event"),
            ),
        }
    }
}

impl DoubleEndedIterator for EventsIterMapped {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .and_then(|position| self.map(position))
    }
}

impl Iterator for EventsIterMapped {
    type Item = Result<Event<stream_new::Facets, u64>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().and_then(|position| self.map(position))
    }
}
