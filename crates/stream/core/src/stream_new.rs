mod storage;

use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use derive_more::{
    Debug,
    with_trait::{
        Add,
        AddAssign,
        Sub,
        SubAssign,
    },
};
use fancy_constructor::new;
use fjall::Database;

use crate::{
    error::Error,
    stream_new::storage::Storage,
};

// =================================================================================================
// Stream
// =================================================================================================

// Facets

#[derive(new, Debug)]
#[new(const_fn, vis(pub(crate)))]
pub struct Facets(
    #[new(name(position))] pub(crate) Position,
    #[new(name(timestamp))] pub(crate) Timestamp,
);

// -------------------------------------------------------------------------------------------------

// Stream

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct Stream {
    #[debug("Database")]
    database: Database,
    storage: Storage,
}

// -------------------------------------------------------------------------------------------------

// Position

#[rustfmt::skip]
#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[derive(Add, AddAssign, Sub, SubAssign)]
#[new(const_fn)]
pub struct Position(#[new(name(position))] pub(crate) u64);

impl Position {
    pub const MAX: Self = Self::new(u64::MAX);
    pub const MIN: Self = Self::new(u64::MIN);
}

impl Add<u64> for Position {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<u64> for Position {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::MIN
    }
}

impl Sub<u64> for Position {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<u64> for Position {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 -= rhs;
    }
}

// -------------------------------------------------------------------------------------------------

// Timestamp

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(#[new(name(nanos))] pub(crate) u64);

impl Timestamp {
    pub fn now() -> Result<Self, Error> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| Error::general(format!("Timestamp/Now/Duration: {err}")))?;

        let nanos = u64::try_from(duration.as_nanos())
            .map_err(|err| Error::general(format!("Timestamp/Now/Duration Size: {err}")))?;

        Ok(Self::new(nanos))
    }
}
