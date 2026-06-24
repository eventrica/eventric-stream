//! Projections: the [`Projection`] trait, which folds selected events into
//! read-model state.
//!
//! # Multi-selector projections: the two-tools rule
//!
//! A `#[derive(Projection)]` may declare more than one `select(..)` clause, and
//! a single `select(..)` may name more than one event type. There are exactly
//! two tools for shaping what a projection consumes, and they mean different
//! things:
//!
//! 1. **Many `select(..)` clauses in one projection** — "these events are all
//!    inputs to *one* piece of derived state; I do not care which clause
//!    matched." The clauses OR together into a single [`Selection`], i.e. one
//!    mask bit. The projection then discriminates finer than the OR (and finer
//!    than type) *inside* its [`Project`] impls, by reading the decoded
//!    payload.
//!
//! 2. **One projection per filter** — "these are *distinct* read-models over
//!    overlapping events." Each projection gets its own mask bit, its own
//!    [`Selection`], and its own [`Project`] impls. (Live example, in the
//!    `course_subscriptions` example: `NumberOfCourseSubscriptions` and
//!    `NumberOfStudentSubscriptions` fold the same `StudentSubscribedToCourse`
//!    event under different tag filters — so they are two projections, not two
//!    selectors on one.)
//!
//! ## Routing is by type; discrimination is by payload
//!
//! An event is routed to a projection **by its event type** via `Project<E>`,
//! and that routing is enforced by Rust's coherence rules: you cannot write two
//! `impl Project<Transfer>` for the same projection, so each type lands in
//! exactly one handler. Any discrimination *finer than type* is the
//! projection's own business and is done from the **payload** — and that is the
//! right place for it, because the tags were derived from the payload fields in
//! the first place. The payload is therefore the canonical, type-checked, and
//! more expressive source; re-deriving a distinction from tags would be a
//! lossy, stringly-typed detour around data you already hold.
//!
//! ## Multi-match is set-valued and cheap
//!
//! Matching is set-valued and free: an event that satisfies several projections
//! sets several mask bits, and every matching projection folds it — there is no
//! "which selector won" arbitration. Nor is there a decode penalty for
//! splitting state across separate projections: [`Update`] recognises (decodes)
//! the payload **once** and shares the boxed [`DispatchEvent`] across every
//! same-type slot in the action's context.
//!
//! A redesign that replaces this `Project<E>` surface with named selectors
//! (per-selection event enums + a per-selection method trait), as part of a
//! declarative derive-grammar overhaul, is designed in `docs/derives.md`; it is
//! not yet built.
//!
//! [`Update`]: crate::action::Update

use std::any::Any;

use derive_more::Deref;
use error_stack::{
    Report,
    ResultExt as _,
};
pub use eventric_macros::Projection;
use eventric_stream::stream::{
    Position,
    Timestamp,
    operate::{
        Selection,
        select::EventAndMask,
    },
};
use fancy_constructor::new;

use crate::{
    error::Error,
    event::Event,
};

// =================================================================================================
// Projection
// =================================================================================================

// Projection

/// A read-model built by folding selected events: the composite of [`Select`]
/// (what events), [`Recognize`] (type-match + decode), and [`Dispatch`] (fold).
/// Derived by `#[derive(Projection)]`.
pub trait Projection: Dispatch + Recognize + Select {}

// Dispatch

/// Folds a recognised event into the projection's state, routing by payload
/// type to the matching [`Project`] impl.
pub trait Dispatch {
    /// Dispatch a decoded [`DispatchEvent`] into this projection's fold.
    fn dispatch(&mut self, event: &DispatchEvent);
}

// Project

/// Folds a single event type `E` into the projection. One impl per event type
/// the projection consumes — coherence makes the routing exhaustive.
pub trait Project<E>
where
    E: Event,
{
    /// Fold one `E` (with its position/timestamp) into the projection's state.
    fn project(&mut self, event: ProjectionEvent<'_, E>);
}

// Recognize

/// Matches a persisted event to this projection by hashed name and, if it
/// matches, decodes it into a [`DispatchEvent`].
pub trait Recognize {
    /// Decode `event` into a [`DispatchEvent`] if its type is one this
    /// projection folds, else `None`.
    fn recognize(&self, event: &EventAndMask) -> Result<Option<DispatchEvent>, Report<Error>>;
}

// Select

/// Builds the [`Selection`] of events this projection folds.
pub trait Select {
    /// The selection — the OR of the projection's `select(..)` clauses.
    fn select(&self) -> Result<Selection, Report<Error>>;
}

// -------------------------------------------------------------------------------------------------

// Dispatch Event

/// A decoded event ready to fold: its boxed payload (downcast per [`Project`]
/// impl) plus the persisted position and timestamp. Decoded once per recognised
/// event and shared across every same-type projection slot.
#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct DispatchEvent {
    /// The decoded payload, type-erased; downcast to the concrete event type.
    pub event: Box<dyn Any>,
    /// The event's position in the stream.
    pub position: Position,
    /// The event's timestamp.
    pub timestamp: Timestamp,
}

impl DispatchEvent {
    /// View the boxed payload as a [`ProjectionEvent`] of `E`, if it is an `E`.
    #[must_use]
    pub fn as_projection_event<E>(&self) -> Option<ProjectionEvent<'_, E>>
    where
        E: Event + 'static,
    {
        self.event
            .downcast_ref()
            .map(|inner_event| ProjectionEvent::new(inner_event, self.position, self.timestamp))
    }

    /// Decode a persisted `event`'s payload into an `E` (via `revision`),
    /// paired with its position and timestamp. A decode failure carries the
    /// stored version and the revision this consumer handles.
    pub fn from_event<E>(event: &EventAndMask) -> Result<Self, Report<Error>>
    where
        E: Event + 'static,
    {
        let inner_event = revision::from_slice::<E>(event.event.data().as_ref())
            .change_context(Error)
            .attach_with(|| {
                format!(
                    "failed to decode event payload (stored version {:?}; this consumer handles \
                     revision {})",
                    event.event.facets().ty().version(),
                    E::revision(),
                )
            })?;

        Ok(Self::new(
            Box::new(inner_event),
            event.event.meta().position(),
            event.event.meta().timestamp(),
        ))
    }
}

// -------------------------------------------------------------------------------------------------

// Projection Event

/// A decoded event handed to a [`Project`] impl: derefs to the payload `&E`,
/// with the persisted position and timestamp available alongside.
#[derive(new, Debug, Deref)]
#[new(const_fn, vis(pub(crate)))]
pub struct ProjectionEvent<'a, E>
where
    E: Event,
{
    #[deref]
    event: &'a E,
    position: Position,
    timestamp: Timestamp,
}

impl<E> ProjectionEvent<'_, E>
where
    E: Event,
{
    /// The event's position in the stream.
    #[must_use]
    pub fn position(&self) -> Position {
        self.position
    }

    /// The event's timestamp.
    #[must_use]
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}
