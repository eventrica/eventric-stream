use std::{
    error::Error,
    path::Path,
};

use eventric_core_model::{
    event::insertion::Event,
    stream::Position,
};
use fancy_constructor::new;

use crate::persistence::{
    self,
    Context,
    Keyspaces,
    data,
    operation::{
        Read,
        Write,
    },
};

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
        let context = persistence::context(path)?;
        let keyspaces = persistence::keyspaces(&context)?;

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
                persistence::insert(&mut write, self.position, event);

                self.position.increment();
            }
        }

        batch.commit()?;

        Ok(())
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
