use derive_more::Deref;
use fancy_constructor::new;

use crate::{
    event::PersistentEvent,
    stream::query::Mask,
};

// =================================================================================================
// Event
// =================================================================================================

/// .
#[derive(new, Debug, Deref, Eq, PartialEq)]
#[new(const_fn, vis(pub(crate)))]
pub struct PersistentEventMasked {
    #[deref]
    event: PersistentEvent,
    mask: Mask,
}

impl PersistentEventMasked {
    /// .
    #[must_use]
    pub fn mask(&self) -> &Mask {
        &self.mask
    }
}
