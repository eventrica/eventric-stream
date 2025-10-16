#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

// =================================================================================================
// Eventric Core
// =================================================================================================

// Re-Exports

pub mod event {
    pub use eventric_core_model::{
        Data,
        Descriptor,
        DescriptorRef,
        Event,
        Identifier,
        Position,
        SequencedEventRef,
        Tag,
        TagRef,
        Version,
    };
}

pub mod query {
    pub use eventric_core_model::{
        Query,
        QueryItem,
        Specifier,
    };
}

pub mod stream {
    pub use eventric_core_stream::Stream;
}
