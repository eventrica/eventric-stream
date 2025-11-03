//! The [`query`][self] module contains types and functionality related to the
//! [`Stream::query`] operation, such as the [`Cache`], query-specific
//! [`Condition`], and [`Options`] types, as well as the fundamental [`Query`]
//! type and its components.

pub(crate) mod cache;
pub(crate) mod condition;
pub(crate) mod iter;
pub(crate) mod options;

use std::sync::Exclusive;

use derive_more::{
    AsRef,
    Debug,
};
use fancy_constructor::new;

use crate::{
    error::Error,
    event::{
        position::Position,
        specifier::{
            Specifier,
            SpecifierHash,
            SpecifierHashRef,
        },
        tag::{
            Tag,
            TagHash,
            TagHashRef,
        },
    },
    stream::{
        Stream,
        data::events::{
            MappedPersistentEventHashIterator,
            PersistentEventHashIterator,
        },
    },
    utils::validation::{
        Validate,
        validate,
        vec,
    },
};

// =================================================================================================
// Query
// =================================================================================================

impl Stream {
    /// Queries the [`Stream`] based on the supplied [`Condition`], using the
    /// [`Cache`] to avoid re-fetching intermediate components such as
    /// [`Identifier`][identifier]s and [`Tag`]s, and optionally configured by
    /// [`Options`] to determine which event data is returned.
    ///
    /// TODO: [Full query documentation + examples][issue]
    ///
    /// # Errors
    ///
    /// Returns an error in the case of an underlying IO/database error.
    ///
    /// [identifier]: crate::event::Identifier
    /// [issue]: https://github.com/eventrica/eventric-stream/issues/21
    #[must_use]
    pub fn query(&self, condition: &Condition<'_>, options: Option<Options>) -> QueryIterator {
        let options = options.unwrap_or_default();
        let references = self.data.references.clone();

        let iter = condition.matches.map_or_else(
            || self.query_events(condition.from),
            |query| {
                let query_hash_ref: QueryHashRef<'_> = query.into();
                let query_hash: QueryHash = (&query_hash_ref).into();

                options.cache.populate(&query_hash_ref);

                self.query_indices(&query_hash, condition.from)
            },
        );
        let iter = Exclusive::new(iter);

        QueryIterator::new(iter, options, references)
    }

    fn query_events(&self, from: Option<Position>) -> PersistentEventHashIterator {
        let iter = self.data.events.iterate(from);

        PersistentEventHashIterator::Direct(iter)
    }

    fn query_indices(
        &self,
        query: &QueryHash,
        from: Option<Position>,
    ) -> PersistentEventHashIterator {
        let events = self.data.events.clone();
        let iter = self.data.indices.query(query, from);
        let iter = MappedPersistentEventHashIterator::new(events, iter);

        PersistentEventHashIterator::Mapped(iter)
    }
}

// -------------------------------------------------------------------------------------------------

// Query

/// The [`Query`] type is the primary type when interacting with a [`Stream`],
/// being used both directly in query [`Condition`] to determine the events to
/// return, but also as part of an [`append::Condition`][append] (where one is
/// supplied) to ensure appropriate concurrency control during a conditional
/// append operation.
///
/// A query is made up of one or more [`Selector`]s, where the events returned
/// will be those that match **ANY** of the supplied selectors. For more
/// information on how selectors are matched to events, see the documentation
/// for the [`Selector`] type.
///
/// [append]: crate::stream::append::Condition
#[derive(new, AsRef, Debug)]
#[as_ref([Selector])]
#[new(const_fn, name(new_inner), vis())]
pub struct Query {
    selectors: Vec<Selector>,
}

impl Query {
    /// Constructs a new [`Query`] given a value which can be converted into an
    /// iterator of [`Selector`] instances.
    ///
    /// # Errors
    ///
    /// Returns an error on validation failure. The supplied collection of
    /// selectors must conform to the following constraints:
    /// - Min 1 Selector (Non-Zero Length/Non-Empty)
    pub fn new<S>(selectors: S) -> Result<Self, Error>
    where
        S: IntoIterator<Item = Selector>,
    {
        Self::new_unvalidated(selectors.into_iter().collect()).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated(selectors: Vec<Selector>) -> Self {
        Self::new_inner(selectors)
    }
}

impl From<Query> for Vec<Selector> {
    fn from(query: Query) -> Self {
        query.selectors
    }
}

impl Validate for Query {
    fn validate(self) -> Result<Self, Error> {
        validate(&self.selectors, "selectors", &[&vec::IsEmpty])?;

        Ok(self)
    }
}

// Hash

#[derive(new, AsRef, Debug)]
#[as_ref([SelectorHash])]
pub(crate) struct QueryHash(Vec<SelectorHash>);

impl From<Query> for QueryHash {
    fn from(query: Query) -> Self {
        (&query).into()
    }
}

impl From<&Query> for QueryHash {
    fn from(query: &Query) -> Self {
        Self::new(query.as_ref().iter().map(Into::into).collect::<Vec<_>>())
    }
}

impl From<QueryHashRef<'_>> for QueryHash {
    fn from(query: QueryHashRef<'_>) -> Self {
        (&query).into()
    }
}

impl From<&QueryHashRef<'_>> for QueryHash {
    fn from(query: &QueryHashRef<'_>) -> Self {
        Self::new(query.as_ref().iter().map(Into::into).collect::<Vec<_>>())
    }
}

// Hash Ref

#[derive(new, AsRef, Debug)]
#[as_ref([SelectorHashRef<'a>])]
pub(crate) struct QueryHashRef<'a>(Vec<SelectorHashRef<'a>>);

impl<'a> From<&'a Query> for QueryHashRef<'a> {
    fn from(query: &'a Query) -> Self {
        Self::new(query.as_ref().iter().map(Into::into).collect::<Vec<_>>())
    }
}

// -------------------------------------------------------------------------------------------------

// Selector

/// The [`Selector`] type is the functional core of a [`Query`], which contains
/// one or more [`Selector`] instances. A query will return all events matched
/// by *any* of the [`Selector`] instances (they are effectively combined as a
/// logical OR operation).
///
/// Each variant of the [`Selector`] has a different meaning.
#[derive(Debug)]
pub enum Selector {
    /// A [`Selector`] based only on [`Specifier`]s, which will return all
    /// events that match *any* of the supplied [`Specifier`]s.
    Specifiers(Specifiers),
    /// A [`Selector`] which has both [`Specifier`]s and [`Tag`]s, which will
    /// return all events that match match *any* of the supplied [`Specifier`]s
    /// *AND* *all* of the supplied [`Tag`]s.
    SpecifiersAndTags(Specifiers, Tags),
    /// A [`Selector`] based only on [`Tag`]s, which will return all events that
    /// match *all* of the supplied [`Tag`]s.
    Tags(Tags),
}

impl Selector {
    /// Convenience function for creating a selector directly from a collection
    /// of [`Specifier`]s without constructing an intermediate [`Specifiers`]
    /// instance directly.
    ///
    /// # Errors
    ///
    /// Returns an error if the implied [`Specifiers`] instance returns an error
    /// on construction.
    pub fn specifiers<S>(specifiers: S) -> Result<Self, Error>
    where
        S: Into<Vec<Specifier>>,
    {
        Ok(Self::Specifiers(Specifiers::new(specifiers)?))
    }

    /// Convenience function for creating a selector directly from a collection
    /// of [`Tag`]s without constructing an intermediate [`Tags`]
    /// instance directly.
    ///
    /// # Errors
    ///
    /// Returns an error if the implied [`Tags`] instance returns an error
    /// on construction.
    pub fn tags<T>(tags: T) -> Result<Self, Error>
    where
        T: Into<Vec<Tag>>,
    {
        Ok(Self::Tags(Tags::new(tags)?))
    }
}

#[derive(Debug)]
pub(crate) enum SelectorHash {
    Specifiers(Vec<SpecifierHash>),
    SpecifiersAndTags(Vec<SpecifierHash>, Vec<TagHash>),
    Tags(Vec<TagHash>),
}

impl From<&Selector> for SelectorHash {
    #[rustfmt::skip]
    fn from(selector: &Selector) -> Self {
        match selector {
            Selector::Specifiers(specifiers) => Self::Specifiers(specifiers.into()),
            Selector::SpecifiersAndTags(specifiers, tags) => Self::SpecifiersAndTags(specifiers.into(), tags.into()),
            Selector::Tags(tags) => Self::Tags(tags.into()),
        }
    }
}

impl From<&SelectorHashRef<'_>> for SelectorHash {
    #[rustfmt::skip]
    fn from(selector: &SelectorHashRef<'_>) -> Self {
        match selector {
            SelectorHashRef::Specifiers(specifiers) => {
                Self::Specifiers(specifiers.iter().map(Into::into).collect())
            }
            SelectorHashRef::SpecifiersAndTags(specifiers, tags) => {
                Self::SpecifiersAndTags(specifiers.iter().map(Into::into).collect(), tags.iter().map(Into::into).collect())
            }
            SelectorHashRef::Tags(tags) => {
                Self::Tags(tags.iter().map(Into::into).collect())
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum SelectorHashRef<'a> {
    Specifiers(Vec<SpecifierHashRef<'a>>),
    SpecifiersAndTags(Vec<SpecifierHashRef<'a>>, Vec<TagHashRef<'a>>),
    Tags(Vec<TagHashRef<'a>>),
}

impl<'a> From<&'a Selector> for SelectorHashRef<'a> {
    #[rustfmt::skip]
    fn from(selector: &'a Selector) -> Self {
        match selector {
            Selector::Specifiers(specifiers) => Self::Specifiers(specifiers.into()),
            Selector::SpecifiersAndTags(specifiers, tags) => Self::SpecifiersAndTags(specifiers.into(), tags.into()),
            Selector::Tags(tags) => Self::Tags(tags.into()),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Specifiers

/// The [`Specifiers`] type is a validating collection of [`Specifier`]
/// instances, used to ensure that invariants are met when constructing queries.
///
/// When used within a [`Selector`] (of whatever variant), the [`Specifier`]
/// instances within a [`Specifiers`] collection are always combined as a
/// logical OR operation, so events that match *any* of the supplied
/// [`Specifier`] instances will be returned.
#[derive(new, AsRef, Debug)]
#[as_ref([Specifier])]
#[new(const_fn, name(new_inner), vis())]
pub struct Specifiers {
    specifiers: Vec<Specifier>,
}

impl Specifiers {
    /// Constructs a new [`Specifiers`] instance given any value which can be
    /// converted into a valid [`Vec`] of [`Specifier`] instances.
    ///
    /// # Errors
    ///
    /// Returns an error on validation failure. Specifiers must conform to the
    /// following constraints:
    /// - Min 1 Specifier (Non-Zero Length/Non-Empty)
    pub fn new<S>(specifiers: S) -> Result<Self, Error>
    where
        S: Into<Vec<Specifier>>,
    {
        Self::new_unvalidated(specifiers.into()).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated(specifiers: Vec<Specifier>) -> Self {
        Self::new_inner(specifiers)
    }
}

impl From<&Specifiers> for Vec<SpecifierHash> {
    fn from(specifiers: &Specifiers) -> Self {
        specifiers.as_ref().iter().map(Into::into).collect()
    }
}

impl<'a> From<&'a Specifiers> for Vec<SpecifierHashRef<'a>> {
    fn from(specifiers: &'a Specifiers) -> Self {
        specifiers.as_ref().iter().map(Into::into).collect()
    }
}

impl Validate for Specifiers {
    fn validate(self) -> Result<Self, Error> {
        validate(&self.specifiers, "specifiers", &[&vec::IsEmpty])?;

        Ok(self)
    }
}

// -------------------------------------------------------------------------------------------------

// Tags

/// The [`Tags`] type is a validating collection of [`Tag`] instances, used to
/// ensure that invariants are met when constructing queries.
///
/// When used within a [`Selector`] (of whatever variant), the [`Tag`]
/// instances within a [`Tags`] collection are always combined as a
/// logical AND operation, so *only* events that match *all* of the supplied
/// [`Tag`] instances will be returned.
#[derive(new, AsRef, Debug)]
#[as_ref([Tag])]
#[new(const_fn, name(new_inner), vis())]
pub struct Tags {
    tags: Vec<Tag>,
}

impl Tags {
    /// Constructs a new [`Tags`] instance given any value which can be
    /// converted into a valid [`Vec`] of [`Tag`] instances.
    ///
    /// # Errors
    ///
    /// Returns an error on validation failure. Tags must conform to the
    /// following constraints:
    /// - Min 1 Tag (Non-Zero Length/Non-Empty)
    pub fn new<T>(tags: T) -> Result<Self, Error>
    where
        T: Into<Vec<Tag>>,
    {
        Self::new_unvalidated(tags.into()).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated(tags: Vec<Tag>) -> Self {
        Self::new_inner(tags)
    }
}

impl From<&Tags> for Vec<TagHash> {
    fn from(tags: &Tags) -> Self {
        tags.as_ref().iter().map(Into::into).collect()
    }
}

impl<'a> From<&'a Tags> for Vec<TagHashRef<'a>> {
    fn from(tags: &'a Tags) -> Self {
        tags.as_ref().iter().map(Into::into).collect()
    }
}

impl Validate for Tags {
    fn validate(self) -> Result<Self, Error> {
        validate(&self.tags, "tags", &[&vec::IsEmpty])?;

        Ok(self)
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Export

pub use self::{
    cache::Cache,
    condition::Condition,
    iter::QueryIterator,
    options::Options,
};
