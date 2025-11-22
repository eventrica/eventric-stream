//! See the `eventric-stream` crate for full documentation, including
//! module-level documentation.

pub(crate) mod filter;

use derive_more::{
    AsRef,
    Debug,
};
use eventric_core::validation::{
    Validate,
    validate,
    vec,
};
use fancy_constructor::new;

use crate::{
    error::Error,
    event::{
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
};

// =================================================================================================
// Query
// =================================================================================================

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
#[derive(new, AsRef, Clone, Debug)]
#[as_ref([Selector])]
#[new(const_fn, name(new_inner), vis())]
pub struct Query {
    /// The set of one or more selectors which make up the overall query.
    pub selectors: Vec<Selector>,
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
    type Err = Error;

    fn validate(self) -> Result<Self, Self::Err> {
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
#[derive(Clone, Debug)]
pub enum Selector {
    /// A [`Selector`] based only on [`Specifier`]s, which will return all
    /// events that match *any* of the supplied [`Specifier`]s.
    Specifiers(Specifiers),
    /// A [`Selector`] which has both [`Specifier`]s and [`Tag`]s, which will
    /// return all events that match match *any* of the supplied [`Specifier`]s
    /// *AND* *all* of the supplied [`Tag`]s.
    SpecifiersAndTags(Specifiers, Tags),
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
    /// of [`Specifier`]s and a collection of [`Tag`]s without constructing
    /// intermediate instances of [`Specifiers`] and [`Tags`] directly.
    ///
    /// # Errors
    ///
    /// Returns an error if the implied [`Specifiers`] or [`Tags`] instances
    /// return an error on construction.
    pub fn specifiers_and_tags<S, T>(specifiers: S, tags: T) -> Result<Self, Error>
    where
        S: Into<Vec<Specifier>>,
        T: Into<Vec<Tag>>,
    {
        Ok(Self::SpecifiersAndTags(
            Specifiers::new(specifiers)?,
            Tags::new(tags)?,
        ))
    }
}

#[derive(Debug)]
pub(crate) enum SelectorHash {
    Specifiers(Vec<SpecifierHash>),
    SpecifiersAndTags(Vec<SpecifierHash>, Vec<TagHash>),
}

impl From<&Selector> for SelectorHash {
    #[rustfmt::skip]
    fn from(selector: &Selector) -> Self {
        match selector {
            Selector::Specifiers(specifiers) => Self::Specifiers(specifiers.into()),
            Selector::SpecifiersAndTags(specifiers, tags) => Self::SpecifiersAndTags(specifiers.into(), tags.into()),
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
        }
    }
}

#[derive(Debug)]
pub(crate) enum SelectorHashRef<'a> {
    Specifiers(Vec<SpecifierHashRef<'a>>),
    SpecifiersAndTags(Vec<SpecifierHashRef<'a>>, Vec<TagHashRef<'a>>),
}

impl<'a> From<&'a Selector> for SelectorHashRef<'a> {
    #[rustfmt::skip]
    fn from(selector: &'a Selector) -> Self {
        match selector {
            Selector::Specifiers(specifiers) => Self::Specifiers(specifiers.into()),
            Selector::SpecifiersAndTags(specifiers, tags) => Self::SpecifiersAndTags(specifiers.into(), tags.into()),
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
#[derive(new, AsRef, Clone, Debug)]
#[as_ref([Specifier])]
#[new(const_fn, name(new_inner), vis())]
pub struct Specifiers {
    /// The collection of one or more [`Specifier`]s which makes up the
    /// [`Specifiers`] collection.
    pub specifiers: Vec<Specifier>,
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
    type Err = Error;

    fn validate(self) -> Result<Self, Self::Err> {
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
#[derive(new, AsRef, Clone, Debug)]
#[as_ref([Tag])]
#[new(const_fn, name(new_inner), vis())]
pub struct Tags {
    /// The collection of one or more [`Tag`]s which makes up the [`Tags`]
    /// collection.
    pub tags: Vec<Tag>,
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
    type Err = Error;

    fn validate(self) -> Result<Self, Self::Err> {
        validate(&self.tags, "tags", &[&vec::IsEmpty])?;

        Ok(self)
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    mod query_tests {
        use assertables::{
            assert_err,
            assert_ok,
        };

        use crate::{
            error::Error,
            event::{
                identifier::Identifier,
                specifier::Specifier,
                tag::Tag,
            },
            stream::query::{
                Query,
                Selector,
            },
        };

        #[test]
        fn new_valid_query_succeeds() -> Result<(), Error> {
            let sel_0 = Selector::specifiers(vec![Specifier::new(Identifier::new("Event1")?)])?;

            assert_ok!(Query::new(vec![sel_0]));

            Ok(())
        }

        #[test]
        fn new_with_multiple_selectors_succeeds() -> Result<(), Error> {
            let sel_0 = Selector::specifiers(vec![Specifier::new(Identifier::new("id_0")?)])?;
            let sel_1 = Selector::specifiers(vec![Specifier::new(Identifier::new("id_1")?)])?;

            assert_ok!(Query::new(vec![sel_0, sel_1]));

            Ok(())
        }

        #[test]
        fn new_with_mixed_selector_types_succeeds() -> Result<(), Error> {
            let sel_0 = Selector::specifiers(vec![Specifier::new(Identifier::new("Event1")?)])?;
            let sel_1 = Selector::specifiers_and_tags(
                vec![Specifier::new(Identifier::new("Event2")?)],
                vec![Tag::new("tag2")?],
            )?;

            assert_ok!(Query::new(vec![sel_0, sel_1]));

            Ok(())
        }

        #[test]
        fn new_empty_query_fails() {
            assert_err!(Query::new(vec![]));
            assert_err!(Query::new(Vec::<Selector>::new()));
        }
    }

    mod specifiers_tests {
        use assertables::{
            assert_err,
            assert_ok,
        };

        use crate::{
            error::Error,
            event::{
                identifier::Identifier,
                specifier::Specifier,
                version::Version,
            },
            stream::query::Specifiers,
        };

        #[test]
        fn new_valid_specifiers_succeeds() -> Result<(), Error> {
            let spec_0 = Specifier::new(Identifier::new("Event1")?);

            assert_ok!(Specifiers::new(vec![spec_0]));

            Ok(())
        }

        #[test]
        fn new_with_multiple_specifiers_succeeds() -> Result<(), Error> {
            let spec_0 = Specifier::new(Identifier::new("Event1")?);
            let spec_1 = Specifier::new(Identifier::new("Event2")?);

            assert_ok!(Specifiers::new(vec![spec_0, spec_1]));

            Ok(())
        }

        #[test]
        #[rustfmt::skip]
        fn new_with_versioned_specifiers_succeeds() -> Result<(), Error> {
            let spec_0 = Specifier::new(Identifier::new("Event1")?).range(Version::new(1)..=Version::new(5));

            assert_ok!(Specifiers::new(vec![spec_0]));

            Ok(())
        }

        #[test]
        #[rustfmt::skip]
        fn new_with_mixed_versioned_and_unversioned_succeeds() -> Result<(), Error> {
            let spec_0 = Specifier::new(Identifier::new("Event1")?);
            let spec_1 = Specifier::new(Identifier::new("Event2")?).range(Version::new(1)..=Version::new(5));
            let spec_2 = Specifier::new(Identifier::new("Event3")?).range(Version::new(10)..);

            assert_ok!(Specifiers::new(vec![spec_0, spec_1, spec_2]));

            Ok(())
        }

        #[test]
        fn new_empty_specifiers_fails() {
            assert_err!(Specifiers::new(vec![]));
            assert_err!(Specifiers::new(Vec::<Specifier>::new()));
        }
    }

    mod tags_tests {
        use assertables::{
            assert_err,
            assert_ok,
        };

        use crate::{
            error::Error,
            event::tag::Tag,
            stream::query::Tags,
        };

        #[test]
        fn new_valid_tags_succeeds() -> Result<(), Error> {
            let tag_0 = Tag::new("tag1")?;

            assert_ok!(Tags::new(vec![tag_0]));

            Ok(())
        }

        #[test]
        fn new_with_multiple_tags_succeeds() -> Result<(), Error> {
            let tag_0 = Tag::new("tag1")?;
            let tag_1 = Tag::new("tag2")?;

            assert_ok!(Tags::new(vec![tag_0, tag_1]));

            Ok(())
        }

        #[test]
        fn new_with_complex_tags_succeeds() -> Result<(), Error> {
            let tag_0 = Tag::new("student:123")?;
            let tag_1 = Tag::new("course:456")?;
            let tag_2 = Tag::new("organization:789")?;

            assert_ok!(Tags::new(vec![tag_0, tag_1, tag_2]));

            Ok(())
        }

        #[test]
        fn new_empty_tags_fails() {
            assert_err!(Tags::new(vec![]));
            assert_err!(Tags::new(Vec::<Tag>::new()));
        }
    }

    mod selector_tests {
        use assertables::{
            assert_err,
            assert_ok,
        };

        use crate::{
            error::Error,
            event::{
                identifier::Identifier,
                specifier::Specifier,
                tag::Tag,
            },
            stream::query::Selector,
        };

        #[test]
        fn selector_specifiers_convenience_method() -> Result<(), Error> {
            let spec_0 = Specifier::new(Identifier::new("Event1")?);

            assert_ok!(Selector::specifiers(vec![spec_0]));

            Ok(())
        }

        #[test]
        fn selector_specifiers_empty_fails() {
            assert_err!(Selector::specifiers(vec![]));
        }

        #[test]
        fn selector_specifiers_and_tags_convenience_method() -> Result<(), Error> {
            let spec_0 = Specifier::new(Identifier::new("Event1")?);
            let tag_0 = Tag::new("tag1")?;

            assert_ok!(Selector::specifiers_and_tags(vec![spec_0], vec![tag_0]));

            Ok(())
        }

        #[test]
        fn selector_specifiers_and_tags_empty_specifiers_fails() -> Result<(), Error> {
            let tag_0 = Tag::new("tag1")?;

            assert_err!(Selector::specifiers_and_tags(vec![], vec![tag_0]));

            Ok(())
        }

        #[test]
        fn selector_specifiers_and_tags_empty_tags_fails() -> Result<(), Error> {
            let spec_0 = Specifier::new(Identifier::new("Event1")?);

            assert_err!(Selector::specifiers_and_tags(vec![spec_0], vec![]));

            Ok(())
        }

        #[test]
        fn selector_specifiers_and_tags_both_empty_fails() {
            assert_err!(Selector::specifiers_and_tags(vec![], vec![]));
        }

        #[test]
        fn selector_with_multiple_specifiers() -> Result<(), Error> {
            let spec_0 = Specifier::new(Identifier::new("Event1")?);
            let spec_1 = Specifier::new(Identifier::new("Event2")?);
            let spec_2 = Specifier::new(Identifier::new("Event3")?);

            assert_ok!(Selector::specifiers(vec![spec_0, spec_1, spec_2]));

            Ok(())
        }

        #[test]
        #[rustfmt::skip]
        fn selector_specifiers_and_tags_with_multiple_each() -> Result<(), Error> {
            let spec_0 = Specifier::new(Identifier::new("Event1")?);
            let spec_1 = Specifier::new(Identifier::new("Event2")?);
            let tag_0 = Tag::new("tag1")?;
            let tag_1 = Tag::new("tag2")?;

            assert_ok!(Selector::specifiers_and_tags(vec![spec_0, spec_1], vec![tag_0, tag_1]));

            Ok(())
        }
    }
}
