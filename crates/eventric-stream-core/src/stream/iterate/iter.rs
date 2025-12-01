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
        iterate::{
            Build,
            cache::Cache,
        },
        select::{
            EventMasked,
            Selection,
            Selections,
            filter::{
                Filter,
                Matches as _,
            },
            mask::Mask,
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

impl Data for () {
    type Data = ();
}

impl Data for Selection {
    type Data = ();
}

impl<const N: usize> Data for Selections<N> {
    type Data = Arc<[Filter; N]>;
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
    iter: Exclusive<EventHashIter>,
    #[new(val(Retrieve::new(cache, fetch_tags, references)))]
    retrieve: Retrieve,
}

// ()

impl Iter<()> {
    fn map(&mut self, event: Result<EventHash, Error>) -> <Self as Iterator>::Item {
        event.and_then(|event| self.retrieve.get(event))
    }
}

impl DoubleEndedIterator for Iter<()> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next_back().map(|event| self.map(event))
    }
}

impl Iterator for Iter<()> {
    type Item = Result<Event, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next().map(|event| self.map(event))
    }
}

// Query

impl Iter<Selection> {
    fn map(&mut self, event: Result<EventHash, Error>) -> <Self as Iterator>::Item {
        event.and_then(|event| self.retrieve.get(event))
    }
}

impl Build<Prepared<Selection>> for Iter<Selection> {
    #[allow(private_interfaces)]
    fn build(iter: EventHashIter, prepared: &Prepared<Selection>, references: References) -> Self {
        let cache = prepared.cache.clone();
        let iter = Exclusive::new(iter);

        Self::new(cache, false, references, (), iter)
    }
}

impl DoubleEndedIterator for Iter<Selection> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next_back().map(|event| self.map(event))
    }
}

impl Iterator for Iter<Selection> {
    type Item = Result<Event, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next().map(|event| self.map(event))
    }
}

// Queries

impl<const N: usize> Iter<Selections<N>> {
    fn map(&mut self, event: Result<EventHash, Error>) -> <Self as Iterator>::Item {
        event.and_then(|event| {
            let mut mask = [false; N];

            for (i, mask) in mask.iter_mut().enumerate().take(N) {
                *mask = self.data[i].matches(&event);
            }

            let mask = Mask::new(mask);

            self.retrieve
                .get(event)
                .map(|event| EventMasked::new(event, mask))
        })
    }
}

impl<const N: usize> Build<Prepared<Selections<N>>> for Iter<Selections<N>> {
    #[allow(private_interfaces)]
    fn build(
        iter: EventHashIter,
        prepared: &Prepared<Selections<N>>,
        references: References,
    ) -> Self {
        let cache = prepared.cache.clone();
        let data = prepared.data.clone();
        let iter = Exclusive::new(iter);

        Self::new(cache, false, references, data, iter)
    }
}

impl<const N: usize> DoubleEndedIterator for Iter<Selections<N>> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next_back().map(|event| self.map(event))
    }
}

impl<const N: usize> Iterator for Iter<Selections<N>> {
    type Item = Result<EventMasked<N>, Error>;

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
            .entry(identifier.hash_val())
            .or_try_insert_with(|| self.fetch_identifier(identifier.hash_val()))
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
    fn get_tags(&self, tags: &BTreeSet<TagHash>) -> Result<BTreeSet<Tag>, Error> {
        tags.iter().filter_map(|tag| self.get_tag(*tag)).collect()
    }

    #[rustfmt::skip]
    fn get_tag(&self, tag: TagHash) -> Option<Result<Tag, Error>> {
        let fetch_tags = &self.fetch_tags;
        let tags = &self.cache.tags;

        fetch_tags
            .then(|| Some(
                tags.entry(tag.hash_val())
                    .or_try_insert_with(|| self.fetch_tag(tag.hash_val()))
                    .map(|entry| entry.value().clone())))
            .unwrap_or_else(||
                tags.get(&tag.hash_val())
                    .map(|entry| Ok(entry.value().clone())))
    }

    fn fetch_tag(&self, hash: u64) -> Result<Tag, Error> {
        self.references
            .get_tag(hash)
            .and_then(|tag| tag.ok_or_else(|| Error::data(format!("tag not found: {hash}"))))
    }
}
