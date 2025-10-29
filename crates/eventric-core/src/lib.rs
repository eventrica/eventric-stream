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

pub use crate::utils::temp_path;
