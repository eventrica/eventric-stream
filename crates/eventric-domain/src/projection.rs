//! Projections: the [`Projection`] trait, which folds selected events into
//! read-model state via **named selections**.
//!
//! # Named selections
//!
//! A `#[derive(Projection)]` declares one or more *named* selections — each a
//! set of event types plus an optional tag filter:
//!
//! ```text
//! #[projection(selections: {
//!     capacity: { events: [CourseDefined, CourseCapacityChanged], filter: { course: id } },
//! })]
//! ```
//!
//! For each selection the derive generates, in a module named after the
//! projection (`snake_case`), a **borrowed enum** with one variant per event
//! type, and a `Project` trait with **one method per selection** taking that
//! enum wrapped in a [`ProjectionEvent`] (so position/timestamp come along).
//! The user implements that trait — one method per selection:
//!
//! ```text
//! impl course_capacity::Project for CourseCapacity {
//!     fn capacity(&mut self, e: ProjectionEvent<course_capacity::Capacity<'_>>) {
//!         match e.event() {
//!             Capacity::CourseDefined(ev)         => self.capacity = ev.capacity,
//!             Capacity::CourseCapacityChanged(ev) => self.capacity = ev.new_capacity,
//!         }
//!     }
//! }
//! ```
//!
//! Adding or removing an event type in a selection changes the enum, so every
//! `match` over it must be updated — a **compile-time** prompt, not a silent
//! drop.
//!
//! ## Shaping inputs
//!
//! Each named selection is its own input channel: distinct read-models over the
//! same event type are *separate named selections* (each its own method), while
//! a single derived state folded from several event types is *one selection*
//! whose method matches the enum. There is no "which selector won" arbitration
//! — matching is set-valued, and the payload is decoded once and shared across
//! every selection (and every same-type projection slot) that matched.
//!
//! ## Dispatch + the mask
//!
//! Each named selection is a separate [`Selection`] in the query, so it has its
//! own positional bit in the stream's mask. The model layer de-positionalises
//! that: [`Dispatch::dispatch`] receives just *this projection's* slice of the
//! mask, and routes each set bit straight to its selection's method — no
//! per-event re-test of the filters. [`Select::SELECTIONS`] is the slice width.

use std::any::Any;

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
/// (what events, as named selections), [`Recognize`] (type-match + decode), and
/// [`Dispatch`] (fold into the matching selection's method). Derived by
/// `#[derive(Projection)]`.
pub trait Projection: Dispatch + Recognize + Select {}

// Select

/// Builds the [`Selection`]s this projection folds — one per named selection,
/// in declaration order.
pub trait Select {
    /// The number of named selections, i.e. the width of this projection's
    /// slice of the query mask.
    const SELECTIONS: usize;

    /// One [`Selection`] per named selection, in declaration order.
    fn select(&self) -> Result<Vec<Selection>, Report<Error>>;
}

// Recognize

/// Matches a persisted event to this projection by hashed name and, if its type
/// is one this projection folds, decodes it into a [`DispatchEvent`].
pub trait Recognize {
    /// Decode `event` into a [`DispatchEvent`] if its type is one this
    /// projection folds, else `None`.
    fn recognize(&self, event: &EventAndMask) -> Result<Option<DispatchEvent>, Report<Error>>;
}

// Dispatch

/// Folds a recognised event into the projection, routing by `mask` — this
/// projection's per-selection bit slice — to the matching selection method(s).
pub trait Dispatch {
    /// Fold `event` into every selection whose bit is set in `mask` (the slice
    /// of the query mask owned by this projection, one bit per named
    /// selection).
    fn dispatch(&mut self, mask: &[bool], event: &DispatchEvent);
}

// -------------------------------------------------------------------------------------------------

// Dispatch Event

/// A decoded event ready to fold: its boxed payload (downcast into the matching
/// selection's enum) plus the persisted position and timestamp. Decoded once
/// per recognised event and shared across every selection and same-type
/// projection slot that matched.
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

/// A matched event handed to a selection method: the selection's borrowed enum
/// (accessed via [`event`](Self::event)), with the persisted position and
/// timestamp available alongside.
#[derive(new, Debug)]
#[new(vis(pub))]
pub struct ProjectionEvent<T> {
    event: T,
    position: Position,
    timestamp: Timestamp,
}

impl<T> ProjectionEvent<T> {
    /// The matched event — the selection's enum (a variant per event type).
    #[must_use]
    pub fn event(&self) -> &T {
        &self.event
    }

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
