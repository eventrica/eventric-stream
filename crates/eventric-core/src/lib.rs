#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_safety_doc)]
#![allow(missing_docs)]
#![doc = include_utils::include_md!("README.md:overview")]

pub mod error;
pub mod event;
pub mod stream;

mod util;

// =================================================================================================
// Eventric Core
// =================================================================================================

// Re-Exports

pub use crate::util::temp_path;
