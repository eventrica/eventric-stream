use bytes::{
    Buf as _,
    BufMut as _,
};
use derive_more::Debug;
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
        Error,
        Facets,
        Position,
        Result,
        Timestamp,
    },
};

// =================================================================================================
// Events
// =================================================================================================

// Event Reader

struct EventReader<'a>(Position, &'a Slice);

impl From<EventReader<'_>> for Event<Facets, u64> {
    fn from(EventReader(position, value): EventReader<'_>) -> Self {
        let mut value = &value[..];

        let name = Name(value.get_u64());
        let version = Version(value.get_u8());
        let ty = Type::new(name, version);
        let tags = (0..value.get_u8()).map(|_| Tag(value.get_u64())).collect();
        let facets = event_new::Facets::new(ty, tags);

        let timestamp = Timestamp(value.get_u64());
        let meta = Facets::new(position, timestamp);

        let data = Data(value.iter().map(ToOwned::to_owned).collect::<Vec<_>>());

        Self::new(data, facets, meta)
    }
}

// -------------------------------------------------------------------------------------------------

// Event Writer

struct EventWriter<'a>(&'a Event<(), u64>, &'a Timestamp);

impl From<EventWriter<'_>> for Vec<u8> {
    fn from(EventWriter(event, timestamp): EventWriter<'_>) -> Self {
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

// Events

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
    pub fn get(&self, position: Position) -> Result<Option<Event<Facets, u64>>> {
        let key = position.0.to_be_bytes(); // Position
        let value = self
            .keyspace
            .get(key)
            .change_context(Error)
            .attach("failed to get value from events keyspace")?;

        Ok(value.map(|value| EventReader(position, &value).into()))
    }
}

impl Events {
    pub fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, facets: &Facets) {
        let key = facets.0.0.to_be_bytes(); // Position
        let value: Vec<u8> = EventWriter(event, &facets.1).into(); // Event & Timestamp

        batch.insert(&self.keyspace, key, value);
    }
}

impl Events {
    pub fn iterate(&self, from: Option<Position>) -> EventsIter {
        let iter = from.map_or_else(
            || self.keyspace.iter(),
            |from| self.keyspace.range(from.0.to_be_bytes()..),
        );

        EventsIter::new(iter)
    }
}

// -------------------------------------------------------------------------------------------------

// Events Iterator

#[derive(new, Debug)]
#[new(const_fn)]
pub struct EventsIter {
    #[debug("Iter")]
    iter: fjall::Iter,
}

impl EventsIter {
    fn next_map(guard: Guard) -> <Self as Iterator>::Item {
        match guard.into_inner() {
            Ok((key, value)) => Ok(EventReader(PositionReader(&key).into(), &value).into()),
            Err(err) => Err(err)
                .change_context(Error)
                .attach("failed to map next event"),
        }
    }
}

impl DoubleEndedIterator for EventsIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Self::next_map)
    }
}

impl Iterator for EventsIter {
    type Item = Result<Event<Facets, u64>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::next_map)
    }
}

// -------------------------------------------------------------------------------------------------

// Position Reader

struct PositionReader<'a>(&'a Slice);

impl From<PositionReader<'_>> for Position {
    fn from(PositionReader(slice): PositionReader<'_>) -> Self {
        Self::new(slice.as_ref().get_u64())
    }
}
