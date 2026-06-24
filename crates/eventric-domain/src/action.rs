//! Commands: the [`Action`] trait (with its [`Act`]/[`Context`]/[`Select`]/
//! [`Update`] components), run by the model
//! [`Enactor`](super::enactor::Enactor).

use std::ops::{
    Deref,
    DerefMut,
};

use error_stack::Report;
pub use eventric_macros::Action;
use eventric_stream::stream::operate::{
    Selection,
    select::EventAndMask,
};

use crate::{
    error::Error,
    event::Events,
};

// =================================================================================================
// Action
// =================================================================================================

// Action

/// A command over the stream: the composite of [`Act`] (business logic),
/// [`Context`] (its projections), [`Select`] (what to replay), and [`Update`]
/// (folding replayed events). Derived by `#[derive(Action)]` and run by an
/// [`Enactor`](crate::enactor::Enactor).
pub trait Action: Act + Context + Select + Update {}

// Act

/// The business logic of an [`Action`]: decide, from the folded [`Context`],
/// what events (if any) to append.
pub trait Act: Context
where
    Self::Err: From<Report<Error>>,
{
    /// The error this action may fail with; must absorb a domain [`Report`]
    /// (`From<Report<Error>>`) so replay/append failures propagate through it.
    type Err;
    /// The success value (`()` by default).
    type Ok = ();

    /// Run the command against its folded `context`, buffering any events to
    /// append (through the context) and returning the success value.
    fn action(&mut self, context: &mut Self::Context) -> Result<Self::Ok, Self::Err>;
}

// Context

/// Supplies an [`Action`]'s context: a generated struct holding the action's
/// projections that derefs to the [`Events`] buffer the action appends into.
pub trait Context
where
    Self::Context: Deref<Target = Events> + DerefMut + Into<Events>,
{
    /// The generated per-action context type.
    type Context;

    /// Build a fresh context, with each projection at its initial state.
    fn context(&self) -> Self::Context;
}

// Select

/// Builds the [`Selection`]s an [`Action`] replays before running — one per
/// projection in its context.
pub trait Select: Context {
    /// The selections to replay (and to guard the append against), derived from
    /// the action's projections.
    fn select(&self, context: &Self::Context) -> Result<Vec<Selection>, Report<Error>>;
}

// Update

/// Folds a replayed event into an [`Action`]'s context, routing it (by mask) to
/// the projections that selected it.
pub trait Update: Context {
    /// Fold `event` into `context`, dispatching it to each projection whose
    /// mask bit it matched.
    fn update(
        &self,
        context: &mut Self::Context,
        event: &EventAndMask,
    ) -> Result<(), Report<Error>>;
}
