use fancy_constructor::new;

use crate::event::{
    Descriptor,
    Tag,
};

// =================================================================================================
// Event
// =================================================================================================

#[derive(new, Debug)]
pub struct Event {
    #[new(into)]
    data: Vec<u8>,
    #[new(into)]
    descriptor: Descriptor,
    #[new(into)]
    tags: Vec<Tag>,
}

impl Event {
    #[must_use]
    pub fn data(&self) -> &Vec<u8> {
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
