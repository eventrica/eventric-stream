#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod append;
mod common;
mod query;

// =================================================================================================
// Eventric Core Model
// =================================================================================================

pub use self::{
    append::{
        DescriptorHashRef,
        Event,
        EventHashRef,
        IdentifierHashRef,
        TagHashRef,
    },
    common::{
        Data,
        Descriptor,
        Identifier,
        Position,
        Tag,
        Version,
    },
    query::{
        DescriptorHash,
        IdentifierHash,
        Query,
        QueryHash,
        QueryItem,
        QueryItemHash,
        SequencedEvent,
        SequencedEventHash,
        Specifier,
        SpecifierHash,
        TagHash,
    },
};
