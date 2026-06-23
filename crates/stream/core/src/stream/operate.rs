mod append;
mod select;

use crate::stream::Position;

// =================================================================================================
// Operations
// =================================================================================================

// Condition

/// A query (and, later, append concurrency) condition: an optional lower
/// position bound plus zero or more [`Selection`]s to match.
///
/// Each [`Selection`] is one mask unit. A matched event carries a
/// [`Mask`](crate::stream::Mask) recording which selections it satisfied,
/// in the order they were supplied. With no selections the condition matches
/// the whole stream (a full scan).
#[derive(Debug, Default)]
pub struct Condition {
    pub(crate) position: Option<Position>,
    pub(crate) selections: Vec<Selection>,
}

impl Condition {
    /// Begin a condition that matches the whole stream from the start.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Restrict the condition to events at or after `position`.
    #[must_use]
    pub fn from(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// Set the selections to match. Each is one mask unit (see [`Condition`]).
    #[must_use]
    pub fn selections<I>(mut self, selections: I) -> Self
    where
        I: IntoIterator<Item = Selection>,
    {
        self.selections = selections.into_iter().collect();
        self
    }
}

// -------------------------------------------------------------------------------------------------

// Selection

/// One mask unit: a set of [`Selector`]s combined with OR. An event matches the
/// selection if it matches any of its selectors. String type-names and tags are
/// hashed when the selection is built.
#[derive(Debug)]
pub struct Selection {
    pub(crate) selectors: Vec<Selector<u64>>,
}

impl Selection {
    /// Build a selection from one or more selectors (combined with OR).
    pub fn new<I>(selectors: I) -> Self
    where
        I: IntoIterator<Item = Selector<String>>,
    {
        Self {
            selectors: selectors.into_iter().map(Into::into).collect(),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub(crate) use self::append::Appender;
pub use self::{
    append::Append,
    select::{
        EventAndMask,
        Mask,
        Select,
        SelectIter,
        Selector,
        TypeSelector,
        VersionSelector,
    },
};
