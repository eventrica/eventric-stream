use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        Exclusive,
    },
};

use fancy_constructor::new;
use smallvec::SmallVec;

use crate::{
    error::Error,
    event::{
        Event,
        EventHash,
        Identifier,
        Tag,
        identifier::IdentifierHash,
        tag::TagHash,
    },
    stream::{
        data::events::EventHashIter,
        select::{
            Selection,
            Selections,
            event::EventAndMask,
            filter::{
                Filter,
                Matches,
            },
            lookup::Lookup,
            mask::Mask,
            prepared::{
                PreparedGen,
                PreparedSelection,
                PreparedSelections,
            },
        },
    },
};

// =================================================================================================
// Iterator
// =================================================================================================

pub(crate) trait IterDefinition
where
    Self: Into<Self::Prepared>,
    Self::Data: Clone,
    Self::Iter: for<'a> From<(EventHashIter, &'a Self::Prepared)>,
{
    type Data;
    type Iter;
    type Prepared;
}

impl IterDefinition for PreparedGen<Selection> {
    type Data = ();
    type Iter = IterGen<Selection>;
    type Prepared = Self;
}

impl IterDefinition for PreparedGen<Selections> {
    type Data = Arc<SmallVec<[Filter; 8]>>;
    type Iter = IterGen<Selections>;
    type Prepared = Self;
}

impl IterDefinition for Selection {
    type Data = ();
    type Iter = IterGen<Selection>;
    type Prepared = PreparedGen<Self>;
}

impl IterDefinition for Selections {
    type Data = Arc<SmallVec<[Filter; 8]>>;
    type Iter = IterGen<Selections>;
    type Prepared = PreparedGen<Self>;
}

#[derive(new, Debug)]
#[new(const_fn, name(new_inner), vis(pub(crate)))]
pub struct IterGen<T>
where
    T: IterDefinition,
{
    data: T::Data,
    iter: Exclusive<EventHashIter>,
    retrieve: Retrieve,
}

impl<T> From<(EventHashIter, &PreparedGen<T>)> for IterGen<T>
where
    T: IterDefinition,
{
    fn from((iter, prepared): (EventHashIter, &PreparedGen<T>)) -> Self {
        Self::new(iter, prepared)
    }
}

impl<T> IterGen<T>
where
    T: IterDefinition,
{
    pub(crate) fn new(iter: EventHashIter, prepared: &PreparedGen<T>) -> Self {
        let data = prepared.data.clone();
        let iter = Exclusive::new(iter);
        let lookup = prepared.lookup.clone();
        let retrieve = Retrieve::new(lookup);

        Self::new_inner(data, iter, retrieve)
    }
}

/// .
#[derive(new, Debug)]
#[new(const_fn, name(new_inner), vis(pub(crate)))]
pub struct Iter {
    iter: Exclusive<EventHashIter>,
    retrieve: Retrieve,
}

impl Iter {
    pub(crate) fn new(iter: EventHashIter, prepared: &PreparedSelection) -> Self {
        let iter = Exclusive::new(iter);
        let lookup = prepared.lookup.clone();
        let retrieve = Retrieve::new(lookup);

        Self::new_inner(iter, retrieve)
    }

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

/// .
#[derive(new, Debug)]
#[new(const_fn, name(new_inner), vis(pub(crate)))]
pub struct IterMultiple {
    iter: Exclusive<EventHashIter>,
    filters: Arc<SmallVec<[Filter; 8]>>,
    retrieve: Retrieve,
}

impl IterMultiple {
    pub(crate) fn new(iter: EventHashIter, prepared: &PreparedSelections) -> Self {
        let filters = prepared.filters.clone();
        let iter = Exclusive::new(iter);
        let lookup = prepared.lookup.clone();
        let retrieve = Retrieve::new(lookup);

        Self::new_inner(iter, filters, retrieve)
    }

    fn map(&mut self, event: Result<EventHash, Error>) -> <Self as Iterator>::Item {
        event.and_then(|event| {
            let mask = Mask::new(
                self.filters
                    .iter()
                    .map(|filter| filter.matches(&event))
                    .collect(),
            );

            self.retrieve
                .get(event)
                .map(|event| EventAndMask::new(event, mask))
        })
    }
}

impl DoubleEndedIterator for IterMultiple {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next_back().map(|event| self.map(event))
    }
}

impl Iterator for IterMultiple {
    type Item = Result<EventAndMask, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next().map(|event| self.map(event))
    }
}

// -------------------------------------------------------------------------------------------------

// Retrieve

#[derive(new, Debug)]
#[new(const_fn)]
struct Retrieve {
    cache: Arc<Lookup>,
}

impl Retrieve {
    #[rustfmt::skip]
    fn get(&self, event: EventHash) -> Result<Event, Error> {
        let (data, identifier, position, tags, timestamp, version) = event.take();

        self.get_identifier(identifier)
            .map(|identifier| (identifier, self.get_tags(&tags)))
            .map(|(identifier, tags)| Event::new(data, identifier, position, tags, timestamp, version))
    }
}

impl Retrieve {
    fn get_identifier(&self, identifier: IdentifierHash) -> Result<Identifier, Error> {
        self.cache
            .identifiers
            .get(&identifier)
            .cloned()
            .ok_or_else(|| Error::data("identifier not found"))
    }
}

impl Retrieve {
    fn get_tags(&self, tags: &BTreeSet<TagHash>) -> BTreeSet<Tag> {
        tags.iter().filter_map(|tag| self.get_tag(*tag)).collect()
    }

    fn get_tag(&self, tag: TagHash) -> Option<Tag> {
        self.cache.tags.get(&tag).cloned()
    }
}
