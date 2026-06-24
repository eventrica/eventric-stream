//! Eventric is a low-level event-stream abstraction for the Eventrica
//! ecosystem, with append/query consistent with [Dynamic Consistency
//! Boundaries (DCB)](https://dcb.events/).
//!
//! The crate is two layers. The stream layer — [`event`] (the payload `Data`,
//! the queryable `Facets`, and the generic `Event`), [`stream`] (the `Stream`,
//! its `Reader`/`Writer` split, the threaded `Owner`/`Proxy`, and the masked
//! `Condition` query/concurrency model), plus [`error`] and [`utils`] — is the
//! foundation. The [`model`] layer is the intended top-level UX: an
//! event-sourcing model of `Event`s, `Projection`s that fold them into
//! read-model state, and `Action`s (commands) run by a `model::Enactor` under a
//! single DCB `Condition`.
//!
//! The `tag!` macro and the `Event`/`Action`/`Projection` derives are
//! re-exported (from the companion `eventric-macros` crate) at the points they
//! are used: [`event::tag`], and `model::{event::Event, action::Action,
//! projection::Projection}`.

#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_safety_doc)]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![feature(associated_type_defaults)]
#![feature(exclusive_wrapper)]

mod combine;

pub mod error;
pub mod event;
pub mod stream;
pub mod utils;

#[allow(missing_docs)]
pub mod model;

// =================================================================================================
// Eventric
// =================================================================================================
