#![allow(clippy::multiple_crate_versions)]
#![doc = include_utils::include_md!("README.md:overview")]
#![warn(missing_docs)]

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
            tag::{
                Tag,
                Tags,
            },
            timestamp::Timestamp,
            version::Version,
        },
        query::{
            Query,
            QueryItem,
            specifier::{
                Specifier,
                Specifiers,
            },
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
    util::temp_path,
};
