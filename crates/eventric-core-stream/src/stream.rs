use std::{
    error::Error,
    path::Path,
};

use eventric_core_model::{
    Event,
    Position,
    Query,
};
use eventric_core_persistence::{
    Context,
    Keyspaces,
    Read,
    Write,
};
use eventric_core_persistence_data as data;
use eventric_core_persistence_index as index;
use eventric_core_persistence_reference as reference;
use fancy_constructor::new;

// =================================================================================================
// Model
// =================================================================================================

#[derive(new, Debug)]
#[new(name(inner_new), vis())]
pub struct Stream {
    context: Context,
    keyspaces: Keyspaces,
    position: Position,
}

impl Stream {
    pub fn new<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let context = Context::new(path)?;
        let keyspaces = Keyspaces::new(
            data::keyspace(&context)?,
            index::keyspace(&context)?,
            reference::keyspace(&context)?,
        );

        let len = data::len(&Read::new(&keyspaces))?;
        let position = len.into();

        Ok(Self::inner_new(context, keyspaces, position))
    }
}

impl Stream {
    pub fn append<E>(&mut self, events: E) -> Result<(), Box<dyn Error>>
    where
        E: IntoIterator<Item = Event>,
    {
        let mut batch = self.context.as_ref().batch();

        {
            let mut write = Write::new(&mut batch, &self.keyspaces);

            for event in events {
                let event = (&event).into();

                data::insert(&mut write, self.position, &event);
                index::insert(&mut write, self.position, &event);
                reference::insert(&mut write, &event);

                self.position.increment();
            }
        }

        batch.commit()?;

        Ok(())
    }

    pub fn query(&self, position: Option<Position>, query: &Query) -> impl Iterator<Item = u64> {
        let read = Read::new(&self.keyspaces);
        let query = query.into();

        index::query(&read, position, &query)
    }
}

impl Stream {
    pub fn is_empty(&self) -> Result<bool, Box<dyn Error>> {
        data::is_empty(&Read::new(&self.keyspaces))
    }

    pub fn len(&self) -> Result<u64, Box<dyn Error>> {
        data::len(&Read::new(&self.keyspaces))
    }
}
