use std::iter;

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
            events::{
                self,
                Events,
            },
            indices::SequentialIterator,
            references::References,
        },
        query::{
            Cache,
            Options,
        },
    },
};

// -------------------------------------------------------------------------------------------------

// Iterator

#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct Iterator<'a> {
    cache: &'a Cache,
    iter: HashIterator<'a>,
    options: Option<Options>,
    references: &'a References,
}

impl Iterator<'_> {
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

impl iter::Iterator for Iterator<'_> {
    type Item = Result<PersistentEvent, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(event)) => {
                let (data, identifier, position, tags, timestamp, version) = event.take();

                Some(
                    self.get_identifier(&identifier)
                        .and_then(|identifier| self.get_tags(&tags).map(|tags| (identifier, tags)))
                        .map(|(identifier, tags)| {
                            PersistentEvent::new(
                                data, identifier, position, tags, timestamp, version,
                            )
                        }),
                )
            }
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }
}

// Hash Iterator

#[derive(Debug)]
pub(crate) enum HashIterator<'a> {
    Direct(events::Iterator<'a>),
    Mapped(MappedHashIterator<'a>),
}

impl iter::Iterator for HashIterator<'_> {
    type Item = Result<PersistentEventHash, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Direct(iter) => iter.next(),
            Self::Mapped(iter) => iter.next(),
        }
    }
}

// Mapped Hash Iterator

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct MappedHashIterator<'a> {
    events: &'a Events,
    iter: SequentialIterator<'a>,
}

impl iter::Iterator for MappedHashIterator<'_> {
    type Item = Result<PersistentEventHash, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(position)) => match self.events.get(position) {
                Ok(Some(event)) => Some(Ok(event)),
                Ok(None) => None,
                Err(err) => Some(Err(err)),
            },
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }
}
