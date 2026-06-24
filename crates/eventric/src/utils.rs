//! Crate utilities: a stable `hashing` function and a small `validation`
//! framework (both internal), plus [`temp_path`] for creating temporary
//! stream-storage paths.

pub(crate) mod hashing;
pub(crate) mod validation;

use std::path::{
    Path,
    PathBuf,
};

// =================================================================================================
// Utilities
// =================================================================================================

// Temporary Path

#[doc(hidden)]
#[must_use]
pub fn temp_path() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let random: u64 = rand::random();

    Path::new(&temp_dir).join(random.to_string())
}
