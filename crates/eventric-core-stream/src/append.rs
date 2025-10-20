use eventric_core_model::{
    Event,
    Position,
    Query,
    Timestamp,
};
use fancy_constructor::new;
use fjall::WriteBatch;

use crate::stream::StreamKeyspaces;

// =================================================================================================
// Append
// =================================================================================================

pub fn append<'a>(
    batch: &mut WriteBatch,
    keyspaces: &StreamKeyspaces,
    events: impl IntoIterator<Item = &'a Event>,
    _condition: Option<AppendCondition<'a>>,
    position: &mut Position,
) {
    // Check condition here!

    for event in events {
        let event = event.into();
        let timestamp = Timestamp::now();

        eventric_core_data::insert(batch, &keyspaces.data, &event, *position, timestamp);
        eventric_core_index::insert(batch, &keyspaces.index, &event, *position, timestamp);
        eventric_core_reference::insert(batch, &keyspaces.reference, &event);

        position.increment();
    }
}

// -------------------------------------------------------------------------------------------------

// Condition

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
