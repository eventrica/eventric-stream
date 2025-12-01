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
pub struct PersistentEventMasked<const N: usize> {
    #[deref]
    event: PersistentEvent,
    mask: Mask<N>,
}

impl<const N: usize> PersistentEventMasked<N> {
    /// .
    #[must_use]
    pub fn mask(&self) -> &Mask<N> {
        &self.mask
    }
}
