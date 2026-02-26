use bytes::BufMut as _;
use derive_more::Debug;
use error_stack::ResultExt;
use fancy_constructor::new;
use fjall::{
    Database,
    Keyspace,
    KeyspaceCreateOptions,
    OwnedWriteBatch as Batch,
};

use crate::{
    event_new::{
        Event,
        Name,
        Tag,
    },
    stream_new::{
        Error,
        Result,
        storage::{
            HASH_LEN,
            ID_LEN,
        },
    },
};

// =================================================================================================
// References
// =================================================================================================

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct References {
    tags: Tags,
    types: Types,
}

impl References {
    pub fn open(database: &Database) -> Result<Self> {
        let keyspace = database
            .keyspace("references", KeyspaceCreateOptions::default)
            .change_context(Error)
            .attach("failed to open references keyspace")?;

        let tags = Tags::new(keyspace.clone());
        let types = Types::new(keyspace);

        Ok(Self::new(tags, types))
    }
}

impl References {
    pub fn insert(&self, batch: &mut Batch, event: &Event<(), (u64, String)>) {
        self.tags.insert(batch, event);
        self.types.insert(batch, event);
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
    fn insert(&self, batch: &mut Batch, event: &Event<(), (u64, String)>) {
        for tag in &event.1.1 {
            let key: TagsKey = TagsKeyConverter(tag).into();
            let value = tag.0.1.as_bytes();

            batch.insert(&self.keyspace, key, value);
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Tags Constants

static TAGS_REFERENCE_ID: u8 = 0;
static TAGS_KEY_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Tags Converters

struct TagsKeyConverter<'a>(&'a Tag<(u64, String)>);

impl From<TagsKeyConverter<'_>> for TagsKey {
    fn from(TagsKeyConverter(tag): TagsKeyConverter<'_>) -> Self {
        let mut key = TagsKey::default();

        {
            let mut key = &mut key[..];

            key.put_u8(TAGS_REFERENCE_ID);
            key.put_u64(tag.0.0); // Tag (Hashed)
        }

        key
    }
}

// -------------------------------------------------------------------------------------------------

// Tags Types

type TagsKey = [u8; TAGS_KEY_LEN];

// -------------------------------------------------------------------------------------------------

// Types

#[derive(new, Debug)]
struct Types {
    #[debug("Keyspace")]
    keyspace: Keyspace,
}

impl Types {
    fn insert(&self, batch: &mut Batch, event: &Event<(), (u64, String)>) {
        let key: TypesKey = TypesKeyConverter(&event.1.0.0).into(); // Name
        let value = event.1.0.0.0.1.as_bytes(); // Name

        batch.insert(&self.keyspace, key, value);
    }
}

// -------------------------------------------------------------------------------------------------

// Types Constants

static TYPES_REFERENCE_ID: u8 = 1;
static TYPES_KEY_LEN: usize = ID_LEN + HASH_LEN;

// -------------------------------------------------------------------------------------------------

// Types Converters

struct TypesKeyConverter<'a>(&'a Name<(u64, String)>);

impl From<TypesKeyConverter<'_>> for TypesKey {
    fn from(TypesKeyConverter(name): TypesKeyConverter<'_>) -> Self {
        let mut key = TypesKey::default();

        {
            let mut key = &mut key[..];

            key.put_u8(TYPES_REFERENCE_ID);
            key.put_u64(name.0.0); // Name (Hashed)
        }

        key
    }
}

// -------------------------------------------------------------------------------------------------

// Types Types

type TypesKey = [u8; TYPES_KEY_LEN];
