pub(crate) mod hashing;
pub(crate) mod iteration;
pub(crate) mod validation;

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
