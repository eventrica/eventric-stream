use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use derive_more::{
    Debug,
    Deref,
};
use fancy_constructor::new;

use crate::error::Error;

// =================================================================================================
// Timestamp
// =================================================================================================

#[derive(new, Clone, Copy, Debug, Deref, Eq, Ord, PartialEq, PartialOrd)]
#[new(args(nanos: u64), const_fn)]
pub struct Timestamp(#[new(val(nanos))] u64);

impl Timestamp {
    /// NOTE: Important - this uses [`SystemTime`] which is not guaranteed to be
    /// monotonic. In practice it's unlikely to be an issue, but this is
    /// worth monitoring/noting when using the generated timestamp values as
    /// part of event sequencing - there is a non-zero chance that timestamp
    /// order may not match positional order (although this isn't likely).
    ///
    /// This should probably be replaced at some point with something like TAI
    /// time.
    pub(crate) fn now() -> Result<Self, Error> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| Error::internal(format!("duration error: {err}")))?;

        let nanos = u64::try_from(duration.as_nanos())
            .map_err(|_| Error::internal("duration size error: {err}"))?;

        Ok(Self(nanos))
    }
}
