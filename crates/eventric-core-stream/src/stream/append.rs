use eventric_core_model::Position;
use eventric_core_state::Write;

use crate::{
    condition::AppendCondition,
    event::Events,
};

// =================================================================================================
// Append
// =================================================================================================

pub fn append<'a>(
    write: &mut Write<'_>,
    events: impl Events<'a>,
    _condition: Option<AppendCondition<'a>>,
    position: &mut Position,
) {
    // Check condition here!

    for event in events {
        let event = event.into();

        eventric_core_data::insert(write, *position, &event);
        eventric_core_index::insert(write, *position, &event);
        eventric_core_reference::insert(write, &event);

        position.increment();
    }
}
