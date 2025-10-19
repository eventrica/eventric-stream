#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod data;
mod descriptor;
mod event;
mod identifier;
mod position;
mod query;
mod specifier;
mod tag;
mod timestamp;
mod version;

use rapidhash::v3::RapidSecrets;

// =================================================================================================
// Eventric Core Model
// =================================================================================================

// Configuration

static SEED: RapidSecrets = RapidSecrets::seed(0x2811_2017);

// -------------------------------------------------------------------------------------------------

// Re-Exports

pub use self::{
    data::Data,
    descriptor::{
        Descriptor,
        DescriptorHash,
        DescriptorHashRef,
        DescriptorRef,
    },
    event::{
        Event,
        EventHashRef,
        SequencedEventHash,
        SequencedEventRef,
    },
    identifier::{
        Identifier,
        IdentifierHash,
        IdentifierHashRef,
    },
    position::Position,
    query::{
        Condition,
        ConditionBuilder,
        Query,
        QueryHash,
        QueryItem,
        QueryItemHash,
    },
    specifier::{
        Specifier,
        SpecifierHash,
    },
    tag::{
        Tag,
        TagHash,
        TagHashRef,
        TagRef,
    },
    timestamp::Timestamp,
    version::Version,
};
