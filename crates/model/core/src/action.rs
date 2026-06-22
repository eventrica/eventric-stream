//! See the `eventric-surface` crate for full documentation, including
//! module-level documentation.

use std::ops::{
    Deref,
    DerefMut,
};

use error_stack::Report;
use eventric_stream::{
    error::Error,
    stream::{
        EventAndMask,
        Selection,
    },
};

use crate::event::Events;

// =================================================================================================
// Action
// =================================================================================================

// Action

pub trait Action: Act + Context + Select + Update {}

// Act

pub trait Act: Context
where
    Self::Err: From<Report<Error>>,
{
    type Err;
    type Ok = ();

    fn action(&mut self, context: &mut Self::Context) -> Result<Self::Ok, Self::Err>;
}

// Context

pub trait Context
where
    Self::Context: Deref<Target = Events> + DerefMut + Into<Events>,
{
    type Context;

    fn context(&self) -> Self::Context;
}

// Select

pub trait Select: Context {
    fn select(&self, context: &Self::Context) -> Result<Vec<Selection>, Report<Error>>;
}

// Update

pub trait Update: Context {
    fn update(
        &self,
        context: &mut Self::Context,
        event: &EventAndMask,
    ) -> Result<(), Report<Error>>;
}
