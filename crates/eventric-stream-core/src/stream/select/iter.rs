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
            event::EventAndMask,
            filter::{
                Filter,
                Matches,
            },
            lookup::Lookup,
            mask::Mask,
            prepared::{
                Prepared,
                PreparedMultiple,
            },
        },
    },
};

// =================================================================================================
// Iterator
// =================================================================================================

/// .
#[derive(new, Debug)]
#[new(args(prepared: &Prepared), vis(pub(crate)))]
pub struct Iter {
    iter: Exclusive<EventHashIter>,
    #[new(val(Retrieve::new(prepared.lookup.clone())))]
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

/// .
#[derive(new, Debug)]
#[new(args(prepared: &PreparedMultiple),  vis(pub(crate)))]
pub struct IterMultiple {
    iter: Exclusive<EventHashIter>,
    #[new(val(prepared.filters.clone()))]
    filters: Arc<SmallVec<[Filter; 8]>>,
    #[new(val(Retrieve::new(prepared.lookup.clone())))]
    retrieve: Retrieve,
}

impl IterMultiple {
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
