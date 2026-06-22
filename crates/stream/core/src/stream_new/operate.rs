mod append;
mod select;

use crate::stream_new::Position;

// =================================================================================================
// Operations
// =================================================================================================

// Condition

#[derive(Debug)]
pub struct Condition {
    pub(crate) position: Option<Position>,
    pub(crate) selection: Option<Selection>,
}

#[derive(Debug)]
pub struct Selection {
    pub(crate) selectors: Vec<Selector<u64>>,
    // ALSO NEED A MASK HERE
}

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::{
    append::Append,
    select::{
        Select,
        SelectIter,
        Selector,
        TypeSelector,
    },
};
