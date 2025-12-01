use std::ops::Index;

use derive_more::{
    AsRef,
    Deref,
};
use fancy_constructor::new;

// =================================================================================================
// Mask
// =================================================================================================

/// The [`Mask`] type represents a [`Selection`][selection] matching mask when
/// iterating over a [`Selections`][selections] collection.
///
/// [selection]: crate::stream::select::Selection
/// [selections]: crate::stream::select::Selections
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
