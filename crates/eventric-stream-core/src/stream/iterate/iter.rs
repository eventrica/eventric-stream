use std::sync::Exclusive;

use derive_more::Debug;
use fancy_constructor::new;

use crate::{
    error::Error,
    event::{
        PersistentEvent,
        PersistentEventHash,
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
            events::PersistentEventHashIterator,
            references::References,
        },
        iterate::options::Options,
        query::filter::{
            Filter,
            Matches as _,
        },
    },
};

// =================================================================================================
// Iterator
// =================================================================================================

// Iterator

/// The [`Iter`] type provides an [`Iterator`]/[`DoubleEndedIterator`]
/// over iteration results for a [`Stream`][stream].
///
/// [stream]: crate::stream::Stream
#[derive(new, Debug)]
#[new(args(options: Options, references: References), const_fn, vis(pub(crate)))]
pub struct Iter {
    #[allow(clippy::struct_field_names)]
    iter: Exclusive<PersistentEventHashIterator>,
    #[new(val(Retrieve::new(options, references)))]
    retrieve: Retrieve,
}

impl Iter {
    fn map(&mut self, event: Result<PersistentEventHash, Error>) -> <Self as Iterator>::Item {
        event.and_then(|event| self.retrieve.get(event))
    }
}

impl DoubleEndedIterator for Iter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next_back().map(|event| self.map(event))
    }
}

impl Iterator for Iter {
    type Item = Result<PersistentEvent, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next().map(|event| self.map(event))
    }
}

// -------------------------------------------------------------------------------------------------

// Iterator (Multiple)

/// The [`IterMulti`] type provides an [`Iterator`]/[`DoubleEndedIterator`]
/// over iteration results for a [`Stream`][stream], along with a matching mask
/// for each result, indicating which of the iteration queries the returned
/// event matches.
///
/// [stream]: crate::stream::Stream
#[derive(new, Debug)]
#[new(args(options: Options, references: References), const_fn, vis(pub(crate)))]
pub struct IterMulti {
    filters: Vec<Filter>,
    #[allow(clippy::struct_field_names)]
    iter: Exclusive<PersistentEventHashIterator>,
    #[new(val(Retrieve::new(options, references)))]
    retrieve: Retrieve,
}

impl IterMulti {
    fn map(&mut self, event: Result<PersistentEventHash, Error>) -> <Self as Iterator>::Item {
        event.and_then(|event| {
            let mask = self
                .filters
                .iter()
                .map(|filter| filter.matches(&event))
                .collect();

            self.retrieve.get(event).map(|event| (event, mask))
        })
    }
}

impl DoubleEndedIterator for IterMulti {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next_back().map(|event| self.map(event))
    }
}

impl Iterator for IterMulti {
    type Item = Result<(PersistentEvent, Vec<bool>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next().map(|event| self.map(event))
    }
}

// -------------------------------------------------------------------------------------------------

// Retrieve

#[derive(new, Debug)]
#[new(const_fn)]
struct Retrieve {
    options: Options,
    references: References,
}

impl Retrieve {
    #[rustfmt::skip]
    fn get(&self, event: PersistentEventHash) -> Result<PersistentEvent, Error> {
        let (data, identifier, position, tags, timestamp, version) = event.take();

        self.get_identifier(identifier)
            .and_then(|identifier| self.get_tags(&tags).map(|tags| (identifier, tags)))
            .map(|(identifier, tags)| PersistentEvent::new(data, identifier, position, tags, timestamp, version))
    }
}

impl Retrieve {
    fn get_identifier(&self, identifier: IdentifierHash) -> Result<Identifier, Error> {
        let identifiers = &self.options.cache.identifiers;

        identifiers
            .entry(identifier.hash())
            .or_try_insert_with(|| self.fetch_identifier(identifier.hash()))
            .map(|entry| entry.value().clone())
    }

    #[rustfmt::skip]
    fn fetch_identifier(&self, hash: u64) -> Result<Identifier, Error> {
        self.references
            .get_identifier(hash)
            .and_then(|identifier| identifier.ok_or_else(|| Error::data(format!("identifier not found: {hash}"))))
    }
}

impl Retrieve {
    fn get_tags(&self, tags: &[TagHash]) -> Result<Vec<Tag>, Error> {
        tags.iter().filter_map(|tag| self.get_tag(*tag)).collect()
    }

    #[rustfmt::skip]
    #[allow(clippy::obfuscated_if_else)]
    fn get_tag(&self, tag: TagHash) -> Option<Result<Tag, Error>> {
        let retrieve_tags = &self.options.retrieve_tags;
        let tags = &self.options.cache.tags;

        retrieve_tags
            .then(|| Some(
                tags.entry(tag.hash())
                    .or_try_insert_with(|| self.fetch_tag(tag.hash()))
                    .map(|entry| entry.value().clone())))
            .unwrap_or_else(||
                tags.get(&tag.hash())
                    .map(|entry| Ok(entry.value().clone())))
    }

    fn fetch_tag(&self, hash: u64) -> Result<Tag, Error> {
        self.references
            .get_tag(hash)
            .and_then(|tag| tag.ok_or_else(|| Error::data(format!("tag not found: {hash}"))))
    }
}
