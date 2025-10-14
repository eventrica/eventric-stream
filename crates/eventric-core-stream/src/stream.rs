use std::{
    error::Error,
    path::Path,
};

use eventric_core_model::{
    event::insertion::Event,
    query::{
        Query,
        QueryItem,
    },
    stream::Position,
};
use eventric_core_persistence::{
    context::Context,
    state::{
        Keyspaces,
        Read,
        Write,
    },
};
use eventric_core_persistence_data as data;
use eventric_core_persistence_index::{
    self as index,
    operation::{
        descriptor,
        tags,
    },
};
use eventric_core_persistence_reference as reference;
use eventric_core_util::iter::{
    and,
    or,
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
    pub fn new<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let context = Context::new(path)?;
        let keyspaces = Keyspaces::new(
            data::configuration::keyspace(&context)?,
            index::configuration::keyspace(&context)?,
            reference::configuration::keyspace(&context)?,
        );

        let len = data::operation::len(&Read::new(&keyspaces))?;
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
                let event = event.into();

                data::operation::insert(&mut write, self.position, &event);
                index::operation::insert(&mut write, self.position, &event);
                reference::operation::insert(&mut write, &event);

                self.position.increment();
            }
        }

        batch.commit()?;

        Ok(())
    }

    pub fn query(&self, position: Option<Position>, query: Query) -> impl Iterator<Item = u64> {
        let read = Read::new(&self.keyspaces);
        let items: Vec<QueryItem> = query.into();

        or::sequential_or(items.into_iter().map(|item| {
            match item {
                QueryItem::Specifiers(specifiers) => or::sequential_or(
                    specifiers
                        .into_iter()
                        .map(|s| descriptor::forward::iterate(&read, position, &s.into())),
                ),
                QueryItem::SpecifiersAndTags(specifiers, tags) => and::sequential_and([
                    or::sequential_or(
                        specifiers
                            .into_iter()
                            .map(|s| descriptor::forward::iterate(&read, position, &s.into())),
                    ),
                    and::sequential_and(
                        tags.into_iter()
                            .map(|t| tags::forward::iterate(&read, position, &t.into())),
                    ),
                ]),
                QueryItem::Tags(tags) => and::sequential_and(
                    tags.into_iter()
                        .map(|t| tags::forward::iterate(&read, position, &t.into())),
                ),
            }
        }))
    }
}

impl Stream {
    pub fn is_empty(&self) -> Result<bool, Box<dyn Error>> {
        data::operation::is_empty(&Read::new(&self.keyspaces))
    }

    pub fn len(&self) -> Result<u64, Box<dyn Error>> {
        data::operation::len(&Read::new(&self.keyspaces))
    }
}
