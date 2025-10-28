use derive_more::Debug;
use fancy_constructor::new;

use crate::{
    event::Position,
    stream::query::Query,
};

// =================================================================================================
// Condition
// =================================================================================================

#[derive(new, Debug)]
#[new(vis())]
pub struct QueryCondition<'a> {
    #[new(default)]
    pub(crate) matches: Option<&'a Query>,
    #[new(default)]
    pub(crate) from: Option<Position>,
}

impl<'a> QueryCondition<'a> {
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

impl Default for QueryCondition<'_> {
    fn default() -> Self {
        Self::new()
    }
}
