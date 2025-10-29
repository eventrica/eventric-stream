//! The [`event`][self] module contains the constituent components for events,
//! both pre- and post- stream append, as well as types related to specifying
//! events within queries.

pub(crate) mod data;
pub(crate) mod identifier;
pub(crate) mod position;
pub(crate) mod specifier;
pub(crate) mod tag;
pub(crate) mod timestamp;
pub(crate) mod version;

use fancy_constructor::new;

use crate::event::{
    identifier::{
        IdentifierHash,
        IdentifierHashRef,
    },
    tag::{
        TagHash,
        TagHashRef,
    },
};

// =================================================================================================
// Event
// =================================================================================================

// Ephemeral

#[derive(new, Debug)]
#[new(const_fn, name(new_inner), vis())]
pub struct EphemeralEvent {
    data: Data,
    identifier: Identifier,
    tags: Vec<Tag>,
    version: Version,
}

impl EphemeralEvent {
    #[must_use]
    pub const fn new(data: Data, identifier: Identifier, tags: Vec<Tag>, version: Version) -> Self {
        Self::new_inner(data, identifier, tags, version)
    }
}

impl EphemeralEvent {
    #[must_use]
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[must_use]
    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        &self.version
    }
}

// Hash Ref

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct EphemeralEventHashRef<'a> {
    pub data: &'a Data,
    pub identifier: IdentifierHashRef<'a>,
    pub tags: Vec<TagHashRef<'a>>,
    pub version: Version,
}

impl<'a> From<&'a EphemeralEvent> for EphemeralEventHashRef<'a> {
    fn from(event: &'a EphemeralEvent) -> Self {
        Self::new(
            event.data(),
            event.identifier().into(),
            event.tags().iter().map(Into::into).collect(),
            *event.version(),
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Persistent

#[derive(new, Debug)]
#[new(const_fn)]
pub struct PersistentEvent {
    data: Data,
    identifier: Identifier,
    position: Position,
    tags: Vec<Tag>,
    timestamp: Timestamp,
    version: Version,
}

impl PersistentEvent {
    #[must_use]
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[must_use]
    pub fn position(&self) -> &Position {
        &self.position
    }

    #[must_use]
    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }

    #[must_use]
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        &self.version
    }
}

// Hash

#[derive(new, Debug)]
#[new(const_fn)]
pub(crate) struct PersistentEventHash {
    pub data: Data,
    pub identifier: IdentifierHash,
    pub position: Position,
    pub tags: Vec<TagHash>,
    pub timestamp: Timestamp,
    pub version: Version,
}

impl PersistentEventHash {
    #[must_use]
    #[rustfmt::skip]
    pub fn take(self) -> (Data, IdentifierHash, Position, Vec<TagHash>, Timestamp, Version) {
        (
            self.data,
            self.identifier,
            self.position,
            self.tags,
            self.timestamp,
            self.version,
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::{
    data::Data,
    identifier::Identifier,
    position::Position,
    specifier::Specifier,
    tag::Tag,
    timestamp::Timestamp,
    version::Version,
};
