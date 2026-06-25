//! Procedural macros for the eventric workspace: the `tag!` function-like macro
//! and the `Event`/`Action`/`Projection` derives. `tag!` is re-exported from
//! `eventric-stream` and the three derives from `eventric-domain`, so consumers
//! never name this crate directly.

#![allow(clippy::multiple_crate_versions)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_safety_doc)]
#![deny(missing_docs)]

pub(crate) mod action;
pub(crate) mod event;
pub(crate) mod projection;
pub(crate) mod tag;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

use crate::{
    action::Action,
    event::Event,
    projection::Projection,
    tag::Tag,
};

// =================================================================================================
// Eventric Macros
// =================================================================================================

macro_rules! emit_impl_or_error {
    ($e:expr) => {
        match $e {
            Ok(val) => val.into_token_stream(),
            Err(err) => err.write_errors(),
        }
    };
}

// Tag

/// Creates an `eventric_stream::event::Tag` from an identifier-compatible
/// prefix and a value which implements `Display`, e.g. `tag!(student,
/// &self.id)?`.
#[proc_macro]
pub fn tag(input: TokenStream) -> TokenStream {
    emit_impl_or_error!(Tag::new(input.into())).into()
}

// Action

/// Derives the domain `Action` trait family from a declarative `#[action(..)]`
/// attribute — generates the action's projections type (a struct in a
/// `snake_case` submodule) and its select/update wiring.
///
/// ```text
/// #[action(projections: {
///     <field_name>: <Type>::new(..),   // field name is the projection-field key;
/// })]                                  // the type is read from the constructor
/// ```
///
/// Each entry names a projection field and its constructor (run with `self`
/// bound to the action). `Act::act(&self, events, projections)` then stages
/// output into the events buffer, reading the folded projections. Omit
/// `projections` for an action with none. See `eventric_domain::action`.
#[proc_macro_derive(Action, attributes(action))]
pub fn action(input: TokenStream) -> TokenStream {
    emit_impl_or_error!(Action::new(&parse_macro_input!(input))).into()
}

// Event

/// Derives the domain `Event` trait family for a struct from a declarative
/// `#[event(..)]` attribute.
///
/// It generates `Identifier` (the event type's stable name) and `Tags` (its
/// queryable tags), plus the `Event` marker; `Specifier` then follows by a
/// blanket impl. The struct must **also** carry `#[revisioned(revision = N)]`
/// (the `revision` crate), which supplies the payload (de)serialisation the
/// `Event` trait requires.
///
/// # Grammar
///
/// ```text
/// #[event(
///     identifier: <ident>,                              // required
///     tags: { <prefix>: <value>, <prefix>: [<v>, ..] }, // optional
/// )]
/// ```
///
/// - **`identifier`** (required) is the event type's **persisted** identity —
///   it is hashed into the stream's type index. It is deliberately explicit and
///   decoupled from the Rust type name, so renaming the `struct` never silently
///   re-identifies already-stored events.
/// - **`tags`** (optional; omit entirely if there are none) is a map of
///   `<prefix>: <value>` entries. The prefix need not match the field (`course:
///   course_id` tags under `course` from the `course_id` field), and a value
///   may be a `[list]` to emit several tags under one prefix (e.g. a transfer
///   tagged `account: [from, to]`).
///
/// ## Tag values
///
/// Each `<value>` is one of:
///
/// ```text
/// tags: {
///     course:  id,                    // bare ident — the field, i.e. `&self.id`
///     student: &self.student_id,      // expression — `self` is the event
///     region:  |e| e.region(),        // closure    — as the expression, but you
///                                     //              name (or `_`-ignore) the receiver
///     account: [from, to],            // list       — one tag per element, same prefix
/// }
/// ```
///
/// The bare ident is shorthand for a plain field. Otherwise the value is an
/// expression evaluated with the event in scope as `self`; the closure form is
/// the same, but lets you bind a different receiver name (e.g. `|_| "literal"`
/// to ignore it, or a multi-statement body); and a `[list]` of any of these
/// emits one tag per element under the prefix. Whatever the form, each value
/// becomes the tag's text — formatted as `prefix:value`, so it must be
/// `Display` — in practice a string field like `&self.id`.
///
/// # Example
///
/// ```rust,ignore
/// #[revisioned(revision = 1)]
/// #[derive(Event)]
/// #[event(
///     identifier: student_subscribed_to_course,
///     tags: {
///         course: course_id,
///         student: student_id,
///     },
/// )]
/// struct StudentSubscribedToCourse {
///     course_id: String,
///     student_id: String,
/// }
/// ```
///
/// (A runnable version is in the `eventric_domain::event` module docs.)
#[proc_macro_derive(Event, attributes(event))]
pub fn event(input: TokenStream) -> TokenStream {
    emit_impl_or_error!(Event::new(&parse_macro_input!(input))).into()
}

// Projection

/// Derives the domain `Projection` trait family
/// (`Dispatch`/`Recognize`/`Select`) from a declarative `#[projection(..)]`
/// attribute of **named selections**:
///
/// ```text
/// #[projection(selections: {
///     <name>: { events: [<Type>, ..], filter: { <prefix>: <value>, .. } }, // filter optional
/// })]
/// ```
///
/// For each selection it generates, in a module named after the projection
/// (`snake_case`), a borrowed enum (one variant per event type). The user folds
/// each selection by implementing the standard `Project<Enum>` trait (in
/// `eventric_domain::projection`) — one impl per selection, with that
/// selection's enum as the type argument. See `eventric_domain::projection`.
#[proc_macro_derive(Projection, attributes(projection))]
pub fn projection(input: TokenStream) -> TokenStream {
    emit_impl_or_error!(Projection::new(&parse_macro_input!(input))).into()
}
