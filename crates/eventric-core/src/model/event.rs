pub mod data;
pub mod identifier;
pub mod tag;
pub mod timestamp;
pub mod version;

use std::sync::Arc;

use fancy_constructor::new;

use crate::model::{
    event::{
        data::Data,
        identifier::{
            Identifier,
            IdentifierHash,
            IdentifierHashRef,
        },
        tag::{
            Tag,
            TagHash,
            TagHashRef,
        },
        timestamp::Timestamp,
        version::Version,
    },
    stream::position::Position,
};

// =================================================================================================
// Event
// =================================================================================================

// Event

#[derive(Debug)]
pub struct Event {
    data: Data,
    identifier: Identifier,
    tags: Vec<Tag>,
    version: Version,
}

impl Event {
    pub const fn new(data: Data, identifier: Identifier, tags: Vec<Tag>, version: Version) -> Self {
        Self {
            data,
            identifier,
            tags,
            version,
        }
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
    pub(crate) data: &'a Data,
    pub(crate) identifier: IdentifierHashRef<'a>,
    pub(crate) tags: Vec<TagHashRef<'a>>,
    pub(crate) version: Version,
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
    identifier: Arc<Identifier>,
    position: Position,
    tags: Vec<Arc<Tag>>,
    timestamp: Timestamp,
    version: Version,
}

impl SequencedEvent {
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
    pub(crate) data: Data,
    pub(crate) identifier: IdentifierHash,
    pub(crate) position: Position,
    pub(crate) tags: Vec<TagHash>,
    pub(crate) timestamp: Timestamp,
    pub(crate) version: Version,
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
