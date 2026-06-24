//! The domain layer's error model: the opaque [`struct@Error`] and the
//! [`Result`] alias returned by every fallible domain operation. `error-stack`
//! is used end-to-end; a stream-layer failure is `change_context`'d into this
//! error at the boundary, so the domain never names the stream's error type.
//! (A stream `Conflict` attachment rides through the `change_context`
//! unchanged, so a rejected append is still recovered via
//! `report.downcast_ref::<eventric_stream::error::Conflict>()`.)

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

/// The opaque error type for every fallible domain operation. Detail rides as
/// `.attach(..)` on the `error-stack` report; a stream-layer failure is
/// `change_context`'d into this at the boundary.
#[derive(Debug, Display, Error)]
#[display("domain error")]
pub struct Error;

// -------------------------------------------------------------------------------------------------

// Result

/// The result type for fallible domain operations: an `error-stack` [`Report`]
/// over [`struct@Error`].
pub type Result<T, E = Error> = result::Result<T, Report<E>>;
