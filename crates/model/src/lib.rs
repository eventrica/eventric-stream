//! The business-logic layer over `eventric-stream`: an event-sourcing model of
//! `Event`s, `Projection`s that fold them into read-model state, and `Action`s
//! (commands) run by an [`Enactor`] under a single DCB `Condition`.

#![allow(clippy::multiple_crate_versions)]
#![deny(missing_docs)]

// =================================================================================================
// Eventric Model
// =================================================================================================

pub mod action {
    //! Commands: the [`Action`] (and [`Act`]/[`Context`]/[`Select`]/[`Update`])
    //! traits, plus the `#[derive(Action)]` macro.

    pub use eventric_model_core::action::{
        Act,
        Action,
        Context,
        Select,
        Update,
    };
    pub use eventric_model_macros::Action;
}

pub mod event {
    //! Events: the [`Event`] trait (and its
    //! [`Identifier`]/[`Specifier`]/[`Tags`] components), the [`Events`]
    //! buffer, and the `#[derive(Event)]` macro.

    pub use eventric_model_core::event::{
        Event,
        Events,
        Identifier,
        Specifier,
        Tags,
    };
    pub use eventric_model_macros::Event;
}

pub mod projection {
    //! Projections: the [`Projection`] trait (and its [`Dispatch`]/[`Project`]/
    //! [`Recognize`]/[`Select`] components, plus [`DispatchEvent`]/
    //! [`ProjectionEvent`]), and the `#[derive(Projection)]` macro.

    pub use eventric_model_core::projection::{
        Dispatch,
        DispatchEvent,
        Project,
        Projection,
        ProjectionEvent,
        Recognize,
        Select,
    };
    pub use eventric_model_macros::Projection;
}

pub use eventric_model_core::core::Enactor;
