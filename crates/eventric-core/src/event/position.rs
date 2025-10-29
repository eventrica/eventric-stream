use derive_more::{
    Deref,
    with_trait::{
        Add,
        AddAssign,
        Sub,
        SubAssign,
    },
};

// =================================================================================================
// Position
// =================================================================================================

/// The [`Position`] type is a typed wrapper around a `u64` value representing
/// the ordinal position of an event in a stream.
#[rustfmt::skip]
#[derive(Add, AddAssign, Clone, Copy, Debug, Deref, Eq, Ord, PartialEq, PartialOrd, Sub, SubAssign)]
pub struct Position(u64);

impl Position {
    /// Constructs a new [`Position`] instance given a `u64` position value.
    #[must_use]
    pub const fn new(position: u64) -> Self {
        Self(position)
    }
}

impl Position {
    /// Represents the maximum possible value of a [`Position`] (which is
    /// effectively `u64::MAX` internally).
    pub const MAX: Self = Self::new(u64::MAX);
    /// Represents the minimum possible value of a [`Position`] (which is
    /// `u64::MIN` - zero - the first event in a stream).
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
