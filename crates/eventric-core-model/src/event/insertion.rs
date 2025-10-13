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
    pub data: Vec<u8>,
    #[new(into)]
    pub descriptor: Descriptor,
    #[new(into)]
    pub tags: Vec<Tag>,
}
