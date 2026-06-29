//! `eventric-runtime` is the mechanism that *runs* `eventric-model` user code:
//! the [`enactor::Enactor`] replays an `Action`'s selected events, runs its
//! business logic, and appends the result against a stream under a single DCB
//! condition. It is the node's Runtime in crate form — the reaction reactor,
//! the channel, and observability will join the `Enactor` here over time. It
//! depends on `eventric-model` (and `eventric-stream`); the model never depends
//! on it.

#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![deny(missing_docs)]

pub mod enactor;

// =================================================================================================
// Eventric Runtime
// =================================================================================================
