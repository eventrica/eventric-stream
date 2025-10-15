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
        Descriptor,
        Identifier,
        Tag,
        Version,
        insertion::Event,
    },
    query::{
        Query,
        QueryItem,
        Specifier,
    },
    stream::Position,
};
