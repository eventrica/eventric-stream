use fancy_constructor::new;

use crate::{
    event::position::Position,
    stream::query::Query,
};

// =================================================================================================
// Condition
// =================================================================================================

/// The [`Condition`] type determines whether a [`Stream::append`][append]
/// operation should be conditional. If a condition is supplied, the append will
/// not proceed if any events match the given query (optionally after a given
/// position in the stream).
///
/// [append]: crate::stream::Stream::query
#[derive(new, Debug)]
#[new(name(new_inner), vis())]
pub struct Condition<'a> {
    #[new(default)]
    pub(crate) after: Option<Position>,
    pub(crate) fail_if_matches: &'a Query,
}

impl<'a> Condition<'a> {
    /// Constructs a new [`Condition`] given a reference to a query which should
    /// cause the append to fail if it is matched.
    #[must_use]
    pub fn new(fail_if_matches: &'a Query) -> Self {
        Self::new_inner(fail_if_matches)
    }
}

impl Condition<'_> {
    /// Sets a position after which the query should apply. If no position is
    /// supplied, the append will fail if *any* events match the query at any
    /// point in the stream, while supplying a [`Position`] will only cause the
    /// append to fail if events match in the stream after the given position.
    #[must_use]
    pub fn after(mut self, after: Position) -> Self {
        self.after = Some(after);
        self
    }
}
