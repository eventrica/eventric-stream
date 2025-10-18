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
    pub fn nanos(&self) -> u64 {
        self.0
    }

    #[must_use]
    pub fn now() -> Self {
        let ns = jiff::Timestamp::now().as_nanosecond();
        let ts = u64::try_from(ns).expect("invalid timestamp");

        Self(ts)
    }
}
