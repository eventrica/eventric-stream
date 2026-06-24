use crate::{
    model::action::Action,
    stream::operate::{
        Condition,
        append::Append,
        select::Select,
    },
};

// =================================================================================================
// Core
// =================================================================================================

// Enactor

pub trait Enactor {
    fn enact<A>(&mut self, action: A) -> Result<A::Ok, A::Err>
    where
        A: Action;
}

impl<T> Enactor for T
where
    T: Append + Select,
{
    fn enact<A>(&mut self, mut action: A) -> Result<A::Ok, A::Err>
    where
        A: Action,
    {
        let mut context = action.context();
        let mut after = None;

        // Replay every event matching the action's selections, folding each into
        // the projection it matched (via the mask, inside `update`).
        let condition = Condition::new().selections(action.select(&context)?);

        for event in self.select(condition) {
            let event_and_mask = event?;

            after = Some(event_and_mask.event.meta().position());

            action.update(&mut context, &event_and_mask)?;
        }

        // Run the business logic, then append any emitted events under a DCB
        // condition: reject if a matching event appeared after the last replayed
        // position.
        let ok = action.action(&mut context)?;

        let selections = action.select(&context)?;
        let events = context.into().take();

        if !events.is_empty() {
            let mut condition = Condition::new().selections(selections);

            if let Some(position) = after {
                condition = condition.from(position + 1);
            }

            self.append(events, condition)?;
        }

        Ok(ok)
    }
}
