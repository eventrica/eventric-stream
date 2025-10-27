pub mod iter;

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
// Utilities
// =================================================================================================

// Temp Path

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
