use std::sync::{
    Arc,
    Exclusive,
};

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
        query::{
            Cache,
            Options,
        },
    },
};

// =================================================================================================
// Iterator
// =================================================================================================

// Iterator

/// The [`QueryIterator`] type provides an [`Iterator`]/[`DoubleEndedIterator`]
/// over query results for a [`Stream`][stream].
///
/// [stream]: crate::stream::Stream
#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct QueryIterator {
    cache: Arc<Cache>,
    iter: Exclusive<PersistentEventHashIterator>,
    options: Option<Options>,
    references: References,
}

impl QueryIterator {
    fn get_identifier(&self, identifier: &IdentifierHash) -> Result<Identifier, Error> {
        self.cache
            .identifiers
            .entry(identifier.hash())
            .or_try_insert_with(|| self.get_identifier_from_references(identifier.hash()))
            .map(|entry| entry.value().clone())
    }

    fn get_identifier_from_references(&self, hash: u64) -> Result<Identifier, Error> {
        self.references.get_identifier(hash).and_then(|identifier| {
            identifier.ok_or_else(|| Error::data(format!("identifier not found: {hash}")))
        })
    }
}

impl QueryIterator {
    fn get_tags(&self, tags: &[TagHash]) -> Result<Vec<Tag>, Error> {
        tags.iter().filter_map(|tag| self.get_tag(tag)).collect()
    }

    fn get_tag(&self, tag: &TagHash) -> Option<Result<Tag, Error>> {
        match &self.options {
            Some(options) if options.retrieve_tags => Some(
                self.cache
                    .tags
                    .entry(tag.hash())
                    .or_try_insert_with(|| self.get_tag_from_references(tag.hash()))
                    .map(|entry| entry.value().clone()),
            ),
            _ => self
                .cache
                .tags
                .get(&tag.hash())
                .map(|key_value| Ok(key_value.value().clone())),
        }
    }

    fn get_tag_from_references(&self, hash: u64) -> Result<Tag, Error> {
        self.references
            .get_tag(hash)
            .and_then(|tag| tag.ok_or_else(|| Error::data(format!("tag not found: {hash}"))))
    }
}

impl QueryIterator {
    fn map(&mut self, event: Result<PersistentEventHash, Error>) -> <Self as Iterator>::Item {
        match event {
            Ok(event) => {
                let (data, identifier, position, tags, timestamp, version) = event.take();

                self.get_identifier(&identifier)
                    .and_then(|identifier| self.get_tags(&tags).map(|tags| (identifier, tags)))
                    .map(|(identifier, tags)| {
                        PersistentEvent::new(data, identifier, position, tags, timestamp, version)
                    })
            }
            Err(err) => Err(err),
        }
    }
}

impl DoubleEndedIterator for QueryIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next_back().map(|event| self.map(event))
    }
}

impl Iterator for QueryIterator {
    type Item = Result<PersistentEvent, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next().map(|event| self.map(event))
    }
}
