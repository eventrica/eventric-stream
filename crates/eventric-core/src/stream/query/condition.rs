use derive_more::Debug;
use fancy_constructor::new;

use crate::{
    event::position::Position,
    stream::query::Query,
};

// =================================================================================================
// Condition
// =================================================================================================

/// The [`Condition`] type defines the criteria used to query a
/// [`Stream`][stream], allowing the supply of a query to match, a position from
/// which to begin the query, or both (or neither).
///
/// [stream]: crate::stream::Stream
#[derive(new, Debug)]
#[new(vis())]
pub struct Condition<'a> {
    #[new(default)]
    pub(crate) matches: Option<&'a Query>,
    #[new(default)]
    pub(crate) from: Option<Position>,
}

impl<'a> Condition<'a> {
    /// Set the [`Query`] which should be matched by the returned events.
    #[must_use]
    pub fn matches(mut self, query: &'a Query) -> Self {
        self.matches = Some(query);
        self
    }

    /// Set the [`Position`] in the [`Stream`][stream] from which the query
    /// should begin to return events.
    ///
    /// [stream]: crate::stream::Stream
    #[must_use]
    pub fn from(mut self, position: Position) -> Self {
        self.from = Some(position);
        self
    }
}

impl Default for Condition<'_> {
    fn default() -> Self {
        Self::new()
    }
}
