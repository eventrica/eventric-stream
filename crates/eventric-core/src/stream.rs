use std::{
    error::Error,
    path::Path,
};

use eventric_core_model::{
    Event,
    Position,
    Query,
    SequencedEventHash,
};
use eventric_core_state::{
    Context,
    Keyspaces,
    Read,
    Write,
};
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
    pub fn new<P>(path: P, temporary: bool) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let context = Context::new(path, temporary)?;
        let keyspaces = Keyspaces::new(
            eventric_core_data::keyspace(&context)?,
            eventric_core_index::keyspace(&context)?,
            eventric_core_reference::keyspace(&context)?,
        );

        let len = eventric_core_data::len(&Read::new(&keyspaces))?;
        let position = Position::new(len);

        Ok(Self::inner_new(context, keyspaces, position))
    }
}

impl Stream {
    pub fn append<'a, E>(&mut self, events: E) -> Result<(), Box<dyn Error>>
    where
        E: IntoIterator<Item = &'a Event>,
    {
        let mut batch = self.context.as_ref().batch();

        {
            let mut write = Write::new(&mut batch, &self.keyspaces);

            for event in events {
                let event = event.into();

                eventric_core_data::insert(&mut write, self.position, &event);
                eventric_core_index::insert(&mut write, self.position, &event);
                eventric_core_reference::insert(&mut write, &event);

                self.position.increment();
            }
        }

        batch.commit()?;

        Ok(())
    }

    pub fn query(
        &self,
        position: Option<Position>,
        query: &Query,
    ) -> impl Iterator<Item = SequencedEventHash> {
        let read = Read::new(&self.keyspaces);
        let query = query.into();

        eventric_core_index::query(&read, position, &query)
            .map(Position::new)
            .map(move |position| {
                eventric_core_data::get(&read, position)
                    .expect("data get error")
                    .expect("data not found error")
            })
    }
}

impl Stream {
    pub fn is_empty(&self) -> Result<bool, Box<dyn Error>> {
        eventric_core_data::is_empty(&Read::new(&self.keyspaces))
    }

    pub fn len(&self) -> Result<u64, Box<dyn Error>> {
        eventric_core_data::len(&Read::new(&self.keyspaces))
    }
}
