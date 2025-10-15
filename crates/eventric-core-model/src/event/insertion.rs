use fancy_constructor::new;

use crate::event::{
    Data,
    Descriptor,
    DescriptorHashRef,
    Tag,
    TagHashRef,
};

// =================================================================================================
// Insertion
// =================================================================================================

// Event

#[derive(new, Debug)]
pub struct Event {
    #[new(into)]
    data: Data,
    #[new(into)]
    descriptor: Descriptor,
    #[new(into)]
    tags: Vec<Tag>,
}

impl Event {
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

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct EventHashRef<'a> {
    #[new(into)]
    pub data: &'a Data,
    #[new(into)]
    pub descriptor: DescriptorHashRef<'a>,
    pub tags: Vec<TagHashRef<'a>>,
}

impl<'a> From<&'a Event> for EventHashRef<'a> {
    fn from(event: &'a Event) -> Self {
        Self::new(
            event.data(),
            event.descriptor(),
            event.tags().iter().map(Into::into).collect(),
        )
    }
}
