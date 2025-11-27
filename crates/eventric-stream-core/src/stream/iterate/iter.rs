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
        Multiple,
        Single,
        data::{
            events::PersistentEventHashIterator,
            references::References,
        },
        iterate::{
            Build,
            cache::Cache,
        },
        query::{
            filter::{
                Filter,
                Matches as _,
            },
            prepared::Prepared,
        },
    },
};

// =================================================================================================
// Iterator
// =================================================================================================

// Data

pub(crate) trait Data {
    type Data;
}

impl Data for Single {
    type Data = ();
}

impl Data for Multiple {
    type Data = Arc<Vec<Filter>>;
}

// -------------------------------------------------------------------------------------------------

// Iterator

/// .
#[allow(private_bounds)]
#[derive(new, Debug)]
#[new(args(cache: Arc<Cache>, fetch_tags: bool, references: References), const_fn, vis(pub(crate)))]
pub struct Iter<T>
where
    T: Data,
{
    data: T::Data,
    #[allow(clippy::struct_field_names)]
    iter: Exclusive<PersistentEventHashIterator>,
    #[new(val(Retrieve::new(cache, fetch_tags, references)))]
    retrieve: Retrieve,
}

// Single

impl Iter<Single> {
    fn map(&mut self, event: Result<PersistentEventHash, Error>) -> <Self as Iterator>::Item {
        event.and_then(|event| self.retrieve.get(event))
    }
}

impl Build<Prepared<Single>> for Iter<Single> {
    #[allow(private_interfaces)]
    fn build(
        optimization: &Prepared<Single>,
        iter: PersistentEventHashIterator,
        references: References,
    ) -> Self {
        let cache = optimization.cache.clone();
        let iter = Exclusive::new(iter);

        Self::new(cache, false, references, (), iter)
    }
}

impl DoubleEndedIterator for Iter<Single> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next_back().map(|event| self.map(event))
    }
}

impl Iterator for Iter<Single> {
    type Item = Result<PersistentEvent, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next().map(|event| self.map(event))
    }
}

// Multiple

impl Iter<Multiple> {
    fn map(&mut self, event: Result<PersistentEventHash, Error>) -> <Self as Iterator>::Item {
        event.and_then(|event| {
            let mask = self
                .data
                .iter()
                .map(|filter| filter.matches(&event))
                .collect();

            self.retrieve.get(event).map(|event| (event, mask))
        })
    }
}

impl Build<Prepared<Multiple>> for Iter<Multiple> {
    #[allow(private_interfaces)]
    fn build(
        optimization: &Prepared<Multiple>,
        iter: PersistentEventHashIterator,
        references: References,
    ) -> Self {
        let cache = optimization.cache.clone();
        let filters = optimization.data.clone();
        let iter = Exclusive::new(iter);

        Self::new(cache, false, references, filters, iter)
    }
}

impl DoubleEndedIterator for Iter<Multiple> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next_back().map(|event| self.map(event))
    }
}

impl Iterator for Iter<Multiple> {
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
    cache: Arc<Cache>,
    fetch_tags: bool,
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
        let identifiers = &self.cache.identifiers;

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
    fn get_tag(&self, tag: TagHash) -> Option<Result<Tag, Error>> {
        let fetch_tags = &self.fetch_tags;
        let tags = &self.cache.tags;

        fetch_tags
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
