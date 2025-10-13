#![allow(clippy::multiple_crate_versions)]

// =================================================================================================
// Eventric Core
// =================================================================================================

// Re-Exports

pub mod event {
    pub use eventric_core_model::event::*;
}

pub mod query {
    pub use eventric_core_model::query::*;
}

pub mod stream {
    pub use eventric_core_model::stream::*;
    pub use eventric_core_stream::stream::*;
}
