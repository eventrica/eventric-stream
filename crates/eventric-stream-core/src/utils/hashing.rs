//! The [`hashing`][hashing] module contains a simple pre-configured hash
//! function for anything which implements `AsRef<[u8]>`, and is used by event
//! components which are stored as hashed references (identifiers, tags, and
//! potentially other values depending on implementation).
//!
//! [hashing]: self

use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};

use rapidhash::v3::{
    self,
    RapidSecrets,
};

// =================================================================================================
// Hashing
// =================================================================================================

// Configuration

static SEED: RapidSecrets = RapidSecrets::seed(0x2811_2017);

// -------------------------------------------------------------------------------------------------

// Hash

/// Compute a 64 bit hash of the target value using the rapidhash v3 algorithm,
/// which should be stable/portable.
pub fn hash<T>(target: &T) -> u64
where
    T: AsRef<[u8]>,
{
    v3::rapidhash_v3_seeded(target.as_ref(), &SEED)
}

pub fn get<T>(target: &T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();

    Hash::hash(target, &mut hasher);
    Hasher::finish(&hasher)
}
