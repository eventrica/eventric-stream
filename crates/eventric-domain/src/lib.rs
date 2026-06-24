//! `eventric-domain` is the event-sourcing UX over the content-agnostic
//! `eventric-stream` substrate: an [`event::Event`] model whose payloads
//! (de)serialise via `revision`, [`projection::Projection`]s that fold selected
//! events into read-model state, and [`action::Action`]s (commands) run by an
//! [`enactor::Enactor`] under a single DCB
//! [`eventric_stream::stream::operate::Condition`].
//!
//! This crate knows about *content* (it (de)serialises payloads); the substrate
//! beneath it does not. The `Event`/`Action`/`Projection` derives are
//! re-exported here (from `eventric-macros`) at [`event::Event`],
//! [`action::Action`], and [`projection::Projection`].

#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![deny(missing_docs)]
#![feature(associated_type_defaults)]

pub mod action;
pub mod enactor;
pub mod error;
pub mod event;
pub mod projection;

// =================================================================================================
// Eventric Domain
// =================================================================================================
