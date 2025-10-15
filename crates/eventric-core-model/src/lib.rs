#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod event;
mod query;
mod stream;

// =================================================================================================
// Eventric Core Model
// =================================================================================================

pub use self::{
    event::{
        Data,
        Descriptor,
        DescriptorHash,
        DescriptorHashRef,
        Identifier,
        IdentifierHash,
        IdentifierHashRef,
        Tag,
        TagHash,
        TagHashRef,
        Version,
        insertion::{
            Event,
            EventHashRef,
        },
        retrieval::EventHash,
    },
    query::{
        Query,
        QueryHash,
        QueryItem,
        QueryItemHash,
        Specifier,
        SpecifierHash,
    },
    stream::Position,
};
