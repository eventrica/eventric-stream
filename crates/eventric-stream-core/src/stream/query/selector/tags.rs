use derive_more::AsRef;
use eventric_core::validation::{
    Validate,
    validate,
    vec,
};
use fancy_constructor::new;

use crate::{
    error::Error,
    event::tag::{
        Tag,
        TagHash,
        TagHashRef,
    },
};

// =================================================================================================
// Tags
// =================================================================================================

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
    use eventric_core::validation::Validate;

    use crate::{
        error::Error,
        event::tag::Tag,
        stream::query::selector::tags::Tags,
    };

    // Tags::new

    #[test]
    fn new_with_single_tag() {
        let tag = Tag::new("user:123").unwrap();

        let result = Tags::new(vec![tag]);

        assert!(result.is_ok());
        let tags = result.unwrap();
        assert_eq!(1, tags.tags.len());
    }

    #[allow(clippy::similar_names)]
    #[test]
    fn new_with_multiple_tags() {
        let tag1 = Tag::new("user:123").unwrap();
        let tag2 = Tag::new("course:456").unwrap();
        let tag3 = Tag::new("tenant:789").unwrap();

        let result = Tags::new(vec![tag1, tag2, tag3]);

        assert!(result.is_ok());

        let tags = result.unwrap();

        assert_eq!(3, tags.tags.len());
    }

    #[test]
    fn new_with_empty_vec_returns_error() {
        let result = Tags::new(Vec::<Tag>::new());

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }

    // Tags::new_unvalidated

    #[test]
    fn new_unvalidated_allows_empty_vec() {
        let tags = Tags::new_unvalidated(vec![]);

        assert_eq!(0, tags.tags.len());
    }

    #[test]
    fn new_unvalidated_with_tags() {
        let tag = Tag::new("user:123").unwrap();

        let tags = Tags::new_unvalidated(vec![tag]);

        assert_eq!(1, tags.tags.len());
    }

    // AsRef<[Tag]>

    #[test]
    fn as_ref_returns_slice() {
        let tag = Tag::new("user:123").unwrap();
        let tags = Tags::new(vec![tag]).unwrap();

        let slice: &[Tag] = tags.as_ref();

        assert_eq!(1, slice.len());
    }

    // Clone

    #[test]
    fn clone_creates_independent_copy() {
        let tag = Tag::new("user:123").unwrap();
        let tags = Tags::new(vec![tag]).unwrap();

        let cloned = tags.clone();

        assert_eq!(tags.tags.len(), cloned.tags.len());
    }

    // From<&Tags> for Vec<TagHash>

    #[allow(clippy::similar_names)]
    #[test]
    fn from_tags_to_tag_hash_vec() {
        use crate::event::tag::TagHash;

        let tag1 = Tag::new("user:123").unwrap();
        let tag2 = Tag::new("course:456").unwrap();

        let tags = Tags::new(vec![tag1, tag2]).unwrap();

        let hashes: Vec<TagHash> = (&tags).into();

        assert_eq!(2, hashes.len());
    }

    // From<&Tags> for Vec<TagHashRef>

    #[allow(clippy::similar_names)]
    #[test]
    fn from_tags_to_tag_hash_ref_vec() {
        use crate::event::tag::TagHashRef;

        let tag1 = Tag::new("user:123").unwrap();
        let tag2 = Tag::new("course:456").unwrap();

        let tags = Tags::new(vec![tag1, tag2]).unwrap();

        let hash_refs: Vec<TagHashRef<'_>> = (&tags).into();

        assert_eq!(2, hash_refs.len());
    }

    // Validate

    #[test]
    fn validate_succeeds_for_non_empty() {
        let tag = Tag::new("user:123").unwrap();
        let tags = Tags::new_unvalidated(vec![tag]);

        let result = tags.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn validate_fails_for_empty() {
        let tags = Tags::new_unvalidated(vec![]);

        let result = tags.validate();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }
}
