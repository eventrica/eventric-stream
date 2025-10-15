use fancy_constructor::new;

use crate::event::{
    Data,
    Descriptor,
    Tag,
};

// =================================================================================================
// Event
// =================================================================================================

#[derive(new, Debug)]
pub struct InsertionEvent {
    #[new(into)]
    data: Data,
    #[new(into)]
    descriptor: Descriptor,
    #[new(into)]
    tags: Vec<Tag>,
}

impl InsertionEvent {
    #[must_use]
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[must_use]
    pub fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }

    #[must_use]
    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }
}
