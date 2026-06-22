use std::{
    cmp::Ordering,
    collections::BTreeSet,
    ops::{
        Index,
        Range,
        RangeFrom,
        RangeFull,
        RangeTo,
    },
    sync::SyncView,
};

use derive_more::{
    AsRef,
    From,
};
use eventric_utils::validation::Error as ValidationError;
use fancy_constructor::new;
use smallvec::SmallVec;

use crate::{
    event::{
        Event,
        Name,
        Tag,
        Version,
    },
    stream::{
        Metadata,
        Result,
        operate::{
            Condition,
            Selection,
        },
        store::{
            Store,
            StoreIter,
        },
    },
};

// =================================================================================================
// Select
// =================================================================================================

pub trait Select {
    fn select(&self, condition: Condition) -> SelectIter;
}

impl Select for Store {
    fn select(&self, condition: Condition) -> SelectIter {
        let Condition {
            position,
            selections,
        } = condition;

        // The store iterates the coarse union of every selector across every
        // selection (the candidate set); the per-selection mask is then computed
        // for each candidate by `SelectIter`.
        let iter = self.iterate(&selections, position);

        SelectIter::new(iter, selections)
    }
}

// -------------------------------------------------------------------------------------------------

// Select Iterator

#[derive(Debug)]
pub struct SelectIter {
    iter: SyncView<StoreIter>,
    selections: Vec<Selection>,
}

impl SelectIter {
    pub(crate) fn new(iter: StoreIter, selections: Vec<Selection>) -> Self {
        Self {
            iter: SyncView::new(iter),
            selections,
        }
    }
}

impl DoubleEndedIterator for SelectIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        let event = self.iter.as_mut().next_back()?;

        Some(event.map(|event| {
            let mask = mask(&self.selections, &event);

            EventAndMask::new(event, mask)
        }))
    }
}

impl Iterator for SelectIter {
    type Item = Result<EventAndMask>;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.iter.as_mut().next()?;

        Some(event.map(|event| {
            let mask = mask(&self.selections, &event);

            EventAndMask::new(event, mask)
        }))
    }
}

// -------------------------------------------------------------------------------------------------

// Event And Mask

/// A matched event paired with the [`Mask`] of which selections it satisfied.
#[derive(new, Debug)]
#[new(vis(pub(crate)))]
pub struct EventAndMask {
    pub event: Event<Metadata, u64>,
    pub mask: Mask,
}

// -------------------------------------------------------------------------------------------------

// Mask

/// A per-event bitmask over a query's selections: `mask[i]` is `true` iff the
/// `i`-th selection (in the order supplied to the [`Condition`]) matched the
/// event.
#[derive(new, AsRef, Clone, Debug, Eq, PartialEq)]
#[as_ref([bool])]
#[new(vis(pub(crate)))]
pub struct Mask(pub(crate) SmallVec<[bool; 8]>);

impl Index<usize> for Mask {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

// Compute, for a queried event, which of `selections` it satisfies. A selection
// matches if any of its selectors matches; a selector matches when the event's
// type-name equals one of the selector's type-names with the event's version in
// that type's range, AND (if the selector carries tags) all those tags are
// present on the event. This mirrors the index-side matching, re-checked here
// on the hashed (`u64`) representation to recover which selection(s) hit.
fn mask(selections: &[Selection], event: &Event<Metadata, u64>) -> Mask {
    let name = &event.1.0.0;
    let version = event.1.0.1;
    let tags = &event.1.1;

    Mask::new(
        selections
            .iter()
            .map(|selection| {
                selection
                    .selectors
                    .iter()
                    .any(|Selector(types, selector_tags)| {
                        types
                            .iter()
                            .any(|ty| &ty.0 == name && ty.1.contains(&version))
                            && selector_tags
                                .as_ref()
                                .is_none_or(|required| required.is_subset(tags))
                    })
            })
            .collect(),
    )
}

// -------------------------------------------------------------------------------------------------

// Selector

/// A single match clause: an event matches when its type is any of `types` AND
/// (if present) it carries all of `tags`.
#[derive(Debug)]
pub struct Selector<T>(
    pub(crate) BTreeSet<TypeSelector<T>>,
    pub(crate) Option<BTreeSet<Tag<T>>>,
);

impl Selector<String> {
    /// A selector matching events whose type is any of `types`, with no tag
    /// filter.
    pub fn types<I>(types: I) -> Self
    where
        I: IntoIterator<Item = TypeSelector<String>>,
    {
        Self(types.into_iter().collect(), None)
    }

    /// A selector matching events whose type is any of `types` AND which carry
    /// all of `tags`.
    pub fn types_and_tags<I, J>(types: I, tags: J) -> Self
    where
        I: IntoIterator<Item = TypeSelector<String>>,
        J: IntoIterator<Item = Tag<String>>,
    {
        Self(
            types.into_iter().collect(),
            Some(tags.into_iter().collect()),
        )
    }
}

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

// -------------------------------------------------------------------------------------------------

// Type Selector

/// A type-name plus the range of versions to match.
#[derive(Debug, Eq, PartialEq)]
pub struct TypeSelector<T>(pub(crate) Name<T>, pub(crate) Range<Version>);

impl TypeSelector<String> {
    /// Select a type by name, across all versions
    /// (`Version::MIN..Version::MAX`; note the half-open range excludes the
    /// maximum sentinel version).
    pub fn new<N>(name: N) -> Result<Self, ValidationError>
    where
        N: Into<String>,
    {
        Ok(Self(Name::new(name)?, Version::MIN..Version::MAX))
    }

    /// Select a type by name, restricted to a range of versions (accepts
    /// `a..b`, `a..`, `..b`, `..`, or a [`VersionSelector`]).
    pub fn with_versions<N, V>(name: N, versions: V) -> Result<Self, ValidationError>
    where
        N: Into<String>,
        V: Into<VersionSelector>,
    {
        let versions: VersionSelector = versions.into();

        Ok(Self(Name::new(name)?, versions.into()))
    }
}

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

impl From<RangeFull> for VersionSelector {
    fn from(_: RangeFull) -> Self {
        Self::RangeFull
    }
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
