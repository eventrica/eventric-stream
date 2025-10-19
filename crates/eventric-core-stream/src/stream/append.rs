use std::error::Error;

use eventric_core_model::{
    Event,
    Position,
};
use eventric_core_state::{
    Context,
    Keyspaces,
    Write,
};

// =================================================================================================
// Append
// =================================================================================================

pub fn append<'a, E>(
    context: &Context,
    keyspaces: &Keyspaces,
    position: &mut Position,
    events: E,
) -> Result<(), Box<dyn Error>>
where
    E: IntoIterator<Item = &'a Event>,
{
    let mut batch = context.as_ref().batch();
    let mut write = Write::new(&mut batch, keyspaces);

    for event in events {
        let event = event.into();

        eventric_core_data::insert(&mut write, *position, &event);
        eventric_core_index::insert(&mut write, *position, &event);
        eventric_core_reference::insert(&mut write, &event);

        position.increment();
    }

    batch.commit()?;

    Ok(())
}
