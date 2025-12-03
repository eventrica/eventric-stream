//! See the `eventric-stream` crate for full documentation, including
//! module-level documentation.

pub(crate) mod event;
pub(crate) mod filter;
pub(crate) mod mask;
pub(crate) mod prepared;
pub(crate) mod selector;
pub(crate) mod source;

// use std::borrow::Cow;

use derive_more::{
    AsRef,
    Debug,
};
use eventric_utils::validation::{
    Validate,
    validate,
    vec,
};
use fancy_constructor::new;

use crate::{
    error::Error,
    stream::{
        iterate::iter::Iter,
        select::selector::{
            SelectorHash,
            SelectorHashAndValue,
        },
    },
};

// =================================================================================================
// Select
// =================================================================================================

/// The [`Selection`] type a key type when interacting with a [`Stream`],
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
pub struct Selection {
    pub(crate) selectors: Vec<Selector>,
}

impl Selection {
    /// Constructs a new [`Selection`] given a value which can be converted into
    /// an iterator of [`Selector`] instances.
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

impl From<Selection> for Vec<Selector> {
    fn from(selection: Selection) -> Self {
        selection.selectors
    }
}

impl Source for Selection {
    type Iterator = Iter<Selection>;
    type Prepared = Prepared<Selection>;

    fn prepare(self) -> Self::Prepared {
        self.into()
    }
}

impl Validate for Selection {
    type Err = Error;

    fn validate(self) -> Result<Self, Self::Err> {
        validate(&self.selectors, "selectors", &[&vec::IsEmpty])?;

        Ok(self)
    }
}

// Hash

/// The [`SelectionHash`] type is the optimized form of a [`Query`] which has
/// been used as part of an [`Iterate`][iterate] or
/// [`IterateMulti`][iterate_multi] operation. This can be used as part of a
/// conditional [`Append`][append] operation, and is more efficient than
/// supplying a complete [`Selection`].
///
/// [append]: crate::stream::append::Append
/// [iterate]: crate::stream::iterate::Iterate
/// [iterate_multi]: crate::stream::iterate::IterateMulti
#[derive(new, AsRef, Clone, Debug)]
#[as_ref([SelectorHash])]
pub struct SelectionHash(pub(crate) Vec<SelectorHash>);

impl From<Selection> for SelectionHash {
    fn from(selection: Selection) -> Self {
        Self(selection.selectors.into_iter().map(Into::into).collect())
    }
}

impl From<SelectionHashAndValue> for SelectionHash {
    fn from(selection: SelectionHashAndValue) -> Self {
        Self(selection.0.into_iter().map(Into::into).collect())
    }
}

// impl From<&Selection> for SelectionHash {
//     fn from(selection: &Selection) -> Self {
//         Self::new(selection.as_ref().iter().map(Into::into).collect())
//     }
// }

// impl From<SelectionHashRef<'_>> for SelectionHash {
//     fn from(selection: SelectionHashRef<'_>) -> Self {
//         (&selection).into()
//     }
// }

// impl From<&SelectionHashRef<'_>> for SelectionHash {
//     fn from(selection: &SelectionHashRef<'_>) -> Self {
//         Self::new(selection.as_ref().iter().map(Into::into).collect())
//     }
// }

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct SelectionHashAndValue(pub(crate) Vec<SelectorHashAndValue>);

impl From<Selection> for SelectionHashAndValue {
    fn from(selection: Selection) -> Self {
        Self(selection.selectors.into_iter().map(Into::into).collect())
    }
}

// Hash Ref

// #[derive(new, AsRef, Debug)]
// #[as_ref([SelectorHashRef<'a>])]
// pub(crate) struct SelectionHashRef<'a>(pub(crate) Vec<SelectorHashRef<'a>>);

// impl<'a> From<&'a Selection> for SelectionHashRef<'a> {
//     fn from(selection: &'a Selection) -> Self {
//         Self::new(selection.as_ref().iter().map(Into::into).collect())
//     }
// }

// -------------------------------------------------------------------------------------------------

// Selections

/// The [`Selections`] type is a validating collection of [`Query`]
/// instances, used to ensure that invariants are met when constructing queries.
#[derive(new, Clone, Debug)]
#[new(const_fn, name(new_inner), vis())]
pub struct Selections(pub(crate) Vec<Selection>);

impl Selections {
    /// Constructs a new [`Selections`] instance given an array of [`Selection`]
    /// instances.
    ///
    /// # Errors
    ///
    /// Returns an error on validation failure. Selections must conform to the
    /// following constraints:
    /// - Min 1 Selection (Non-Zero Length/Non-Empty)
    pub fn new<S>(selections: S) -> Result<Self, Error>
    where
        S: IntoIterator<Item = Selection>,
    {
        Self::new_unvalidated(selections.into_iter().collect()).validate()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_unvalidated(selections: Vec<Selection>) -> Self {
        Self::new_inner(selections)
    }
}

impl Source for Selections {
    type Iterator = Iter<Selections>;
    type Prepared = Prepared<Selections>;

    fn prepare(self) -> Self::Prepared {
        self.into()
    }
}

impl Validate for Selections {
    type Err = Error;

    fn validate(self) -> Result<Self, Self::Err> {
        validate(&self.0, "queries", &[&vec::IsEmpty])?;

        Ok(self)
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::{
    event::EventMasked,
    mask::Mask,
    prepared::Prepared,
    selector::{
        Selector,
        specifiers::Specifiers,
        tags::Tags,
    },
    source::Source,
};

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use eventric_utils::validation::Validate;

    use crate::{
        error::Error,
        event::{
            identifier::Identifier,
            specifier::Specifier,
            tag::Tag,
        },
        stream::select::{
            Selection,
            SelectionHash,
            // SelectionHashRef,
            Selector,
        },
    };

    // Query::new

    #[test]
    fn new_with_single_selector() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let selector = Selector::specifiers(vec![spec]).unwrap();

        let result = Selection::new(vec![selector]);

        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(1, query.selectors.len());
    }

    #[test]
    fn new_with_multiple_selectors() {
        let id1 = Identifier::new("EventA").unwrap();
        let id2 = Identifier::new("EventB").unwrap();
        let id3 = Identifier::new("EventC").unwrap();

        let spec1 = Specifier::new(id1);
        let spec2 = Specifier::new(id2);
        let spec3 = Specifier::new(id3);

        let selector1 = Selector::specifiers(vec![spec1]).unwrap();
        let selector2 = Selector::specifiers(vec![spec2]).unwrap();
        let selector3 = Selector::specifiers(vec![spec3]).unwrap();

        let result = Selection::new(vec![selector1, selector2, selector3]);

        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(3, query.selectors.len());
    }

    #[test]
    fn new_with_mixed_selector_types() {
        let id1 = Identifier::new("EventA").unwrap();
        let spec1 = Specifier::new(id1);
        let selector1 = Selector::specifiers(vec![spec1]).unwrap();

        let id2 = Identifier::new("EventB").unwrap();
        let spec2 = Specifier::new(id2);
        let tag = Tag::new("user:123").unwrap();
        let selector2 = Selector::specifiers_and_tags(vec![spec2], vec![tag]).unwrap();

        let result = Selection::new(vec![selector1, selector2]);

        assert!(result.is_ok());

        let query = result.unwrap();

        assert_eq!(2, query.selectors.len());
    }

    #[test]
    fn new_with_empty_vec_returns_error() {
        let result = Selection::new(Vec::<Selector>::new());

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }

    // Query::new_unvalidated

    #[test]
    fn new_unvalidated_allows_empty_vec() {
        let query = Selection::new_unvalidated(vec![]);

        assert_eq!(0, query.selectors.len());
    }

    #[test]
    fn new_unvalidated_with_selectors() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let selector = Selector::specifiers(vec![spec]).unwrap();

        let query = Selection::new_unvalidated(vec![selector]);

        assert_eq!(1, query.selectors.len());
    }

    // AsRef<[Selector]>

    #[test]
    fn as_ref_returns_slice() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let selector = Selector::specifiers(vec![spec]).unwrap();
        let query = Selection::new(vec![selector]).unwrap();

        let slice: &[Selector] = query.as_ref();

        assert_eq!(1, slice.len());
    }

    // Clone

    #[test]
    fn clone_creates_independent_copy() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let selector = Selector::specifiers(vec![spec]).unwrap();
        let query = Selection::new(vec![selector]).unwrap();

        let cloned = query.clone();

        assert_eq!(query.selectors.len(), cloned.selectors.len());
    }

    // From<Query> for Vec<Selector>

    #[allow(clippy::similar_names)]
    #[test]
    fn from_query_to_vec_selector() {
        let id1 = Identifier::new("EventA").unwrap();
        let id2 = Identifier::new("EventB").unwrap();

        let spec1 = Specifier::new(id1);
        let spec2 = Specifier::new(id2);

        let selector1 = Selector::specifiers(vec![spec1]).unwrap();
        let selector2 = Selector::specifiers(vec![spec2]).unwrap();

        let query = Selection::new(vec![selector1, selector2]).unwrap();

        let selectors: Vec<Selector> = query.into();

        assert_eq!(2, selectors.len());
    }

    // Validate

    #[test]
    fn validate_succeeds_for_non_empty() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let selector = Selector::specifiers(vec![spec]).unwrap();
        let query = Selection::new_unvalidated(vec![selector]);

        let result = query.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn validate_fails_for_empty() {
        let query = Selection::new_unvalidated(vec![]);

        let result = query.validate();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }

    // From<Query> for QueryHash

    #[test]
    fn from_query_to_query_hash_by_value() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let selector = Selector::specifiers(vec![spec]).unwrap();
        let query = Selection::new(vec![selector]).unwrap();

        let _hash: SelectionHash = query.into();
    }

    // #[test]
    // fn from_query_to_query_hash_by_ref() {
    //     let id = Identifier::new("TestEvent").unwrap();
    //     let spec = Specifier::new(id);
    //     let selector = Selector::specifiers(vec![spec]).unwrap();
    //     let query = Selection::new(vec![selector]).unwrap();

    //     let _hash: SelectionHash = (&query).into();
    // }

    // From<&Query> for QueryHashRef

    // #[test]
    // fn from_query_to_query_hash_ref() {
    //     let id = Identifier::new("TestEvent").unwrap();
    //     let spec = Specifier::new(id);
    //     let selector = Selector::specifiers(vec![spec]).unwrap();
    //     let query = Selection::new(vec![selector]).unwrap();

    //     let _hash_ref: SelectionHashRef<'_> = (&query).into();
    // }

    // // From<QueryHashRef> for QueryHash

    // #[test]
    // fn from_query_hash_ref_to_query_hash() {
    //     let id = Identifier::new("TestEvent").unwrap();
    //     let spec = Specifier::new(id);
    //     let selector = Selector::specifiers(vec![spec]).unwrap();
    //     let query = Selection::new(vec![selector]).unwrap();

    //     let hash_ref: SelectionHashRef<'_> = (&query).into();
    //     let _hash: SelectionHash = hash_ref.into();
    // }

    // #[test]
    // fn from_query_hash_ref_to_query_hash_by_ref() {
    //     let id = Identifier::new("TestEvent").unwrap();
    //     let spec = Specifier::new(id);
    //     let selector = Selector::specifiers(vec![spec]).unwrap();
    //     let query = Selection::new(vec![selector]).unwrap();

    //     let hash_ref: SelectionHashRef<'_> = (&query).into();
    //     let _hash: SelectionHash = (&hash_ref).into();
    // }
}
