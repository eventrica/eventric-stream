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
#[new(const_fn, name(new_inner), vis())]
pub struct Iter {
    #[allow(clippy::struct_field_names)]
    iter: Exclusive<EventHashIter>,
    retrieve: Retrieve,
}

impl Iter {
    pub(crate) fn new(cache: Arc<Cache>, iter: EventHashIter, references: References) -> Self {
        let iter = Exclusive::new(iter);
        let retrieve = Retrieve::new(cache, true, references);

        Self::new_inner(iter, retrieve)
    }
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
    fetch_tags: bool,
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
        let identifiers = &self.cache.identifiers;

        identifiers
            .entry(identifier)
            .or_try_insert_with(|| self.fetch_identifier(identifier))
            .map(|entry| entry.value().clone())
    }

    #[rustfmt::skip]
    fn fetch_identifier(&self, identifier: IdentifierHash) -> Result<Identifier, Error> {
        self.references
            .get_identifier(identifier)
            .and_then(|identifier| identifier.ok_or_else(|| Error::data("identifier not found")))
    }
}

impl Retrieve {
    fn get_tags(&self, tags: &BTreeSet<TagHash>) -> Result<BTreeSet<Tag>, Error> {
        tags.iter().filter_map(|tag| self.get_tag(*tag)).collect()
    }

    #[rustfmt::skip]
    fn get_tag(&self, tag: TagHash) -> Option<Result<Tag, Error>> {
        let fetch_tags = &self.fetch_tags;
        let tags = &self.cache.tags;

        fetch_tags
            .then(|| Some(
                tags.entry(tag)
                    .or_try_insert_with(|| self.fetch_tag(tag))
                    .map(|entry| entry.value().clone())))
            .unwrap_or_else(||
                tags.get(&tag)
                    .map(|entry| Ok(entry.value().clone())))
    }

    fn fetch_tag(&self, tag: TagHash) -> Result<Tag, Error> {
        self.references
            .get_tag(tag)
            .and_then(|tag| tag.ok_or_else(|| Error::data("tag not found")))
    }
}
