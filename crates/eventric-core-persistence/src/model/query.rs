use std::ops::Range;

use eventric_core_model::{
    event::Version,
    query::{
        Query,
        QueryItem,
        Specifier,
    },
};
use fancy_constructor::new;

use crate::model::event::{
    IdentifierRef,
    TagRef,
};

// =================================================================================================
// Query
// =================================================================================================

#[derive(new, Debug)]
pub struct QueryRef<'a> {
    #[new(into)]
    items: Vec<QueryItemRef<'a>>,
}

impl<'a> QueryRef<'a> {
    #[must_use]
    pub fn items(&self) -> &Vec<QueryItemRef<'a>> {
        &self.items
    }
}

impl<'a> From<&'a Query> for QueryRef<'a> {
    fn from(value: &'a Query) -> Self {
        Self::new(
            value
                .items()
                .iter()
                .map(Into::into)
                .collect::<Vec<QueryItemRef<'a>>>(),
        )
    }
}

#[derive(Debug)]
pub enum QueryItemRef<'a> {
    Specifiers(Vec<SpecifierRef<'a>>),
    SpecifiersAndTags(Vec<SpecifierRef<'a>>, Vec<TagRef<'a>>),
    Tags(Vec<TagRef<'a>>),
}

impl<'a> From<&'a QueryItem> for QueryItemRef<'a> {
    fn from(value: &'a QueryItem) -> Self {
        match value {
            QueryItem::Specifiers(specifiers) => {
                Self::Specifiers(specifiers.iter().map(Into::into).collect())
            }
            QueryItem::SpecifiersAndTags(specifiers, tags) => Self::SpecifiersAndTags(
                specifiers.iter().map(Into::into).collect(),
                tags.iter().map(Into::into).collect(),
            ),
            QueryItem::Tags(tags) => Self::Tags(tags.iter().map(Into::into).collect()),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Specifier

#[derive(new, Debug)]
#[new(vis())]
pub struct SpecifierRef<'a>(IdentifierRef<'a>, Option<&'a Range<Version>>);

impl<'a> SpecifierRef<'a> {
    #[must_use]
    pub fn identifer(&self) -> &IdentifierRef<'a> {
        &self.0
    }

    #[must_use]
    pub fn range(&self) -> Option<&'a Range<Version>> {
        self.1
    }
}

impl<'a> From<&'a Specifier> for SpecifierRef<'a> {
    fn from(specifier: &'a Specifier) -> Self {
        let identifier = specifier.identifier().into();
        let range = specifier.range();

        Self::new(identifier, range)
    }
}
