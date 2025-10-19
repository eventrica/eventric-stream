use derive_more::Debug;
use eventric_core_model::{
    Position,
    Query,
};
use fancy_constructor::new;

// =================================================================================================
// Condition
// =================================================================================================

// Append Condition

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct AppendCondition<'a> {
    query: &'a Query,
    position: Option<Position>,
}

impl<'a> AppendCondition<'a> {
    #[must_use]
    pub fn take(self) -> (&'a Query, Option<Position>) {
        (self.query, self.position)
    }
}

impl<'a> AppendCondition<'a> {
    #[must_use]
    pub fn builder(fail_if_match: &'a Query) -> AppendConditionBuilder<'a> {
        AppendConditionBuilder::new(fail_if_match)
    }
}

#[derive(new, Debug)]
#[new(vis())]
pub struct AppendConditionBuilder<'a> {
    query: &'a Query,
    #[new(default)]
    position: Option<Position>,
}

impl<'a> AppendConditionBuilder<'a> {
    #[must_use]
    pub fn build(self) -> AppendCondition<'a> {
        AppendCondition::new(self.query, self.position)
    }
}

impl AppendConditionBuilder<'_> {
    #[must_use]
    pub fn after(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }
}

// -------------------------------------------------------------------------------------------------

// Query Condition

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct QueryCondition<'a> {
    query: Option<&'a Query>,
    position: Option<Position>,
}

impl<'a> QueryCondition<'a> {
    #[must_use]
    pub fn take(self) -> (Option<&'a Query>, Option<Position>) {
        (self.query, self.position)
    }
}

impl<'a> QueryCondition<'a> {
    #[must_use]
    pub fn builder() -> QueryConditionBuilder<'a> {
        QueryConditionBuilder::new()
    }
}

#[derive(new, Debug)]
#[new(vis())]
pub struct QueryConditionBuilder<'a> {
    #[new(default)]
    query: Option<&'a Query>,
    #[new(default)]
    position: Option<Position>,
}

impl<'a> QueryConditionBuilder<'a> {
    #[must_use]
    pub fn build(self) -> QueryCondition<'a> {
        QueryCondition::new(self.query, self.position)
    }
}

impl<'a> QueryConditionBuilder<'a> {
    #[must_use]
    pub fn after(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    #[must_use]
    pub fn query(mut self, query: &'a Query) -> Self {
        self.query = Some(query);
        self
    }
}
