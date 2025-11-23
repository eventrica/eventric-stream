use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use derive_more::{
    Debug,
    Deref,
};

use crate::error::Error;

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
    /// Constructs a new [`Timestamp`] instance with the underlying value
    /// derived from the current system time.
    ///
    /// NOTE: This uses [`SystemTime`] which is not guaranteed to be monotonic.
    /// In practice it's unlikely to be an issue, but this is
    /// worth monitoring/noting when using the generated timestamp values as
    /// part of event sequencing - there is a non-zero chance that timestamp
    /// order may not match positional order (although this isn't likely). Do
    /// not rely on sorting by timestamp being identical to sorting by position
    /// when working with events.
    ///
    /// # Errors
    ///
    /// Returns an error if the current system time is too late to be stored in
    /// a `u64` number of nanoseconds. This will become an issue in C22.
    pub fn now() -> Result<Self, Error> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| Error::data(format!("duration error: {err}")))?;

        let nanos = u64::try_from(duration.as_nanos())
            .map_err(|_| Error::data("duration size error: {err}"))?;

        Ok(Self(nanos))
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use crate::event::timestamp::Timestamp;

    // Timestamp::new

    #[test]
    fn new_creates_timestamp_with_given_value() {
        let timestamp = Timestamp::new(123_456_789);

        assert_eq!(123_456_789, *timestamp);
    }

    #[test]
    fn new_creates_timestamp_with_zero() {
        let timestamp = Timestamp::new(0);

        assert_eq!(0, *timestamp);
    }

    #[test]
    fn new_creates_timestamp_with_max_value() {
        let timestamp = Timestamp::new(u64::MAX);

        assert_eq!(u64::MAX, *timestamp);
    }

    // Timestamp::now

    #[test]
    fn now_creates_timestamp_from_system_time() {
        let result = Timestamp::now();

        assert!(result.is_ok());
    }

    // Deref

    #[test]
    fn deref_returns_inner_value() {
        let timestamp = Timestamp::new(123_456_789);

        let value: u64 = *timestamp;

        assert_eq!(123_456_789, value);
    }

    // Clone

    #[allow(clippy::clone_on_copy)]
    #[test]
    fn clone_creates_identical_copy() {
        let timestamp = Timestamp::new(123_456_789);

        let cloned = timestamp.clone();

        assert_eq!(timestamp, cloned);
        assert_eq!(*timestamp, *cloned);
    }

    // Copy

    #[test]
    fn copy_creates_identical_copy() {
        let timestamp = Timestamp::new(123_456_789);

        let copied = timestamp;

        assert_eq!(timestamp, copied);
        assert_eq!(*timestamp, *copied);
    }

    // PartialEq / Eq

    #[test]
    fn equal_timestamps_compare_as_equal() {
        let timestamp1 = Timestamp::new(123_456_789);
        let timestamp2 = Timestamp::new(123_456_789);

        assert_eq!(timestamp1, timestamp2);
    }

    #[test]
    fn different_timestamps_compare_as_not_equal() {
        let timestamp1 = Timestamp::new(123_456_789);
        let timestamp2 = Timestamp::new(987_654_321);

        assert_ne!(timestamp1, timestamp2);
    }

    // PartialOrd / Ord

    #[test]
    fn smaller_timestamp_is_less_than_larger() {
        let timestamp1 = Timestamp::new(100);
        let timestamp2 = Timestamp::new(200);

        assert!(timestamp1 < timestamp2);
        assert!(timestamp2 > timestamp1);
    }

    #[test]
    fn equal_timestamps_compare_as_equal_with_ordering() {
        let timestamp1 = Timestamp::new(123_456_789);
        let timestamp2 = Timestamp::new(123_456_789);

        assert!(timestamp1 <= timestamp2);
        assert!(timestamp1 >= timestamp2);
    }

    #[test]
    fn timestamps_can_be_sorted() {
        let mut timestamps = [
            Timestamp::new(300),
            Timestamp::new(100),
            Timestamp::new(200),
        ];

        timestamps.sort();

        assert_eq!(Timestamp::new(100), timestamps[0]);
        assert_eq!(Timestamp::new(200), timestamps[1]);
        assert_eq!(Timestamp::new(300), timestamps[2]);
    }
}
