#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
// #![warn(missing_docs)]

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
    error::Error,
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
