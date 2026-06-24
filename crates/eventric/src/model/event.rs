//! Events: the [`Event`] trait (with its [`Identifier`]/[`Specifier`]/[`Tags`]
//! components) and the [`Events`] buffer.

use std::collections::BTreeSet;

use error_stack::{
    Report,
    ResultExt as _,
};
pub use eventric_macros::Event;
use fancy_constructor::new;
use revision::{
    DeserializeRevisioned,
    SerializeRevisioned,
};

use crate::{
    error::Error,
    event::{
        Data,
        Event as StreamEvent,
        Facets,
        Name,
        Tag,
        Type,
        Version,
    },
    stream::operate::select::TypeSelector,
};

// =================================================================================================
// Event
// =================================================================================================

// Event

pub trait Event: DeserializeRevisioned + Identifier + Tags + SerializeRevisioned {}

// Identifier

pub trait Identifier {
    /// The event type's name, as a validated literal.
    fn identifier() -> &'static str;

    /// The event type's name hashed the same way the stream indexes it, used to
    /// recognise an event by type.
    fn type_name() -> Result<Name<u64>, Report<Error>> {
        Name::new(Self::identifier()).map(Into::into)
    }
}

// Specifier

pub trait Specifier {
    fn specifier() -> Result<TypeSelector<String>, Report<Error>>;
}

impl<T> Specifier for T
where
    T: Identifier,
{
    fn specifier() -> Result<TypeSelector<String>, Report<Error>> {
        TypeSelector::new(T::identifier())
    }
}

// Tags

pub trait Tags {
    fn tags(&self) -> Result<Vec<Tag<String>>, Report<Error>>;
}

// -------------------------------------------------------------------------------------------------

// Events

#[derive(new, Debug)]
pub struct Events {
    #[new(default)]
    events: Vec<StreamEvent<(), String>>,
}

impl Events {
    pub fn append<E>(&mut self, event: &E) -> Result<(), Report<Error>>
    where
        E: Event,
    {
        let data = revision::to_vec(event)
            .change_context(Error)
            .attach("events/append/revision")?;
        let data = Data::new(data)?;

        let name = Name::new(E::identifier())?;
        // The event's stream `Version` is sourced directly from the `revision`
        // schema number, so the two cannot diverge (there is no separate version
        // to declare or forget to bump). `revision` is a `u16`; `Version` is a
        // `u8` — 256+ revisions of one event type is implausible, but it errors
        // rather than truncating silently.
        let version = u8::try_from(E::revision())
            .change_context(Error)
            .attach("events/append/version: revision exceeds u8::MAX")?;
        let ty = Type::new(name, Version::new(version));
        let tags = event.tags()?.into_iter().collect::<BTreeSet<_>>();

        self.events
            .push(StreamEvent::new(data, Facets::new(ty, tags), ()));

        Ok(())
    }

    #[must_use]
    pub fn take(self) -> Vec<StreamEvent<(), String>> {
        self.events
    }
}
