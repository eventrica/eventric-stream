mod append;
mod properties;
mod query;

use std::{
    error::Error,
    path::Path,
};

use derive_more::Debug;
use eventric_core_model::{
    Event,
    Position,
    Query,
    SequencedEventRef,
};
use eventric_core_state::{
    Context,
    Keyspaces,
};
use fancy_constructor::new;

// =================================================================================================
// Stream
// =================================================================================================

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct Stream {
    context: Context,
    keyspaces: Keyspaces,
    position: Position,
}

impl Stream {
    pub fn append<'a, E>(&mut self, events: E) -> Result<(), Box<dyn Error>>
    where
        E: IntoIterator<Item = &'a Event>,
    {
        append::append(&self.context, &self.keyspaces, &mut self.position, events)
    }

    pub fn query<'a>(
        &self,
        position: Option<Position>,
        query: &'a Query,
    ) -> impl Iterator<Item = SequencedEventRef<'a>> {
        query::query(&self.keyspaces, position, query)
    }
}

impl Stream {
    pub fn is_empty(&self) -> Result<bool, Box<dyn Error>> {
        properties::is_empty(&self.keyspaces)
    }

    pub fn len(&self) -> Result<u64, Box<dyn Error>> {
        properties::len(&self.keyspaces)
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
    #[new(default)]
    temporary: Option<bool>,
    path: P,
}

impl<P> StreamConfigurator<P>
where
    P: AsRef<Path>,
{
    pub fn open(self) -> Result<Stream, Box<dyn Error>> {
        let context = Context::new(self.path, self.temporary.unwrap_or_default())?;
        let keyspaces = Keyspaces::new(
            eventric_core_data::keyspace(&context)?,
            eventric_core_index::keyspace(&context)?,
            eventric_core_reference::keyspace(&context)?,
        );
        let position = properties::len(&keyspaces).map(Position::new)?;

        Ok(Stream::new(context, keyspaces, position))
    }
}

impl<P> StreamConfigurator<P>
where
    P: AsRef<Path>,
{
    pub fn temporary(mut self, temporary: bool) -> Self {
        self.temporary = Some(temporary);
        self
    }
}
