use std::{
    error::Error,
    path::Path,
};

use derive_more::Debug;
use eventric_core_model::{
    Event,
    Position,
    SequencedEventRef,
};
use fancy_constructor::new;
use fjall::{
    Database,
    Keyspace,
};

use crate::{
    append::{
        self,
        AppendCondition,
    },
    query::{
        self,
        QueryCondition,
    },
};

// =================================================================================================
// Stream
// =================================================================================================

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct Stream {
    #[debug("Database")]
    database: Database,
    keyspaces: StreamKeyspaces,
    position: Position,
}

impl Stream {
    pub fn append<'a, E>(
        &mut self,
        events: E,
        condition: Option<AppendCondition<'a>>,
    ) -> Result<(), Box<dyn Error>>
    where
        E: IntoIterator<Item = &'a Event>,
    {
        let mut batch = self.database.batch();

        append::append(
            &mut batch,
            &self.keyspaces,
            events,
            condition,
            &mut self.position,
        );

        batch.commit()?;

        Ok(())
    }

    pub fn query<'a>(
        &self,
        condition: QueryCondition<'a>,
    ) -> impl Iterator<Item = SequencedEventRef<'a>> {
        query::query(&self.keyspaces, condition)
    }
}

impl Stream {
    pub fn is_empty(&self) -> Result<bool, Box<dyn Error>> {
        eventric_core_data::is_empty(&self.keyspaces.data)
    }

    pub fn len(&self) -> Result<u64, Box<dyn Error>> {
        eventric_core_data::len(&self.keyspaces.data)
    }
}

impl Stream {
    pub fn configure<P>(path: P) -> StreamConfigurator<P>
    where
        P: AsRef<Path>,
    {
        StreamConfigurator::new(path)
    }
}

// -------------------------------------------------------------------------------------------------

// Configurator

#[derive(new, Debug)]
#[new(vis())]
pub struct StreamConfigurator<P>
where
    P: AsRef<Path>,
{
    path: P,
    #[new(default)]
    temporary: Option<bool>,
}

impl<P> StreamConfigurator<P>
where
    P: AsRef<Path>,
{
    pub fn open(self) -> Result<Stream, Box<dyn Error>> {
        let path = self.path;
        let temporary = self.temporary.unwrap_or_default();
        let database = Database::builder(path).temporary(temporary).open()?;

        let keyspaces = StreamKeyspaces::new(
            eventric_core_data::keyspace(&database)?,
            eventric_core_index::keyspace(&database)?,
            eventric_core_reference::keyspace(&database)?,
        );

        let position = eventric_core_data::len(&keyspaces.data).map(Position::new)?;

        Ok(Stream::new(database, keyspaces, position))
    }
}

impl<P> StreamConfigurator<P>
where
    P: AsRef<Path>,
{
    #[must_use]
    pub fn temporary(mut self, temporary: bool) -> Self {
        self.temporary = Some(temporary);
        self
    }
}

// -------------------------------------------------------------------------------------------------

// Keyspaces

#[derive(new, Clone, Debug)]
#[new(const_fn)]
pub struct StreamKeyspaces {
    #[debug("Keyspace(\"{}\")", data.name)]
    pub data: Keyspace,
    #[debug("Keyspace(\"{}\")", index.name)]
    pub index: Keyspace,
    #[debug("Keyspace(\"{}\")", reference.name)]
    pub reference: Keyspace,
}
