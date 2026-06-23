//! The crate's error model: the opaque [`struct@Error`], the [`Conflict`]
//! marker attached when an append is rejected by its condition, and the
//! [`Result`] alias returned by every fallible operation. `error-stack` is used
//! end-to-end, so detail rides as `.attach(..)` on the report rather than as
//! error variants.

use std::result;

use derive_more::{
    Debug,
    Display,
    Error,
};
use error_stack::Report;

// =================================================================================================
// Error
// =================================================================================================

// Error

/// The opaque error type for every fallible stream operation. Detail rides as
/// `.attach(..)` on the `error-stack` report; a rejected append additionally
/// attaches the [`Conflict`] marker.
#[derive(Debug, Display, Error)]
#[display("stream error")]
pub struct Error;

// -------------------------------------------------------------------------------------------------

// Conflict

/// Marker attached to an [`struct@Error`] report when an append is rejected by
/// its condition (an optimistic-concurrency / DCB conflict). Distinguish a
/// conflict from any other failure with `report.downcast_ref::<Conflict>()`.
///
/// An index-read failure while evaluating the condition surfaces as a plain
/// [`struct@Error`] with no `Conflict` attached, so the absence of this marker
/// does not imply the append would otherwise have succeeded.
#[derive(Debug, Display)]
#[display("append condition conflict")]
pub struct Conflict;

// -------------------------------------------------------------------------------------------------

// Result

/// The result type for fallible stream operations: an `error-stack` [`Report`]
/// over [`struct@Error`].
pub type Result<T, E = Error> = result::Result<T, Report<E>>;
