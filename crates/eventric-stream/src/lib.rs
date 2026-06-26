//! `eventric-stream` is the content-agnostic substrate of the Eventrica
//! ecosystem: a low-level event stream with append/query consistent with
//! [Dynamic Consistency Boundaries (DCB)](https://dcb.events/).
//!
//! It is deliberately ignorant of payload *content*: an event's `Data` is an
//! opaque byte string, and the queryable `Facets` (a `Name`, a `Version`, and
//! `Tag`s) are all it indexes — there is no (de)serialisation here. [`event`]
//! holds the payload `Data`, the `Facets`, and the generic `Event`; [`stream`]
//! holds the `Stream`, its `Reader`/`Writer` split, the threaded
//! `Owner`/`Proxy`, and the masked `Condition` query/concurrency model; with
//! [`error`] and [`utils`] alongside.
//!
//! The higher-level event-sourcing UX — events, projections, and actions —
//! lives in the companion `eventric-domain` crate, which is built on this one.
//! The `tag!` macro is re-exported here (from `eventric-macros`) at
//! [`event::tag`].

#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_safety_doc)]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![feature(exclusive_wrapper)]

mod iter;

pub mod error;
pub mod event;
pub mod stream;
pub mod utils;

// =================================================================================================
// Eventric Stream
// =================================================================================================
