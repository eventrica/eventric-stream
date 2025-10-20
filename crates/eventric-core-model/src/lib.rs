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
        DescriptorArc,
        DescriptorHash,
        DescriptorHashRef,
    },
    event::{
        Event,
        EventHashRef,
        SequencedEventArc,
        SequencedEventHash,
    },
    identifier::{
        Identifier,
        IdentifierHash,
        IdentifierHashRef,
    },
    position::Position,
    query::{
        Query,
        QueryHash,
        QueryHashRef,
        QueryItem,
        QueryItemHash,
        QueryItemHashRef,
    },
    specifier::{
        Specifier,
        SpecifierHash,
        SpecifierHashRef,
    },
    tag::{
        Tag,
        TagArc,
        TagHash,
        TagHashRef,
    },
    timestamp::Timestamp,
    version::Version,
};
