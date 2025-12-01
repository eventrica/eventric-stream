use derive_more::Deref;
use fancy_constructor::new;

use crate::{
    event::Event,
    stream::query::Mask,
};

// =================================================================================================
// Event
// =================================================================================================

/// .
#[derive(new, Debug, Deref, Eq, PartialEq)]
#[new(const_fn, vis(pub(crate)))]
pub struct EventMasked<const N: usize> {
    #[deref]
    event: Event,
    mask: Mask<N>,
}

impl<const N: usize> EventMasked<N> {
    /// .
    #[must_use]
    pub fn mask(&self) -> &Mask<N> {
        &self.mask
    }
}
