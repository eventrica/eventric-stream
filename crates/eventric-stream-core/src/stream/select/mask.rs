use std::ops::Index;

use derive_more::{
    AsRef,
    Deref,
};
use fancy_constructor::new;
use smallvec::SmallVec;

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
pub struct Mask(#[new(name(mask))] pub(crate) SmallVec<[bool; 8]>);

impl Index<usize> for Mask {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}
