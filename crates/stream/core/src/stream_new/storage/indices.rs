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
    event_new::{
        Event,
        Tag,
    },
    stream_new::{
        Facets,
        storage::{
            HASH_LEN,
            ID_LEN,
            POSITION_LEN,
        },
    },
};

// =================================================================================================
// Indices
// =================================================================================================

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct Indices {
    tags: Tags,
    timestamps: Timestamps,
    types: Types,
}

impl Indices {
    pub fn open(database: &Database) -> Result<Self, Error> {
        let keyspace = database.keyspace("indices", KeyspaceCreateOptions::default)?;

        let tags = Tags::new(keyspace.clone());
        let timestamps = Timestamps::new(keyspace.clone());
        let types = Types::new(keyspace);

        Ok(Self::new(tags, timestamps, types))
    }
}

impl Indices {
    pub fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, facets: &Facets) {
        self.tags.insert(batch, event, facets);
        self.timestamps.insert(batch, facets);
        self.types.insert(batch, event, facets);
    }
}

// -------------------------------------------------------------------------------------------------

// Tags

#[derive(new, Debug)]
struct Tags {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Tags {
    fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, facets: &Facets) {
        for tag in &event.1.1 {
            let key: TagsKey = TagsKeyConverter(tag, facets).into(); // Tag & Position
            let value = []; // Empty

            batch.insert(&self.keyspace, key, value);
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Tags Constants

static TAGS_INDEX_ID: u8 = 0;
static TAGS_KEY_LEN: usize = ID_LEN + HASH_LEN + POSITION_LEN;
static TAGS_PREFIX_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Tags Converters

struct TagsKeyConverter<'a>(&'a Tag<u64>, &'a Facets);

impl From<TagsKeyConverter<'_>> for TagsKey {
    fn from(TagsKeyConverter(tag, facets): TagsKeyConverter<'_>) -> Self {
        let mut key = TagsKey::default();

        {
            let mut key = &mut key[..];

            key.put_u8(TAGS_INDEX_ID);
            key.put_u64(tag.0); // Tag
            key.put_u64(facets.0.0); // Position
        }

        key
    }
}

// -------------------------------------------------------------------------------------------------

// Tags Types

type TagsKey = [u8; TAGS_KEY_LEN];

// -------------------------------------------------------------------------------------------------

// Timestamps

#[derive(new, Debug)]
struct Timestamps {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Timestamps {
    fn insert(&self, batch: &mut Batch, facets: &Facets) {
        let key: TimestampsKey = TimestampsKeyConverter(facets).into(); // Timestamp
        let value = facets.0.0.to_be_bytes(); // Position

        batch.insert(&self.keyspace, key, value);
    }
}

// -------------------------------------------------------------------------------------------------

// Timestamps Constants

static TIMESTAMPS_INDEX_ID: u8 = 1;
static TIMESTAMPS_KEY_LEN: usize = ID_LEN + TIMESTAMPS_LEN;
static TIMESTAMPS_LEN: usize = size_of::<u64>();

// -------------------------------------------------------------------------------------------------

// Timestamps Converters

struct TimestampsKeyConverter<'a>(&'a Facets);

impl From<TimestampsKeyConverter<'_>> for TimestampsKey {
    fn from(TimestampsKeyConverter(facets): TimestampsKeyConverter<'_>) -> Self {
        let mut key = TimestampsKey::default();

        {
            let mut key = &mut key[..];

            key.put_u8(TIMESTAMPS_INDEX_ID);
            key.put_u64(facets.1.0);
        }

        key
    }
}

// -------------------------------------------------------------------------------------------------

// Timestamps Types

type TimestampsKey = [u8; TIMESTAMPS_KEY_LEN];

// -------------------------------------------------------------------------------------------------

// Types

#[derive(new, Debug)]
struct Types {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Types {
    fn insert(&self, batch: &mut Batch, event: &Event<(), u64>, facets: &Facets) {
        let key: TypesKey = TypesKeyConverter(event, facets).into(); // Type & Position
        let value = event.1.0.1.0.to_be_bytes(); // Version

        batch.insert(&self.keyspace, key, value);
    }
}

// -------------------------------------------------------------------------------------------------

// Types Constants

static TYPES_INDEX_ID: u8 = 2;
static TYPES_KEY_LEN: usize = ID_LEN + HASH_LEN + POSITION_LEN;
static TYPES_PREFIX_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Types Converters

struct TypesKeyConverter<'a>(&'a Event<(), u64>, &'a Facets);

impl From<TypesKeyConverter<'_>> for TypesKey {
    fn from(TypesKeyConverter(event, facets): TypesKeyConverter<'_>) -> Self {
        let mut key = TypesKey::default();

        {
            let mut key = &mut key[..];

            key.put_u8(TYPES_INDEX_ID);
            key.put_u64(event.1.0.0.0); // Type Name
            key.put_u64(facets.0.0); // Position
        }

        key
    }
}

// -------------------------------------------------------------------------------------------------

// Types Types

type TypesKey = [u8; TYPES_KEY_LEN];
