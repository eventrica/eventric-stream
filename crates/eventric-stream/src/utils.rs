pub(crate) mod hashing;
pub(crate) mod iteration;
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
