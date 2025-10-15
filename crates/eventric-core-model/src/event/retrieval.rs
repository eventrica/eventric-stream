use derive_more::Debug;
use fancy_constructor::new;

use crate::event::{
    Data,
    DescriptorHash,
    TagHash,
};

// =================================================================================================
// Retrieval
// =================================================================================================

// Event

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct EventHash {
    #[new(into)]
    pub data: Data,
    #[new(into)]
    pub descriptor: DescriptorHash,
    pub tags: Vec<TagHash>,
}
