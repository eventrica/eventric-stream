use bytes::BufMut as _;
use derive_more::Debug;
use fancy_constructor::new;
use fjall::{
    Database,
    Keyspace,
    KeyspaceCreateOptions,
    OwnedWriteBatch as Batch,
};

use crate::{
    error::Error,
    event_new::Event,
    stream_new::Facets,
};

// =================================================================================================
// Events
// =================================================================================================

#[derive(new, Debug)]
pub struct Events {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Events {
    pub fn open(database: &Database) -> Result<Self, Error> {
        database
            .keyspace("events", KeyspaceCreateOptions::default)
            .map(Self::new)
            .map_err(Into::into)
    }
}

impl Events {
    pub fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, facets: &Facets) {
        let key = facets.0.0.to_be_bytes(); // Position
        let value: Vec<u8> = ValueConverter(event, facets).into(); // Event & Timestamp

        batch.insert(&self.keyspace, key, value);
    }
}

// -------------------------------------------------------------------------------------------------

// Converters

struct ValueConverter<'a>(&'a Event<(), u64>, &'a Facets);

impl From<ValueConverter<'_>> for Vec<u8> {
    fn from(ValueConverter(event, facets): ValueConverter<'_>) -> Self {
        let mut value = Vec::new();

        value.put_u64(event.1.0.0.0); // Event Type Name
        value.put_u8(event.1.0.1.0); // Event Type Version
        value.put_u8(u8::try_from(event.1.1.len()).expect("max tag count exceeded")); // Tags Len

        for tag in &event.1.1 {
            value.put_u64(tag.0); // Tag
        }

        value.put_u64(facets.1.0); // Timestamp
        value.put_slice(event.0.as_ref()); // Data

        value
    }
}
