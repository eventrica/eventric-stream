use derive_more::Deref;
use fancy_constructor::new;

use crate::{
    event::Event,
    stream::select::Mask,
};

// =================================================================================================
// Event
// =================================================================================================

/// .
#[derive(new, Debug, Deref, Eq, PartialEq)]
#[new(const_fn, vis(pub(crate)))]
pub struct EventMasked {
    #[deref]
    event: Event,
    mask: Mask,
}

impl EventMasked {
    /// .
    #[must_use]
    pub fn mask(&self) -> &Mask {
        &self.mask
    }
}
