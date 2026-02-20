//! See the `eventric-stream` crate for full documentation, including
//! module-level documentation.

pub(crate) mod cache;
pub(crate) mod iter;

use std::sync::{
    Arc,
    Exclusive,
};

use crate::{
    event::position::Position,
    stream::{
        data::{
            Data,
            events::EventHashIter,
        },
        iterate::cache::Cache,
    },
};

// =================================================================================================
// Iterate
// =================================================================================================

// Iterate

/// .
pub trait Iterate {
    /// .
    fn iter(&self, from: Option<Position>) -> Iter;
}

// Implementations

pub(crate) fn iter(data: &Data, from: Option<Position>) -> Iter {
    let cache = Arc::new(Cache::default());
    let references = data.references.clone();

    let iter = data.events.iterate(from);
    let iter = EventHashIter::Direct(iter);
    let iter = Exclusive::new(iter);

    Iter::new(cache, references, iter)
}

// -------------------------------------------------------------------------------------------------

// Re-Export

pub use self::iter::Iter;
