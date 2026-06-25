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

/// Derives the domain `Action` trait family (`Context`/`Select`/`Update`) for a
/// command from a declarative `#[action(..)]` attribute — an action reads its
/// projections, then decides what events (if any) to append.
///
/// It generates a `Projections` struct (in a module named after the action,
/// `snake_case`) with one field per entry, plus the wiring that builds and
/// folds it. You write the business logic by implementing the standard
/// `Act<Projections>` trait, with the generated `Projections` struct as the
/// type argument — mirroring how a projection implements
/// [`Project<Enum>`](macro@Projection).
///
/// # Grammar
///
/// ```text
/// #[action(projections: {
///     <field_name>: <Type>::new(..),   // field name keys the context field;
///     // .. more ..                     // the projection type is read from the constructor
/// })]                                  // (omit `projections` entirely for an action with none)
/// ```
///
/// - Each entry pairs a **context-field name** with a **projection
///   constructor**. The constructor runs with `self` bound to the action (so it
///   can read the action's fields, e.g. `Balance::new(&self.account)`), and the
///   projection type is inferred from the constructor's path — so it must be a
///   `Type::new(..)`-style call. The explicit field name lets two slots of the
///   same projection type coexist.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Action)]
/// #[action(projections: {
///     balance: AccountBalance::new(&self.account),
/// })]
/// struct Withdraw {
///     account: String,
///     amount: u64,
/// }
///
/// impl Act<withdraw::Projections> for Withdraw {
///     fn act(&self, events: &mut Events, projections: &withdraw::Projections)
///         -> Result<Self::Ok, Self::Err>
///     {
///         if projections.balance.balance < self.amount as i64 {
///             return Err(Report::new(Error).attach("insufficient funds"));
///         }
///         events.append(&Withdrawn::new(&self.account, self.amount))?;
///         Ok(())
///     }
/// }
/// ```
///
/// (A runnable version is in the `eventric_domain::action` module docs; the
/// `course_subscriptions` example runs actions via `Enactor::enact`.)
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
/// (`Select`/`Recognize`/`Dispatch`) for a read-model from a declarative
/// `#[projection(..)]` attribute of **named selections** — each a set of event
/// types plus an optional tag filter.
///
/// For each selection it generates, in a module named after the projection
/// (`snake_case`), a **borrowed enum** with one variant per event type. You
/// fold each selection by implementing the standard `Project<Enum>` trait (one
/// impl per selection), with that selection's generated enum as the type
/// argument; the enum's `match` is **compile-time exhaustive**, so adding or
/// removing an event type forces the fold to be updated rather than silently
/// dropping or mis-folding events.
///
/// # Grammar
///
/// ```text
/// #[projection(selections: {
///     <name>: {
///         events: [<Type>, ..],               // required — the event types this selection folds
///         filter: { <prefix>: <value>, .. },  // optional — tags scoping it to one entity
///     },
///     // .. more named selections ..
/// })]
/// ```
///
/// - **`<name>`** keys the selection: it names the generated enum
///   (`UpperCamelCase`) and owns one bit of the query mask. *Distinct*
///   read-models over the same event type are *separate* selections (each its
///   own `Project` impl); a *single* state folded from several event types is
///   *one* selection whose `match` discriminates.
/// - **`events`** (required) lists the event types folded into this selection.
/// - **`filter`** (optional) scopes the selection by tags, reusing the
///   [`Event`] derive's value forms (bare ident / `self`-expression / closure /
///   `[list]`) — but here `self` is the **projection**, not the event (the
///   filter is built in the projection's `select`, before any event is read),
///   so a bare `account` is `&self.account`, reading the projection's own
///   field.
///
/// # Example
///
/// ```rust,ignore
/// // One selection folding two event types into a single running balance.
/// #[derive(Projection)]
/// #[projection(selections: {
///     balance: { events: [Deposited, Withdrawn], filter: { account: account } },
/// })]
/// struct AccountBalance {
///     account: String,
///     balance: i64,
/// }
///
/// impl Project<account_balance::Balance<'_>> for AccountBalance {
///     fn project(&mut self, event: projection::Event<account_balance::Balance<'_>>) {
///         match event.event() {
///             account_balance::Balance::Deposited(e) => self.balance += e.amount as i64,
///             account_balance::Balance::Withdrawn(e) => self.balance -= e.amount as i64,
///         }
///     }
/// }
/// ```
///
/// (A runnable version is in the `eventric_domain::projection` module docs; the
/// `multi_selector_projections` example folds projections against a real
/// stream.)
#[proc_macro_derive(Projection, attributes(projection))]
pub fn projection(input: TokenStream) -> TokenStream {
    emit_impl_or_error!(Projection::new(&parse_macro_input!(input))).into()
}
