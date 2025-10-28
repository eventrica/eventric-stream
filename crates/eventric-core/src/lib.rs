#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_safety_doc)]
#![allow(missing_docs)]
#![doc = include_utils::include_md!("README.md:overview")]

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
            SequencedEventArc,
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
