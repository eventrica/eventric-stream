//! The masked read query: lowering a [`Condition`] to a [`SelectIter`] of
//! [`EventAndMask`]s, and the selector vocabulary (`Selector`, `TypeSelector`,
//! `VersionSelector`) used to build queries.

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
use fancy_constructor::new;
use smallvec::SmallVec;

use crate::{
    error::Result,
    event::{
        Event,
        Name,
        Tag,
        Version,
    },
    stream::{
        Metadata,
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

/// The read side of a stream: run a [`Condition`] as a masked query.
pub trait Select {
    /// Run `condition` as a query, yielding each matching event paired with the
    /// [`Mask`] of which selections it satisfied.
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

/// A lazy, double-ended iterator over the events matching a query, each paired
/// with its per-selection [`Mask`].
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
    /// The matched (persisted) event.
    pub event: Event<Metadata, u64>,
    /// Which of the query's selections this event satisfied.
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
    let name = event.facets().ty().name();
    let version = event.facets().ty().version();
    let tags = event.facets().tags();

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
    pub fn new<N>(name: N) -> Result<Self>
    where
        N: Into<String>,
    {
        Ok(Self(Name::new(name)?, Version::MIN..Version::MAX))
    }

    /// Select a type by name, restricted to a range of versions (accepts
    /// `a..b`, `a..`, `..b`, `..`, or a [`VersionSelector`]).
    pub fn with_versions<N, V>(name: N, versions: V) -> Result<Self>
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

/// The range of versions a [`TypeSelector`] matches, adapted from the standard
/// range syntaxes (`a..b`, `a..`, `..`, `..b`).
#[allow(clippy::enum_variant_names)]
#[derive(Debug, From)]
pub enum VersionSelector {
    /// A bounded range, `a..b`.
    Range(Range<Version>),
    /// An unbounded-above range, `a..` (extends to `Version::MAX`).
    RangeFrom(RangeFrom<Version>),
    /// The full range, `..` (`Version::MIN..Version::MAX`).
    RangeFull,
    /// An unbounded-below range, `..b` (starts at `Version::MIN`).
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

// =================================================================================================
// Tests
// =================================================================================================

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::VersionSelector;
    use crate::event::Version;

    fn lower(selector: VersionSelector) -> Range<Version> {
        selector.into()
    }

    #[test]
    fn lowers_a_bounded_range() {
        assert_eq!(
            lower((Version::new(1)..Version::new(3)).into()),
            Version::new(1)..Version::new(3),
        );
    }

    #[test]
    fn lowers_range_from_to_max() {
        assert_eq!(
            lower((Version::new(2)..).into()),
            Version::new(2)..Version::MAX
        );
    }

    #[test]
    fn lowers_range_to_from_min() {
        assert_eq!(
            lower((..Version::new(4)).into()),
            Version::MIN..Version::new(4)
        );
    }

    #[test]
    fn lowers_the_full_range_to_min_max() {
        assert_eq!(lower((..).into()), Version::MIN..Version::MAX);
    }

    // The full/default range is half-open at `Version::MAX`, so the 255 sentinel is
    // unmatchable: a v255 event can be appended but never selected. Pinned as a
    // known limitation (tied to the version-as-selection question —
    // versioning.md §7.1).
    #[test]
    fn the_max_version_sentinel_is_unmatchable() {
        let full: Range<Version> = VersionSelector::RangeFull.into();

        assert!(!full.contains(&Version::MAX));
        assert!(full.contains(&Version::new(254)));
    }
}
