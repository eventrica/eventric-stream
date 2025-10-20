use fancy_constructor::new;
use itertools::Itertools;

use crate::{
    data::Data,
    descriptor::{
        Descriptor,
        DescriptorArc,
        DescriptorHash,
        DescriptorHashRef,
    },
    position::Position,
    tag::{
        Tag,
        TagArc,
        TagHash,
        TagHashRef,
    },
    timestamp::Timestamp,
};

// =================================================================================================
// Event
// =================================================================================================

// Event

#[derive(new, Debug)]
#[new(const_fn)]
pub struct Event {
    data: Data,
    descriptor: Descriptor,
    tags: Vec<Tag>,
}

impl Event {
    #[must_use]
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[must_use]
    pub fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }

    #[must_use]
    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }
}

// Hash Ref

#[derive(new, Debug)]
#[new(const_fn)]
pub struct EventHashRef<'a> {
    data: &'a Data,
    descriptor: DescriptorHashRef<'a>,
    tags: Vec<TagHashRef<'a>>,
    timestamp: Timestamp,
}

impl EventHashRef<'_> {
    #[must_use]
    pub fn data(&self) -> &Data {
        self.data
    }

    #[must_use]
    pub fn descriptor(&self) -> &DescriptorHashRef<'_> {
        &self.descriptor
    }

    #[must_use]
    pub fn tags(&self) -> &Vec<TagHashRef<'_>> {
        &self.tags
    }

    #[must_use]
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl<'a> From<&'a Event> for EventHashRef<'a> {
    fn from(event: &'a Event) -> Self {
        let timestamp = Timestamp::now();

        Self::new(
            event.data(),
            event.descriptor().into(),
            event.tags().iter().map_into().collect_vec(),
            timestamp,
        )
    }
}

// -------------------------------------------------------------------------------------------------

// Sequenced Event

// Arc

#[derive(new, Debug)]
#[new(const_fn)]
pub struct SequencedEventArc {
    data: Data,
    descriptor: DescriptorArc,
    position: Position,
    tags: Vec<TagArc>,
    timestamp: Timestamp,
}

impl SequencedEventArc {
    #[must_use]
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[must_use]
    pub fn descriptor(&self) -> &DescriptorArc {
        &self.descriptor
    }

    #[must_use]
    pub fn position(&self) -> &Position {
        &self.position
    }

    #[must_use]
    pub fn tags(&self) -> &Vec<TagArc> {
        &self.tags
    }

    #[must_use]
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

// Hash

#[derive(new, Debug)]
#[new(const_fn)]
pub struct SequencedEventHash {
    data: Data,
    descriptor: DescriptorHash,
    position: Position,
    tags: Vec<TagHash>,
    timestamp: Timestamp,
}

impl SequencedEventHash {
    #[must_use]
    pub fn take(self) -> (Data, DescriptorHash, Position, Vec<TagHash>, Timestamp) {
        (
            self.data,
            self.descriptor,
            self.position,
            self.tags,
            self.timestamp,
        )
    }
}
