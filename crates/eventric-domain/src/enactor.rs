//! The [`Enactor`]: runs an [`Action`] against a stream under a single DCB
//! [`Condition`] — replay, decide, append-or-reject.

use error_stack::ResultExt as _;
use eventric_stream::stream::operate::{
    Condition,
    append::Append,
    select::Select,
};

use crate::{
    action::Action,
    error::Error,
};

// =================================================================================================
// Core
// =================================================================================================

// Enactor

/// Runs [`Action`]s against a stream. Blanket-implemented for any
/// [`Append`] + [`Select`] handle (a `Stream`, or a concurrent `Proxy`).
pub trait Enactor {
    /// Enact `action`: build its context, replay the matching events (folding
    /// each into the projection it matched), run the business logic, then
    /// append any emitted events under a DCB condition that rejects if a
    /// matching event appeared since the replay. Returns the action's
    /// success value, or its error (a conflict surfaces as the stream
    /// `Conflict` attachment).
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
            let event_and_mask = event.change_context(Error)?;

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

            self.append(events, condition).change_context(Error)?;
        }

        Ok(ok)
    }
}
