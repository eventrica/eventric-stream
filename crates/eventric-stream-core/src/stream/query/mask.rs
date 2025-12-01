use std::ops::Index;

use derive_more::{
    AsRef,
    Deref,
};
use fancy_constructor::new;

// =================================================================================================
// Mask
// =================================================================================================

/// The [`Mask`] type represents a [`Query`][query] matching mask when iterating
/// over a [`Queries`][queries] collection.
///
/// [query]: crate::stream::query::Query
/// [queries]: crate::stream::query::Queries
#[derive(new, AsRef, Clone, Deref, Debug, Eq, PartialEq)]
#[as_ref([bool])]
#[new(const_fn)]
pub struct Mask<const N: usize>(#[new(name(mask))] pub(crate) [bool; N]);

impl<const N: usize> Index<usize> for Mask<N> {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}
