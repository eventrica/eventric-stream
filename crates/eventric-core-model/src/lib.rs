#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod event;
mod query;

// =================================================================================================
// Eventric Core Model
// =================================================================================================

pub use self::{
    event::{
        Event,
        EventHashRef,
        SequencedEventHash,
        SequencedEventRef,
        data::Data,
        descriptor::{
            Descriptor,
            DescriptorHash,
            DescriptorHashRef,
            DescriptorRef,
        },
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
            TagRef,
        },
        timestamp::Timestamp,
        version::Version,
    },
    query::{
        Query,
        QueryHash,
        QueryItem,
        QueryItemHash,
        specifier::{
            Specifier,
            SpecifierHash,
        },
    },
};
