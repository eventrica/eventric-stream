use derive_more::Debug;
use fancy_constructor::new;

use crate::model::{
    query::Query,
    stream::position::Position,
};

// =================================================================================================
// Condition
// =================================================================================================

#[derive(new, Debug, Default)]
#[new(const_fn, vis())]
pub struct Condition<'a> {
    pub(crate) query: Option<&'a Query>,
    pub(crate) position: Option<Position>,
}

impl<'a> Condition<'a> {
    #[must_use]
    pub fn query(mut self, query: &'a Query) -> Self {
        self.query = Some(query);
        self
    }

    #[must_use]
    pub fn position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }
}
