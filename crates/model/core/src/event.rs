//! See the `eventric-surface` crate for full documentation, including
//! module-level documentation.

use std::collections::BTreeSet;

use error_stack::{
    Report,
    ResultExt as _,
};
use eventric_stream::{
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
    stream::TypeSelector,
};
use fancy_constructor::new;
use revision::{
    DeserializeRevisioned,
    SerializeRevisioned,
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
        Name::new(Self::identifier())
            .change_context(Error)
            .map(Into::into)
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
        TypeSelector::new(T::identifier()).change_context(Error)
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
        let data = Data::new(data).change_context(Error)?;

        let name = Name::new(E::identifier()).change_context(Error)?;
        let ty = Type::new(name, Version::default());
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
