use fancy_constructor::new;

use crate::{
    event::position::Position,
    stream::query::Query,
};

// =================================================================================================
// Condition
// =================================================================================================

#[derive(new, Debug)]
#[new(name(new_inner), vis())]
pub struct Condition<'a> {
    #[new(default)]
    pub(crate) after: Option<Position>,
    pub(crate) fail_if_matches: &'a Query,
}

impl<'a> Condition<'a> {
    #[must_use]
    pub fn new(fail_if_matches: &'a Query) -> Self {
        Self::new_inner(fail_if_matches)
    }
}

impl Condition<'_> {
    #[must_use]
    pub fn after(mut self, after: Position) -> Self {
        self.after = Some(after);
        self
    }
}
