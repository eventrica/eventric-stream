//! Commands: the [`Action`] trait (with its [`Act`]/[`Context`]/[`Select`]/
//! [`Update`] components), run by the model
//! [`Enactor`](super::enactor::Enactor).

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
pub trait Action: Context + Act<Self::Projections> + Select + Update {}

// Act

/// The business logic of an [`Action`]: decide, from its folded projections,
/// what events (if any) to append. Implemented with the action's generated
/// `Projections` struct as the type argument (e.g.
/// `impl Act<make_deposit::Projections> for MakeDeposit`), mirroring how a
/// projection implements `Project<Enum>` — the
/// [`Enactor`](crate::enactor::Enactor) supplies the argument from
/// [`Context::Projections`].
pub trait Act<P>
where
    Self::Err: From<Report<Error>>,
{
    /// The error this action may fail with; must absorb a domain [`Report`]
    /// (`From<Report<Error>>`) so replay/append failures propagate through it.
    type Err;
    /// The success value (`()` by default).
    type Ok = ();

    /// Run the command against its folded `projections`, staging any events to
    /// append into `events`, and returning the success value.
    fn act(&self, events: &mut Events, projections: &P) -> Result<Self::Ok, Self::Err>;
}

// Context

/// Supplies an [`Action`]'s projections: a generated struct (in a module named
/// after the action) holding each of its projections — what the replay folds
/// into and the business logic reads. Separate from the [`Events`] the action
/// appends.
pub trait Context {
    /// The generated per-action projections struct.
    type Projections;

    /// Build the projections, each at its initial (pre-replay) state.
    fn projections(&self) -> Self::Projections;
}

// Select

/// Builds the [`Selection`]s an [`Action`] replays before running — one per
/// *named selection* across its projections (a projection with N named
/// selections contributes N), flattened in projection order to form the mask
/// layout.
pub trait Select: Context {
    /// The selections to replay (and to guard the append against) — one per
    /// named selection across the action's projections, in projection
    /// order.
    fn select(&self, projections: &Self::Projections) -> Result<Vec<Selection>, Report<Error>>;
}

// Update

/// Folds a replayed event into an [`Action`]'s projections, routing it (by
/// mask) to the projections that selected it.
pub trait Update: Context {
    /// Fold `event` into `projections`, dispatching it to each projection whose
    /// mask bit it matched.
    fn update(
        &self,
        projections: &mut Self::Projections,
        event: &EventAndMask,
    ) -> Result<(), Report<Error>>;
}
