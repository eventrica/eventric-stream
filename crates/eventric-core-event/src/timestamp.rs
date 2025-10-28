use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use derive_more::{
    Debug,
    Deref,
};
use eventric_core_error::Error;

// =================================================================================================
// Timestamp
// =================================================================================================

/// The [`Timestamp`] type is a typed wrapper around a `u64` nanosecond value,
/// used to represent the insertion time of an event in a stream. The value
/// represents nanoseconds since Unix Epoch, a u64 being sufficient to represent
/// comfortably over a century.
#[derive(Clone, Copy, Debug, Deref, Eq, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Constructs a new instance of [`Timestamp`] from a given `u64` nanosecond
    /// value.
    #[must_use]
    pub const fn new(nanos: u64) -> Self {
        Self(nanos)
    }
}

impl Timestamp {
    /// NOTE: Important - this uses [`SystemTime`] which is not guaranteed to be
    /// monotonic. In practice it's unlikely to be an issue, but this is
    /// worth monitoring/noting when using the generated timestamp values as
    /// part of event sequencing - there is a non-zero chance that timestamp
    /// order may not match positional order (although this isn't likely).
    ///
    /// This should probably be replaced at some point with something like TAI
    /// time.
    pub fn now() -> Result<Self, Error> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| Error::data(format!("duration error: {err}")))?;

        let nanos = u64::try_from(duration.as_nanos())
            .map_err(|_| Error::data("duration size error: {err}"))?;

        Ok(Self(nanos))
    }
}
