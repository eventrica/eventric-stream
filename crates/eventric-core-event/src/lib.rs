#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_safety_doc)]
#![allow(missing_docs)]
#![doc = include_utils::include_md!("../NOTICE.md")]

pub mod data;
pub mod identifier;
pub mod position;
pub mod specifier;
pub mod tag;
pub mod timestamp;
pub mod version;

use std::sync::Arc;

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

// Event

#[derive(new, Debug)]
#[new(const_fn, name(new_inner), vis())]
pub struct Event {
    data: Data,
    identifier: Identifier,
    tags: Vec<Tag>,
    version: Version,
}

impl Event {
    #[must_use]
    pub const fn new(data: Data, identifier: Identifier, tags: Vec<Tag>, version: Version) -> Self {
        Self::new_inner(data, identifier, tags, version)
    }
}

impl Event {
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
pub struct EventHashRef<'a> {
    pub data: &'a Data,
    pub identifier: IdentifierHashRef<'a>,
    pub tags: Vec<TagHashRef<'a>>,
    pub version: Version,
}

impl<'a> From<&'a Event> for EventHashRef<'a> {
    fn from(event: &'a Event) -> Self {
        Self::new(
            event.data(),
            event.identifier().into(),
            event.tags().iter().map(Into::into).collect(),
            *event.version(),
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Sequenced Event

#[derive(new, Debug)]
#[new(const_fn)]
pub struct SequencedEvent {
    data: Data,
    identifier: Identifier,
    position: Position,
    tags: Vec<Tag>,
    timestamp: Timestamp,
    version: Version,
}

impl SequencedEvent {
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

impl From<SequencedEventArc> for SequencedEvent {
    fn from(value: SequencedEventArc) -> Self {
        Self {
            data: value.data,
            identifier: Arc::unwrap_or_clone(value.identifier),
            position: value.position,
            tags: value.tags.into_iter().map(Arc::unwrap_or_clone).collect(),
            timestamp: value.timestamp,
            version: value.version,
        }
    }
}

// Arc

#[derive(new, Debug)]
#[new(const_fn)]
pub struct SequencedEventArc {
    data: Data,
    identifier: Arc<Identifier>,
    position: Position,
    tags: Vec<Arc<Tag>>,
    timestamp: Timestamp,
    version: Version,
}

impl SequencedEventArc {
    #[must_use]
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[must_use]
    pub fn identifier(&self) -> &Arc<Identifier> {
        &self.identifier
    }

    #[must_use]
    pub fn position(&self) -> &Position {
        &self.position
    }

    #[must_use]
    pub fn tags(&self) -> &Vec<Arc<Tag>> {
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
pub struct SequencedEventHash {
    pub data: Data,
    pub identifier: IdentifierHash,
    pub position: Position,
    pub tags: Vec<TagHash>,
    pub timestamp: Timestamp,
    pub version: Version,
}

impl SequencedEventHash {
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
