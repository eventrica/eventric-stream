use std::sync::Arc;

use fancy_constructor::new;
use itertools::Itertools;

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
#[new(const_fn)]
pub struct Event {
    data: Data,
    identifier: Identifier,
    tags: Vec<Tag>,
    version: Version,
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
    data: &'a Data,
    identifier: IdentifierHashRef<'a>,
    tags: Vec<TagHashRef<'a>>,
    version: Version,
}

impl EventHashRef<'_> {
    #[must_use]
    pub fn data(&self) -> &Data {
        self.data
    }

    #[must_use]
    pub fn identifier(&self) -> &IdentifierHashRef<'_> {
        &self.identifier
    }

    #[must_use]
    pub fn tags(&self) -> &Vec<TagHashRef<'_>> {
        &self.tags
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        &self.version
    }
}

impl<'a> From<&'a Event> for EventHashRef<'a> {
    fn from(event: &'a Event) -> Self {
        Self::new(
            event.data(),
            event.identifier().into(),
            event.tags().iter().map_into().collect_vec(),
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
    data: Data,
    identifier: IdentifierHash,
    position: Position,
    tags: Vec<TagHash>,
    timestamp: Timestamp,
    version: Version,
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
