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
    stream::query::selector::{
        specifiers::Specifiers,
        tags::Tags,
    },
};

pub(crate) mod specifiers;
pub(crate) mod tags;

// =================================================================================================
// Selector
// =================================================================================================

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

// Tests

#[cfg(test)]
mod tests {
    use crate::{
        error::Error,
        event::{
            identifier::Identifier,
            specifier::Specifier,
            tag::Tag,
        },
        stream::query::Selector,
    };

    // Selector::specifiers

    #[test]
    fn specifiers_with_single_specifier() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);

        let result = Selector::specifiers(vec![spec]);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Selector::Specifiers(_)));
    }

    #[test]
    fn specifiers_with_multiple_specifiers() {
        let id1 = Identifier::new("EventA").unwrap();
        let id2 = Identifier::new("EventB").unwrap();

        let spec1 = Specifier::new(id1);
        let spec2 = Specifier::new(id2);

        let result = Selector::specifiers(vec![spec1, spec2]);

        assert!(result.is_ok());

        match result.unwrap() {
            Selector::Specifiers(specifiers) => {
                assert_eq!(2, specifiers.specifiers.len());
            }
            Selector::SpecifiersAndTags(..) => panic!("Expected Specifiers variant"),
        }
    }

    #[test]
    fn specifiers_with_empty_vec_returns_error() {
        let result = Selector::specifiers(Vec::<Specifier>::new());

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }

    // Selector::specifiers_and_tags

    #[test]
    fn specifiers_and_tags_with_valid_inputs() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let tag = Tag::new("user:123").unwrap();

        let result = Selector::specifiers_and_tags(vec![spec], vec![tag]);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Selector::SpecifiersAndTags(_, _)));
    }

    #[allow(clippy::similar_names)]
    #[test]
    fn specifiers_and_tags_with_multiple_items() {
        let id1 = Identifier::new("EventA").unwrap();
        let id2 = Identifier::new("EventB").unwrap();
        let spec1 = Specifier::new(id1);
        let spec2 = Specifier::new(id2);

        let tag1 = Tag::new("user:123").unwrap();
        let tag2 = Tag::new("course:456").unwrap();

        let result = Selector::specifiers_and_tags(vec![spec1, spec2], vec![tag1, tag2]);

        assert!(result.is_ok());
        match result.unwrap() {
            Selector::SpecifiersAndTags(specifiers, tags) => {
                assert_eq!(2, specifiers.specifiers.len());
                assert_eq!(2, tags.tags.len());
            }
            Selector::Specifiers(_) => panic!("Expected SpecifiersAndTags variant"),
        }
    }

    #[test]
    fn specifiers_and_tags_with_empty_specifiers_returns_error() {
        let tag = Tag::new("user:123").unwrap();

        let result = Selector::specifiers_and_tags(Vec::<Specifier>::new(), vec![tag]);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }

    #[test]
    fn specifiers_and_tags_with_empty_tags_returns_error() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);

        let result = Selector::specifiers_and_tags(vec![spec], Vec::<Tag>::new());

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }

    #[test]
    fn specifiers_and_tags_with_both_empty_returns_error() {
        let result = Selector::specifiers_and_tags(Vec::<Specifier>::new(), Vec::<Tag>::new());

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }

    // Clone

    #[test]
    fn clone_specifiers_variant() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let selector = Selector::specifiers(vec![spec]).unwrap();

        let cloned = selector.clone();

        assert!(matches!(cloned, Selector::Specifiers(_)));
    }

    #[test]
    fn clone_specifiers_and_tags_variant() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let tag = Tag::new("user:123").unwrap();
        let selector = Selector::specifiers_and_tags(vec![spec], vec![tag]).unwrap();

        let cloned = selector.clone();

        assert!(matches!(cloned, Selector::SpecifiersAndTags(_, _)));
    }

    // From<&Selector> for SelectorHash

    #[test]
    fn from_selector_to_selector_hash_specifiers() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let selector = Selector::specifiers(vec![spec]).unwrap();

        let hash = (&selector).into();

        assert!(matches!(
            hash,
            crate::stream::query::selector::SelectorHash::Specifiers(_)
        ));
    }

    #[test]
    fn from_selector_to_selector_hash_specifiers_and_tags() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let tag = Tag::new("user:123").unwrap();
        let selector = Selector::specifiers_and_tags(vec![spec], vec![tag]).unwrap();

        let hash = (&selector).into();

        assert!(matches!(
            hash,
            crate::stream::query::selector::SelectorHash::SpecifiersAndTags(_, _)
        ));
    }

    // From<&Selector> for SelectorHashRef

    #[test]
    fn from_selector_to_selector_hash_ref_specifiers() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let selector = Selector::specifiers(vec![spec]).unwrap();

        let hash_ref = (&selector).into();

        assert!(matches!(
            hash_ref,
            crate::stream::query::selector::SelectorHashRef::Specifiers(_)
        ));
    }

    #[test]
    fn from_selector_to_selector_hash_ref_specifiers_and_tags() {
        let id = Identifier::new("TestEvent").unwrap();
        let spec = Specifier::new(id);
        let tag = Tag::new("user:123").unwrap();
        let selector = Selector::specifiers_and_tags(vec![spec], vec![tag]).unwrap();

        let hash_ref = (&selector).into();

        assert!(matches!(
            hash_ref,
            crate::stream::query::selector::SelectorHashRef::SpecifiersAndTags(_, _)
        ));
    }
}
