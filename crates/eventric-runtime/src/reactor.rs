//! The [`Reactor`]: drives a single [`React`]ion over a stream — replays every
//! event of the reaction's type since the last checkpoint, builds the reaction
//! (`From`), runs it, and applies its staged effects. [`Reactor::run`] applies
//! view deltas only (read-only); [`Reactor::drive`] also dispatches the
//! commands a reaction issues (read-write), closing the
//! event→reaction→command→event loop in-memory.

use error_stack::{
    Report,
    ResultExt as _,
};
use eventric_model::{
    action::Command,
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
        append::Append,
        select::{
            EventAndMask,
            Select,
            Selector,
        },
    },
};

use crate::enactor::Enactor as _;

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
    /// staged view deltas. Commands a reaction stages are **ignored** here —
    /// use [`Reactor::drive`] for a command-issuing reaction. Advances the
    /// checkpoint to the last event seen.
    pub fn run<S>(&mut self, stream: &S) -> Result<(), Report<Error>>
    where
        S: Select,
    {
        for event in stream.select(self.condition()?) {
            let event = event.change_context(Error)?;
            let reaction = Self::recognize(&event)?;

            let mut effects = Effects::new();
            reaction.react(&mut effects);
            let (deltas, _commands) = effects.into_parts();
            for delta in deltas {
                self.view.apply(delta);
            }

            self.checkpoint = Some(event.event.meta().position());
        }

        Ok(())
    }

    /// Like [`Reactor::run`], but also **dispatches** the commands a reaction
    /// stages: each command is routed to its [`Command::Action`], built from
    /// the command (`From`), and enacted against the stream. Because a
    /// command may append events the reaction itself reacts to, this
    /// repeats until no new matching event remains — bounded by a runaway
    /// guard.
    pub fn drive<S>(&mut self, stream: &mut S) -> Result<(), Report<Error>>
    where
        S: Append + Select,
        R::Command: Command,
    {
        // Guards a reaction whose command re-triggers it without converging.
        const MAX_PASSES: usize = 1024;

        for _ in 0..MAX_PASSES {
            // Collect first, releasing the read borrow before a command writes.
            let events = stream.select(self.condition()?).collect::<Vec<_>>();
            if events.is_empty() {
                return Ok(());
            }

            for event in events {
                let event = event.change_context(Error)?;
                let reaction = Self::recognize(&event)?;

                let mut effects = Effects::new();
                reaction.react(&mut effects);
                let (deltas, commands) = effects.into_parts();
                for delta in deltas {
                    self.view.apply(delta);
                }
                for command in commands {
                    // The command → action registry is the type itself.
                    let action = <R::Command as Command>::Action::from(command);
                    stream.enact(action)?;
                }

                self.checkpoint = Some(event.event.meta().position());
            }
        }

        Err(Report::new(Error)
            .attach("reactor: drive exceeded the pass limit (a non-converging command loop?)"))
    }

    // The condition selecting the reaction's event type, from just past the
    // checkpoint.
    fn condition(&self) -> Result<Condition, Report<Error>> {
        let selection = Selection::new([Selector::types([R::Event::specifier()?])]);
        let mut condition = Condition::new().selections([selection]);

        if let Some(position) = self.checkpoint {
            condition = condition.from(position + 1);
        }

        Ok(condition)
    }

    // Decode a selected event into the reaction's event type, building the
    // reaction from it.
    fn recognize(event: &EventAndMask) -> Result<R, Report<Error>> {
        let decoded: R::Event = revision::from_slice(event.event.data().as_ref())
            .change_context(Error)
            .attach("reactor: decode event payload")?;

        Ok(R::from(decoded))
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
