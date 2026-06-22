//! See the `eventric-surface` crate for full documentation, including
//! module-level documentation.

use std::any::Any;

use derive_more::Deref;
use error_stack::{
    Report,
    ResultExt as _,
};
use eventric_stream::{
    error::Error,
    stream::{
        EventAndMask,
        Position,
        Selection,
        Timestamp,
    },
};
use fancy_constructor::new;

use crate::event::Event;

// =================================================================================================
// Projection
// =================================================================================================

// Projection

pub trait Projection: Dispatch + Recognize + Select {}

// Dispatch

pub trait Dispatch {
    fn dispatch(&mut self, event: &DispatchEvent);
}

// Project

pub trait Project<E>
where
    E: Event,
{
    fn project(&mut self, event: ProjectionEvent<'_, E>);
}

// Recognize

pub trait Recognize {
    fn recognize(&self, event: &EventAndMask) -> Result<Option<DispatchEvent>, Report<Error>>;
}

// Select

pub trait Select {
    fn select(&self) -> Result<Selection, Report<Error>>;
}

// -------------------------------------------------------------------------------------------------

// Dispatch Event

#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct DispatchEvent {
    pub event: Box<dyn Any>,
    pub position: Position,
    pub timestamp: Timestamp,
}

impl DispatchEvent {
    #[must_use]
    pub fn as_projection_event<E>(&self) -> Option<ProjectionEvent<'_, E>>
    where
        E: Event + 'static,
    {
        self.event
            .downcast_ref()
            .map(|inner_event| ProjectionEvent::new(inner_event, self.position, self.timestamp))
    }

    pub fn from_event<E>(event: &EventAndMask) -> Result<Self, Report<Error>>
    where
        E: Event + 'static,
    {
        let inner_event = revision::from_slice::<E>(event.event.data().as_ref())
            .change_context(Error)
            .attach("dispatch_event/from_event/from_slice")?;

        Ok(Self::new(
            Box::new(inner_event),
            event.event.meta().position(),
            event.event.meta().timestamp(),
        ))
    }
}

// -------------------------------------------------------------------------------------------------

// Projection Event

#[derive(new, Debug, Deref)]
#[new(const_fn, vis(pub(crate)))]
pub struct ProjectionEvent<'a, E>
where
    E: Event,
{
    #[deref]
    event: &'a E,
    position: Position,
    timestamp: Timestamp,
}

impl<E> ProjectionEvent<'_, E>
where
    E: Event,
{
    #[must_use]
    pub fn position(&self) -> Position {
        self.position
    }

    #[must_use]
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}
