use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use derive_more::Debug;
use fancy_constructor::new;

// =================================================================================================
// Timestamp
// =================================================================================================

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(const_fn)]
pub struct Timestamp(u64);

impl Timestamp {
    #[must_use]
    pub fn nanos(self) -> u64 {
        self.0
    }

    /// NOTE: Important - this uses [`SystemTime`] which is not guaranteed to be
    /// monotonic. In practice it's unlikely to be an issue, but this is
    /// worth monitoring/noting when using the generated timestamp values as
    /// part of event sequencing - there is a non-zero chance that timestamp
    /// order may not match positional order (although this isn't likely).
    ///
    /// This should probably be replaced at some point with something like TAI
    /// time.
    #[must_use]
    pub fn now() -> Self {
        let ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("unix time error")
            .as_nanos();

        let ns = u64::try_from(ns).expect("unix time excession error");

        Self(ns)
    }
}
