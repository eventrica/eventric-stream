pub mod data;
pub mod descriptor;
pub mod identifier;
pub mod position;
pub mod tag;
pub mod timestamp;
pub mod version;

use fancy_constructor::new;
use itertools::Itertools;
use rapidhash::v3::RapidSecrets;

use crate::event::{
    data::Data,
    descriptor::{
        Descriptor,
        DescriptorHash,
        DescriptorHashRef,
        DescriptorRef,
    },
    position::Position,
    tag::{
        Tag,
        TagHash,
        TagHashRef,
        TagRef,
    },
    timestamp::Timestamp,
};

// =================================================================================================
// Event
// =================================================================================================

// Configuration

static SEED: RapidSecrets = RapidSecrets::seed(0x2811_2017);

// -------------------------------------------------------------------------------------------------

// Event

#[derive(new, Debug)]
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
#[new(vis(pub))]
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

// Hash

#[derive(new, Debug)]
#[new(vis(pub))]
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

// Ref

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct SequencedEventRef<'a> {
    pub data: Data,
    pub descriptor: DescriptorRef<'a>,
    pub position: Position,
    pub tags: Vec<TagRef<'a>>,
    pub timestamp: Timestamp,
}

impl SequencedEventRef<'_> {
    #[must_use]
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[must_use]
    pub fn descriptor(&self) -> &DescriptorRef<'_> {
        &self.descriptor
    }

    #[must_use]
    pub fn position(&self) -> &Position {
        &self.position
    }

    #[must_use]
    pub fn tags(&self) -> &Vec<TagRef<'_>> {
        &self.tags
    }

    #[must_use]
    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}
