#![allow(clippy::multiple_crate_versions)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_safety_doc)]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![doc = include_utils::include_md!("../NOTICE.md")]

pub mod hashing;
pub mod iteration;
pub mod validation;

use std::{
    path::{
        Path,
        PathBuf,
    },
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

// =================================================================================================
// Eventric Core Utilities
// =================================================================================================

// Temporary Path

#[doc(hidden)]
#[must_use]
pub fn temp_path() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    Path::new(&temp_dir).join(nanos.to_string())
}
