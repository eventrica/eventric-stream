//! The [`Reactor`]: drives a single [`React`]ion over a stream — replays every
//! event of the reaction's type since the last checkpoint, builds the reaction
//! (`From`), runs it, and applies the staged deltas to the view it maintains.

use error_stack::{
    Report,
    ResultExt as _,
};
use eventric_model::{
    error::Error,
    event::Specifier as _,
    reaction::{
        Effects,
        React,
        View as _,
    },
};
use eventric_stream::stream::{
    Position,
    operate::{
        Condition,
        Selection,
        select::{
            Select,
            Selector,
        },
    },
};

// =================================================================================================
// Reactor
// =================================================================================================

/// Drives a single [`React`]ion over a stream, owning and maintaining the view
/// it folds into. The progress checkpoint is in-memory for now (boundary §9: a
/// persisted checkpoint is a later step).
pub struct Reactor<R>
where
    R: React,
{
    view: R::View,
    checkpoint: Option<Position>,
}

impl<R> Reactor<R>
where
    R: React,
{
    /// A fresh reactor: an empty (`Default`) view and no progress.
    #[must_use]
    pub fn new() -> Self {
        Self {
            view: R::View::default(),
            checkpoint: None,
        }
    }

    /// The maintained view.
    #[must_use]
    pub fn view(&self) -> &R::View {
        &self.view
    }

    /// Process every event of the reaction's type that has appeared since the
    /// last checkpoint: decode it, build the reaction, run it, and apply the
    /// staged deltas to the view. Advances the checkpoint to the last event
    /// seen.
    pub fn run<S>(&mut self, stream: &S) -> Result<(), Report<Error>>
    where
        S: Select,
    {
        let selection = Selection::new([Selector::types([R::Event::specifier()?])]);
        let mut condition = Condition::new().selections([selection]);

        if let Some(position) = self.checkpoint {
            condition = condition.from(position + 1);
        }

        for event in stream.select(condition) {
            let event = event.change_context(Error)?;

            let decoded: R::Event = revision::from_slice(event.event.data().as_ref())
                .change_context(Error)
                .attach("reactor: decode event payload")?;

            let reaction = R::from(decoded);

            let mut effects = Effects::new();
            reaction.react(&mut effects);
            for delta in effects.into_deltas() {
                self.view.apply(delta);
            }

            self.checkpoint = Some(event.event.meta().position());
        }

        Ok(())
    }
}

impl<R> Default for Reactor<R>
where
    R: React,
{
    fn default() -> Self {
        Self::new()
    }
}
