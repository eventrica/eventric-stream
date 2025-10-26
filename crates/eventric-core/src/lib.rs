#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod data;
mod error;
mod model;
mod stream;
mod util;

// =================================================================================================
// Eventric Core
// =================================================================================================

// Re-Exports

pub use self::{
    error::{
        Error,
        Result,
    },
    model::{
        event::{
            Event,
            SequencedEvent,
            data::Data,
            identifier::Identifier,
            tag::Tag,
            timestamp::Timestamp,
            version::Version,
        },
        query::{
            Query,
            QueryItem,
            specifier::Specifier,
        },
        stream::position::Position,
    },
    stream::{
        Stream,
        StreamBuilder,
        append::AppendCondition,
        query::{
            QueryCache,
            QueryCondition,
            QueryIterator,
            QueryOptions,
        },
    },
};
