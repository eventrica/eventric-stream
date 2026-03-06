use std::{
    cmp::Ordering,
    collections::BTreeSet,
    ops::{
        Range,
        RangeFrom,
        RangeTo,
    },
    sync::Exclusive,
};

use derive_more::From;
use fancy_constructor::new;

use crate::{
    event_new::{
        Event,
        Name,
        Tag,
        Version,
    },
    stream_new::{
        Facets,
        Position,
        Result,
        storage::{
            EventsIter,
            Storage,
        },
    },
};

// =================================================================================================
// Select
// =================================================================================================

// Iterator

#[derive(new, Debug)]
#[new(args(iter: EventsIter), vis())]
pub struct Iter {
    #[new(val(Exclusive::new(iter)))]
    iter: Exclusive<EventsIter>,
}

impl DoubleEndedIterator for Iter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next_back()
    }
}

impl Iterator for Iter {
    type Item = Result<Event<Facets, u64>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.get_mut().next()
    }
}

// -------------------------------------------------------------------------------------------------

// Select

pub trait Select {
    fn select(&self, selection: Selection, from: Option<Position>) -> Iter;
    fn select_multiple(&self, selections: Selections, from: Option<Position>);
}

impl Select for Storage {
    fn select(&self, selection: Selection, from: Option<Position>) -> Iter {
        let selection = selection.into_iter().map(Into::into).collect::<Vec<_>>();
        let iter = self.iterate(&selection, from);

        Iter::new(iter)
    }

    fn select_multiple(&self, selections: Selections, from: Option<Position>) {}
}

// -------------------------------------------------------------------------------------------------

// Selector

#[derive(new, Debug)]
pub struct Selector<T>(
    #[new(name(types))] pub(crate) BTreeSet<TypeSelector<T>>,
    #[new(name(tags))] pub(crate) Option<BTreeSet<Tag<T>>>,
);

#[rustfmt::skip]
macro_rules! selector_from {
    ($from:ty, $to:ty) => {
        impl From<Selector<$from>> for Selector<$to> {
            fn from(selector: Selector<$from>) -> Self {
                Self(
                    selector.0.into_iter().map(Into::into).collect(),
                    selector.1.map(|tags| tags.into_iter().map(Into::into).collect()),
                )
            }
        }
    };
}

selector_from!(String, u64);
selector_from!(String, (u64, String));
selector_from!((u64, String), u64);

// -------------------------------------------------------------------------------------------------

// Types

pub type Selection = Vec<Selector<String>>;
pub type Selections = Vec<Selection>;

// -------------------------------------------------------------------------------------------------

// Type Selector

#[derive(new, Debug, Eq, PartialEq)]
pub struct TypeSelector<T>(
    #[new(name(name))] pub(crate) Name<T>,
    #[new(name(version))] pub(crate) Range<Version>,
);

macro_rules! type_selector_from {
    ($from:ty, $to:ty) => {
        impl From<TypeSelector<$from>> for TypeSelector<$to> {
            fn from(selector: TypeSelector<$from>) -> Self {
                Self(selector.0.into(), selector.1)
            }
        }
    };
}

type_selector_from!(String, u64);
type_selector_from!(String, (u64, String));
type_selector_from!((u64, String), u64);

impl<T> Ord for TypeSelector<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.cmp(&other.0) {
            Ordering::Equal => match self.1.start.cmp(&other.1.start) {
                Ordering::Equal => self.1.end.cmp(&other.1.end),
                ordering => ordering,
            },
            ordering => ordering,
        }
    }
}

impl<T> PartialOrd for TypeSelector<T>
where
    T: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// -------------------------------------------------------------------------------------------------

// Version Selector

#[allow(clippy::enum_variant_names)]
#[derive(Debug, From)]
pub enum VersionSelector {
    Range(Range<Version>),
    RangeFrom(RangeFrom<Version>),
    RangeFull,
    RangeTo(RangeTo<Version>),
}

impl From<VersionSelector> for Range<Version> {
    fn from(versions: VersionSelector) -> Self {
        match versions {
            VersionSelector::Range(range) => range,
            VersionSelector::RangeFrom(range) => range.start..Version::MAX,
            VersionSelector::RangeFull => Version::MIN..Version::MAX,
            VersionSelector::RangeTo(range) => Version::MIN..range.end,
        }
    }
}
