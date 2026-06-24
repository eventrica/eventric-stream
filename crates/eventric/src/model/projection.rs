//! See the `eventric-model` crate for full documentation, including
//! module-level documentation.
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
//! A deliberately deferred extension (named/keyed selectors that would generate
//! per-selector payload enums and a per-selector-method trait) is recorded in
//! `docs/keyed-selectors.md` at the repository root; it is intentionally *not*
//! built today.
//!
//! [`Update`]: crate::model::action::Update

use std::any::Any;

use derive_more::Deref;
use error_stack::{
    Report,
    ResultExt as _,
};
pub use eventric_macros::Projection;
use fancy_constructor::new;

use crate::{
    error::Error,
    model::event::Event,
    stream::{
        Position,
        Timestamp,
        operate::{
            Selection,
            select::EventAndMask,
        },
    },
};

// =================================================================================================
// Projection
// =================================================================================================

// Projection

pub trait Projection: Dispatch + Recognize + Select {}

// Dispatch

pub trait Dispatch {
    fn dispatch(&mut self, event: &DispatchEvent);
}

// Project

pub trait Project<E>
where
    E: Event,
{
    fn project(&mut self, event: ProjectionEvent<'_, E>);
}

// Recognize

pub trait Recognize {
    fn recognize(&self, event: &EventAndMask) -> Result<Option<DispatchEvent>, Report<Error>>;
}

// Select

pub trait Select {
    fn select(&self) -> Result<Selection, Report<Error>>;
}

// -------------------------------------------------------------------------------------------------

// Dispatch Event

#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct DispatchEvent {
    pub event: Box<dyn Any>,
    pub position: Position,
    pub timestamp: Timestamp,
}

impl DispatchEvent {
    #[must_use]
    pub fn as_projection_event<E>(&self) -> Option<ProjectionEvent<'_, E>>
    where
        E: Event + 'static,
    {
        self.event
            .downcast_ref()
            .map(|inner_event| ProjectionEvent::new(inner_event, self.position, self.timestamp))
    }

    pub fn from_event<E>(event: &EventAndMask) -> Result<Self, Report<Error>>
    where
        E: Event + 'static,
    {
        let inner_event = revision::from_slice::<E>(event.event.data().as_ref())
            .change_context(Error)
            .attach("dispatch_event/from_event/from_slice")?;

        Ok(Self::new(
            Box::new(inner_event),
            event.event.meta().position(),
            event.event.meta().timestamp(),
        ))
    }
}

// -------------------------------------------------------------------------------------------------

// Projection Event

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
    #[must_use]
    pub fn position(&self) -> Position {
        self.position
    }

    #[must_use]
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}
