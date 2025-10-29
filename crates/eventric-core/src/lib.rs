#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_safety_doc)]
#![allow(missing_docs)]
#![deny(unsafe_code)]
#![doc = include_utils::include_md!("README.md:overview")]

pub mod error;
pub mod event;
pub mod stream;

pub(crate) mod utils;

// =================================================================================================
// Eventric Core
// =================================================================================================

// Re-Exports

// /// The [`error`] module contains the common [`Error`][error] type used
// /// throughout `eventric-core`.
// ///
// /// [error]: crate::error::Error
// pub mod error {
//     pub use eventric_core_error::Error;
// }

// /// The [`event`] module contains the constituent components for events, both
// /// pre- and post- stream append, as well as types related to specifying
// events /// within queries.
// pub mod event {
//     pub use eventric_core_event::{
//         EphemeralEvent,
//         PersistentEvent,
//         data::Data,
//         identifier::Identifier,
//         position::Position,
//         specifier::Specifier,
//         tag::Tag,
//         timestamp::Timestamp,
//         version::Version,
//     };
// }

// /// The [`stream`] module contains the core [`Stream`][stream] abstraction,
// /// along with support for configuring and opening stream instances.
// Sub-modules /// contain types related to appending events to the stream, and
// querying the /// stream for previously appended events.
// ///
// /// [stream]: crate::stream::Stream
// pub mod stream {
//     /// The [`append`] module contains types related to append operations on
// the     /// core [`Stream`]. Note that append operations (when conditional)
// rely on     /// queries, and thus will also likely require types from the
// [`query`]     /// module.
//     pub mod append {
//         pub use eventric_core_stream::append::condition::Condition;
//     }

//     /// The [`query`] module contains types related to query operations on
// the     /// core [`Stream`].
//     pub mod query {
//         pub use eventric_core_stream::query::{
//             Query,
//             QueryItem,
//             Specifiers,
//             Tags,
//             cache::Cache,
//             condition::Condition,
//             options::Options,
//         };
//     }

//     pub use eventric_core_stream::{
//         Builder,
//         Stream,
//     };
// }

// pub use eventric_core_utils::temp_path;
pub use crate::utils::temp_path;
