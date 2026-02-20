use fancy_constructor::new;

use crate::{
    event::Event,
    stream::select::mask::Mask,
};

// =================================================================================================
// Event
// =================================================================================================

/// .
#[derive(new, Debug, Eq, PartialEq)]
#[new(const_fn, vis(pub(crate)))]
pub struct EventAndMask {
    /// .
    pub event: Event,
    /// .
    pub mask: Mask,
}
