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

/// Compute a 64 bit hash of the target value using the seeded rapidhash v3
/// algorithm.
///
/// This is the **stable, on-disk hash contract**: identifiers and tags are
/// persisted (and queried) purely as this hash, so its output must remain
/// stable across Rust versions and platforms forever. rapidhash v3 with a fixed
/// [`SEED`] is portable and deterministic, satisfying that requirement. Any new
/// code that persists a hash MUST use this function, never [`get`].
pub fn hash<T>(target: &T) -> u64
where
    T: AsRef<[u8]>,
{
    v3::rapidhash_v3_seeded(target.as_ref(), &SEED)
}

/// Compute a 64 bit hash of the target via the standard library
/// [`DefaultHasher`].
///
/// The output is **not** guaranteed stable across Rust versions or platforms,
/// so it must never be used for a value that is persisted or otherwise treated
/// as a stable identity — use [`hash`] for that. This remains only because the
/// legacy `event` module still derives its hashes through it; it should be
/// removed at cutover once that module is gone.
pub fn get<T>(target: &T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();

    Hash::hash(target, &mut hasher);
    Hasher::finish(&hasher)
}

// =================================================================================================
// Tests
// =================================================================================================

#[cfg(test)]
mod tests {
    use super::hash;

    // These literals pin the on-disk hash contract. Once the `references`
    // keyspace is removed, the hash is the ONLY persisted record of an
    // identifier/tag (it cannot be re-derived from a stored string), so a change
    // to any of these values is a silent, unrecoverable data-format break. If a
    // rapidhash upgrade legitimately changes them, that is a deliberate format
    // migration, not a value to "just update".
    #[test]
    fn hash_matches_pinned_values() {
        assert_eq!(
            hash(&"StudentSubscribedToCourse"),
            11_050_574_304_676_595_385
        );
        assert_eq!(hash(&"student:3242"), 11_829_712_411_174_272_360);
        assert_eq!(hash(&"course:523"), 6_605_660_143_518_382_207);
    }

    #[test]
    fn hash_is_deterministic() {
        assert_eq!(hash(&"student:3242"), hash(&"student:3242"));
    }

    #[test]
    fn hash_distinguishes_inputs() {
        assert_ne!(hash(&"student:1"), hash(&"student:2"));
    }
}
