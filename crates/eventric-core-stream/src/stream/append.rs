use eventric_core_model::{
    Event,
    Position,
};
use eventric_core_state::Write;

// =================================================================================================
// Append
// =================================================================================================

pub fn append<'a, E>(write: &mut Write<'_>, position: &mut Position, events: E)
where
    E: IntoIterator<Item = &'a Event>,
{
    for event in events {
        let event = event.into();

        eventric_core_data::insert(write, *position, &event);
        eventric_core_index::insert(write, *position, &event);
        eventric_core_reference::insert(write, &event);

        position.increment();
    }
}
