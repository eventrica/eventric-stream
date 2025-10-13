use derive_more::Debug;

use crate::model::event::{
    Descriptor,
    Tag,
};

// =================================================================================================
// Query
// =================================================================================================

#[derive(Debug)]
pub struct Query {
    _items: Vec<QueryItem>,
}

#[derive(Debug)]
pub enum QueryItem {
    Descriptors(Vec<Descriptor>),
    DescriptorsAndTags(Vec<Descriptor>, Vec<Tag>),
    Tags(Vec<Tag>),
}
