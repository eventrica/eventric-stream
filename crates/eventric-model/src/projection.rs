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
//! projection (`snake_case`), a **borrowed enum** named after the selection
//! (`UpperCamelCase`), with one variant per event type — so the fold target is
//! `<projection_snake_case>::<SelectionUpperCamelCase>` (e.g. selection
//! `capacity` on `CourseCapacity` ⇒ `course_capacity::Capacity`). The user
//! folds each selection by implementing
//! [`Project<Enum>`](Project) — one impl per selection, with that selection's
//! enum (wrapped in an [`Event`], so position/timestamp come along) as the type
//! argument:
//!
//! ```text
//! impl Project<course_capacity::Capacity<'_>> for CourseCapacity {
//!     fn project(&mut self, e: Event<course_capacity::Capacity<'_>>) {
//!         match e.event() {
//!             course_capacity::Capacity::CourseDefined(ev)         => self.capacity = ev.capacity,
//!             course_capacity::Capacity::CourseCapacityChanged(ev) => self.capacity = ev.new_capacity,
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
//! same event type are *separate named selections* (each its own `Project`
//! impl), while a single derived state folded from several event types is *one
//! selection* whose `project` matches the enum. There is no "which selector
//! won" arbitration — matching is set-valued, and the payload is decoded once
//! and shared across every selection (and every same-type projection slot) that
//! matched.
//!
//! ## Dispatch + the mask
//!
//! Each named selection is a separate [`Selection`] in the query, so it has its
//! own positional bit in the stream's mask. The model layer de-positionalises
//! that: [`Dispatch::dispatch`] receives just *this projection's* slice of the
//! mask, and routes each set bit straight to the matching
//! `Project<Enum>::project` — no per-event re-test of the filters.
//! [`Select::SELECTIONS`] is the slice width.
//!
//! # Example
//!
//! A projection with one named selection (`capacity`) folding two event types
//! into a single read-model. The derive generates the `course_capacity` module
//! (the borrowed `Capacity` enum); you implement [`Project<Enum>`](Project):
//!
//! ```
//! # #![allow(dead_code)]
//! use eventric_model::{
//!     event::Event,
//!     projection::{
//!         self,
//!         Project,
//!         Projection,
//!         Select,
//!     },
//! };
//! use revision::revisioned;
//!
//! #[revisioned(revision = 1)]
//! #[derive(Event)]
//! #[event(identifier: course_defined, tags: { course: id })]
//! struct CourseDefined {
//!     id: String,
//!     capacity: u8,
//! }
//!
//! #[revisioned(revision = 1)]
//! #[derive(Event)]
//! #[event(identifier: course_capacity_changed, tags: { course: id })]
//! struct CourseCapacityChanged {
//!     id: String,
//!     new_capacity: u8,
//! }
//!
//! #[derive(Projection)]
//! #[projection(selections: {
//!     capacity: { events: [CourseDefined, CourseCapacityChanged], filter: { course: id } },
//! })]
//! struct CourseCapacity {
//!     id: String,
//!     capacity: u8,
//! }
//!
//! impl Project<course_capacity::Capacity<'_>> for CourseCapacity {
//!     fn project(&mut self, event: projection::Event<course_capacity::Capacity<'_>>) {
//!         match event.event() {
//!             course_capacity::Capacity::CourseDefined(e) => self.capacity = e.capacity,
//!             course_capacity::Capacity::CourseCapacityChanged(e) => {
//!                 self.capacity = e.new_capacity
//!             }
//!         }
//!     }
//! }
//!
//! // The derive generated one named selection, so the projection owns one mask bit.
//! fn main() {
//!     assert_eq!(CourseCapacity::SELECTIONS, 1);
//! }
//! ```
//!
//! The `multi_selector_projections` example folds projections like this against
//! a real stream.

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
    event,
};

// =================================================================================================
// Projection
// =================================================================================================

// Projection

/// A read-model built by folding selected events: the composite of [`Select`]
/// (what events, as named selections), [`Recognize`] (type-match + decode), and
/// [`Dispatch`] (fold via the matching [`Project<Enum>`](Project)). Derived by
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
/// is one this projection folds, decodes it into a [`Recognized`].
pub trait Recognize {
    /// Decode `event` into a [`Recognized`] if its type is one this
    /// projection folds, else `None`.
    fn recognize(&self, event: &EventAndMask) -> Result<Option<Recognized>, Report<Error>>;
}

// Dispatch

/// Folds a recognised event into the projection, routing by `mask` — this
/// projection's per-selection bit slice — to the matching
/// [`Project<Enum>::project`](Project::project).
pub trait Dispatch {
    /// Fold `event` into every selection whose bit is set in `mask` (the slice
    /// of the query mask owned by this projection, one bit per named
    /// selection).
    fn dispatch(&mut self, mask: &[bool], event: &Recognized);
}

// -------------------------------------------------------------------------------------------------

// Recognized

/// A decoded event ready to fold: its boxed payload (downcast into the matching
/// selection's enum) plus the persisted position and timestamp. Decoded once
/// per recognised event and shared across every selection and same-type
/// projection slot that matched.
#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct Recognized {
    /// The decoded payload, type-erased; downcast to the concrete event type.
    pub event: Box<dyn Any>,
    /// The event's position in the stream.
    pub position: Position,
    /// The event's timestamp.
    pub timestamp: Timestamp,
}

impl Recognized {
    /// Decode a persisted `event`'s payload into an `E` (via `revision`),
    /// paired with its position and timestamp. A decode failure carries the
    /// stored version and the revision this consumer handles.
    pub fn from_event<E>(event: &EventAndMask) -> Result<Self, Report<Error>>
    where
        E: event::Event + 'static,
    {
        let inner = revision::from_slice::<E>(event.event.data().as_ref())
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
            Box::new(inner),
            event.event.meta().position(),
            event.event.meta().timestamp(),
        ))
    }
}

// -------------------------------------------------------------------------------------------------

// Event

/// A matched event handed to a selection method: the selection's borrowed enum
/// (accessed via [`event`](Self::event)), with the persisted position and
/// timestamp available alongside.
#[derive(new, Debug)]
#[new(vis(pub))]
pub struct Event<T> {
    inner: T,
    position: Position,
    timestamp: Timestamp,
}

impl<T> Event<T> {
    /// The matched event — the selection's enum (a variant per event type).
    #[must_use]
    pub fn event(&self) -> &T {
        &self.inner
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

// -------------------------------------------------------------------------------------------------

// Project

/// The fold for one named selection — implemented once per selection, with that
/// selection's generated borrowed enum as the type argument (e.g. `impl
/// Project<course_capacity::Capacity<'_>> for CourseCapacity`). The derive's
/// [`Dispatch`] routes each matched event to the matching
/// `Project<Enum>::project`, keyed by the mask. The enum's `match` is
/// compile-time exhaustive, so adding or removing an event type in a selection
/// forces the fold to be updated.
pub trait Project<T> {
    /// Fold the matched `event` — the selection's enum (a variant per event
    /// type), with its position and timestamp — into this read-model.
    fn project(&mut self, event: Event<T>);
}
