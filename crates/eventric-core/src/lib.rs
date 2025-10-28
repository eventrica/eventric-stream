#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_safety_doc)]
#![allow(missing_docs)]
#![doc = include_utils::include_md!("README.md:overview")]

// =================================================================================================
// Eventric Core
// =================================================================================================

// Re-Exports

pub mod error {
    pub use eventric_core_error::Error;
}

pub mod event {
    pub use eventric_core_event::{
        Event,
        SequencedEvent,
        SequencedEventArc,
        data::Data,
        identifier::Identifier,
        position::Position,
        specifier::Specifier,
        tag::Tag,
        timestamp::Timestamp,
        version::Version,
    };
}

pub mod stream {
    pub mod append {
        pub use eventric_core_stream::append::condition::Condition;
    }

    pub mod query {
        pub use eventric_core_stream::query::{
            Query,
            QueryItem,
            Specifiers,
            Tags,
            cache::Cache,
            condition::Condition,
            options::Options,
        };
    }

    pub use eventric_core_stream::{
        Builder,
        Stream,
    };
}

pub use eventric_core_utils::temp_path;
