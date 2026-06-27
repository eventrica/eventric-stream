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
    error::{
        Error,
        Result,
    },
    event::{
        self,
        Data,
        Event,
        Name,
        Tag,
        Type,
        Version,
    },
    stream::{
        Metadata,
        Position,
        Timestamp,
    },
};

// =================================================================================================
// Events
// =================================================================================================

// Event Reader

struct EventReader<'a>(Position, &'a Slice);

impl From<EventReader<'_>> for Event<Metadata, u64> {
    fn from(EventReader(position, value): EventReader<'_>) -> Self {
        let mut value = &value[..];

        let name = Name(value.get_u64());
        let version = Version(value.get_u8());
        let ty = Type::new(name, version);
        let tags = (0..value.get_u8()).map(|_| Tag(value.get_u64())).collect();
        let facets = event::Facets::new(ty, tags);

        let timestamp = Timestamp(value.get_u64());
        let meta = Metadata::new(position, timestamp);

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
        let ty = event.facets().ty();
        let tags = event.facets().tags();

        value.put_u64(ty.name().0); // Event Type Name (hash)
        value.put_u8(ty.version().0); // Event Type Version
        value.put_u8(u8::try_from(tags.len()).expect("tag count > u8::MAX (rejected at append)")); // Tags Len

        for tag in tags {
            value.put_u64(tag.0); // Tag (hash)
        }

        value.put_u64(timestamp.0); // Timestamp
        value.put_slice(event.data().as_ref()); // Data

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
    pub fn get(&self, position: Position) -> Result<Option<Event<Metadata, u64>>> {
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
    pub fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, meta: &Metadata) {
        let key = meta.0.0.to_be_bytes(); // Position
        let value: Vec<u8> = EventWriter(event, &meta.1).into(); // Event & Timestamp

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
    type Item = Result<Event<Metadata, u64>>;

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
