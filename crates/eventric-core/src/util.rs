pub mod iter;
pub mod validate;

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

use rapidhash::v3::{
    self,
    RapidSecrets,
};

// =================================================================================================
// Utilities
// =================================================================================================

// Configuration

static SEED: RapidSecrets = RapidSecrets::seed(0x2811_2017);

// -------------------------------------------------------------------------------------------------

// Hash

pub fn hash<T>(target: &T) -> u64
where
    T: AsRef<[u8]>,
{
    v3::rapidhash_v3_seeded(target.as_ref(), &SEED)
}

// -------------------------------------------------------------------------------------------------

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
