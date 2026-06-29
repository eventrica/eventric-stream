//! `eventric-runtime` is the mechanism that *runs* `eventric-model` user code:
//! the [`enactor::Enactor`] replays an `Action`'s selected events and appends
//! its result under a single DCB condition, and the [`reactor::Reactor`] drives
//! a [`React`](eventric_model::reaction::React)ion over a stream, folding
//! matching events into a view. It is the node's Runtime in crate form — the
//! channel and observability will join these over time. It depends on
//! `eventric-model` (and `eventric-stream`); the model never depends on it.

#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![deny(missing_docs)]

pub mod enactor;
pub mod reactor;

// =================================================================================================
// Eventric Runtime
// =================================================================================================
