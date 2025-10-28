#![allow(clippy::multiple_crate_versions)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_safety_doc)]
#![allow(missing_docs)]
#![deny(unsafe_code)]
#![doc = include_utils::include_md!("../NOTICE.md")]

pub mod data;
pub mod identifier;
pub mod position;
pub mod specifier;
pub mod tag;
pub mod timestamp;
pub mod version;

use fancy_constructor::new;

use crate::{
    data::Data,
    identifier::{
        Identifier,
        IdentifierHash,
        IdentifierHashRef,
    },
    position::Position,
    tag::{
        Tag,
        TagHash,
        TagHashRef,
    },
    timestamp::Timestamp,
    version::Version,
};

// =================================================================================================
// Event
// =================================================================================================

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
pub struct PersistentEventHash {
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
pub struct EphemeralEventHashRef<'a> {
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
