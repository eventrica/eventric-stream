use derive_more::Debug;
use eventric_core_event::position::Position;
use fancy_constructor::new;

use crate::query::Query;

// =================================================================================================
// Condition
// =================================================================================================

#[derive(new, Debug)]
#[new(vis())]
pub struct Condition<'a> {
    #[new(default)]
    pub(crate) matches: Option<&'a Query>,
    #[new(default)]
    pub(crate) from: Option<Position>,
}

impl<'a> Condition<'a> {
    #[must_use]
    pub fn matches(mut self, query: &'a Query) -> Self {
        self.matches = Some(query);
        self
    }

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
