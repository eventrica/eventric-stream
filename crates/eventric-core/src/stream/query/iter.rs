use std::sync::Arc;

use derive_more::Debug;
use fancy_constructor::new;

use crate::{
    error::Error,
    event::{
        PersistentEvent,
        PersistentEventHash,
        Position,
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
            events::{
                Events,
                PersistentEventHashIterator,
            },
            indices::PositionIterator,
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

#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub(crate) struct PersistentEventIterator {
    cache: Arc<Cache>,
    iter: CombinedPersistentEventHashIterator,
    options: Option<Options>,
    references: References,
}

// Identifier

impl PersistentEventIterator {
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

// Tags

impl PersistentEventIterator {
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

// Map

impl PersistentEventIterator {
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

impl DoubleEndedIterator for PersistentEventIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|event| self.map(event))
    }
}

impl Iterator for PersistentEventIterator {
    type Item = Result<PersistentEvent, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|event| self.map(event))
    }
}

// -------------------------------------------------------------------------------------------------

// Combined

#[derive(Debug)]
pub(crate) enum CombinedPersistentEventHashIterator {
    Direct(#[debug("Peristent Event Hash Iterator")] PersistentEventHashIterator),
    Mapped(MappedPersistentHashIterator),
}

impl DoubleEndedIterator for CombinedPersistentEventHashIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Direct(iter) => iter.next_back(),
            Self::Mapped(iter) => iter.next_back(),
        }
    }
}

impl Iterator for CombinedPersistentEventHashIterator {
    type Item = Result<PersistentEventHash, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Direct(iter) => iter.next(),
            Self::Mapped(iter) => iter.next(),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Mapped

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct MappedPersistentHashIterator {
    events: Events,
    iter: PositionIterator,
}

impl MappedPersistentHashIterator {
    fn map(&mut self, position: Result<Position, Error>) -> <Self as Iterator>::Item {
        match position {
            Ok(position) => match self.events.get(position) {
                Ok(Some(event)) => Ok(event),
                Ok(None) => Err(Error::data("event not found")),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }
}

impl DoubleEndedIterator for MappedPersistentHashIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|position| self.map(position))
    }
}

impl Iterator for MappedPersistentHashIterator {
    type Item = Result<PersistentEventHash, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|position| self.map(position))
    }
}
