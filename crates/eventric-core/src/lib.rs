#![allow(clippy::multiple_crate_versions)]

// =================================================================================================
// Eventric Core
// =================================================================================================

// Re-Exports

pub mod event {
    pub use eventric_core_model::{
        AppendEvent,
        Data,
        Descriptor,
        Identifier,
        Tag,
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
    pub use eventric_core_model::Position;
    pub use eventric_core_stream::Stream;
}
