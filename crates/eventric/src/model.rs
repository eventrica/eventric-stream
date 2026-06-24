//! The business-logic layer over the stream: an event-sourcing model of
//! [`event::Event`]s, [`projection::Projection`]s that fold selected events
//! into read-model state, and [`action::Action`]s (commands) run by an
//! [`enactor::Enactor`] under a single DCB
//! [`crate::stream::operate::Condition`].

pub mod action;
pub mod enactor;
pub mod event;
pub mod projection;

// =================================================================================================
// Model
// =================================================================================================
