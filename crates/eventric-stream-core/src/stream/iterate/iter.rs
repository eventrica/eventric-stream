use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        Exclusive,
    },
};

use derive_more::Debug;
use fancy_constructor::new;

use crate::{
    error::Error,
    event::{
        Event,
        EventHash,
        identifier::{
            Identifier,
            IdentifierHash,
        },
        tag::{
            Tag,
            TagHash,
        },
    },
    stream::{
        data::{
            events::EventHashIter,
            references::References,
        },
        iterate::cache::Cache,
    },
};

// =================================================================================================
// Iterator
// =================================================================================================

// Iterator

/// .
#[derive(new, Debug)]
#[new(args(cache: Arc<Cache>, references: References), vis(pub(crate)))]
pub struct Iter {
    #[allow(clippy::struct_field_names)]
    iter: Exclusive<EventHashIter>,
    #[new(val(Retrieve::new(cache, references)))]
    retrieve: Retrieve,
}

impl Iter {
    fn map(&mut self, event: Result<EventHash, Error>) -> <Self as Iterator>::Item {
        event.and_then(|event| self.retrieve.get(event))
    }
}

impl DoubleEndedIterator for Iter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next_back().map(|event| self.map(event))
    }
}

impl Iterator for Iter {
    type Item = Result<Event, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next().map(|event| self.map(event))
    }
}

// -------------------------------------------------------------------------------------------------

// Retrieve

#[derive(new, Debug)]
#[new(const_fn)]
struct Retrieve {
    cache: Arc<Cache>,
    references: References,
}

impl Retrieve {
    #[rustfmt::skip]
    fn get(&self, event: EventHash) -> Result<Event, Error> {
        let (data, identifier, position, tags, timestamp, version) = event.take();

        self.get_identifier(identifier)
            .and_then(|identifier| self.get_tags(&tags).map(|tags| (identifier, tags)))
            .map(|(identifier, tags)| Event::new(data, identifier, position, tags, timestamp, version))
    }
}

impl Retrieve {
    fn get_identifier(&self, identifier: IdentifierHash) -> Result<Identifier, Error> {
        self.cache
            .identifiers
            .entry(identifier)
            .or_try_insert_with(|| self.fetch_identifier(identifier))
            .map(|entry| entry.value().clone())
    }

    fn fetch_identifier(&self, identifier: IdentifierHash) -> Result<Identifier, Error> {
        self.references
            .get_identifier(identifier)
            .and_then(|identifier| {
                identifier
                    .ok_or_else(|| Error::general("Iter/Fetch Identifier/Identifier Not Found"))
            })
    }
}

impl Retrieve {
    fn get_tags(&self, tags: &BTreeSet<TagHash>) -> Result<BTreeSet<Tag>, Error> {
        tags.iter().map(|tag| self.get_tag(*tag)).collect()
    }

    fn get_tag(&self, tag: TagHash) -> Result<Tag, Error> {
        self.cache
            .tags
            .entry(tag)
            .or_try_insert_with(|| self.fetch_tag(tag))
            .map(|entry| entry.value().clone())
    }

    fn fetch_tag(&self, tag: TagHash) -> Result<Tag, Error> {
        self.references
            .get_tag(tag)
            .and_then(|tag| tag.ok_or_else(|| Error::general("Iter/Fetch Tag/Tag Not Found")))
    }
}
