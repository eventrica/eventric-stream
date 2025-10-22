use std::error::Error;

use bytes::Buf as _;
use derive_more::Debug;
use eventric_core_model::{
    EventHashRef,
    Position,
    SequencedEventHash,
    Timestamp,
};
use fancy_constructor::new;
use fjall::{
    Database,
    Keyspace,
    KeyspaceCreateOptions,
    WriteBatch,
};

// =================================================================================================
// Data
// =================================================================================================

// Configuration

static KEYSPACE_NAME: &str = "data";

// -------------------------------------------------------------------------------------------------

// Data

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct Data {
    #[debug("Keyspace(\"{}\")", data.name)]
    data: Keyspace,
}

impl Data {
    pub fn open(database: &Database) -> Result<Self, Box<dyn Error>> {
        let data = database.keyspace(KEYSPACE_NAME, KeyspaceCreateOptions::default())?;

        Ok(Self::new(data))
    }
}

// Get

impl Data {
    pub fn get(&self, position: Position) -> Result<Option<SequencedEventHash>, Box<dyn Error>> {
        crate::operation::get(&self.data, position)
    }
}

// Put

impl Data {
    pub fn put(
        &self,
        batch: &mut WriteBatch,
        event: &EventHashRef<'_>,
        position: Position,
        timestamp: Timestamp,
    ) {
        crate::operation::insert(batch, &self.data, event, position, timestamp);
    }
}

// Properties

impl Data {
    pub fn is_empty(&self) -> Result<bool, Box<dyn Error>> {
        self.len().map(|len| len == 0)
    }

    pub fn len(&self) -> Result<u64, Box<dyn Error>> {
        let key_value = self.data.last_key_value()?;

        if let Some((key, _)) = key_value {
            let key = key.as_ref().get_u64();
            let len = key + 1;

            Ok(len)
        } else {
            Ok(0)
        }
    }
}
