#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod stream;

// =================================================================================================
// Eventric Core
// =================================================================================================

// Re-Exports

pub use eventric_core_model::{
    Data,
    Descriptor,
    Identifier,
    Position,
    Tag,
    Version,
};

pub use self::stream::Stream;

pub mod append {
    pub use eventric_core_model::Event;
}

pub mod query {
    pub use eventric_core_model::{
        Query,
        QueryItem,
        Specifier,
    };
}
