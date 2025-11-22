use std::ops::{
    Add,
    AddAssign,
    Sub,
    SubAssign,
};

use derive_more::Deref;

// =================================================================================================
// Version
// =================================================================================================

/// The [`Version`] type is a typed wrapper around a `u8` version value, which
/// should be used as a monotonic indicator of the *type version* of the event.
/// When paired with the [`Identifier`][ident] value, the pair forms a
/// specification of the logical versioned *type* of the event.
///
/// [ident]: crate::event::identifier::Identifier
#[derive(Clone, Copy, Debug, Deref, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version(u8);

impl Version {
    /// Constructs a new instance of [`Version`] from a given `u8` version
    /// value.
    #[must_use]
    pub const fn new(version: u8) -> Self {
        Self(version)
    }
}

impl Version {
    /// Represents the maximum possible value of a [`Version`] (which is
    /// effectively `u8::MAX` internally).
    pub const MAX: Self = Self::new(u8::MAX);
    /// Represents the minimum possible value of a [`Version`] (which is
    /// `u8::MIN` internally).
    pub const MIN: Self = Self::new(u8::MIN);
}

impl Add<u8> for Version {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<u8> for Version {
    fn add_assign(&mut self, rhs: u8) {
        self.0 += rhs;
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::MIN
    }
}

impl Sub<u8> for Version {
    type Output = Self;

    fn sub(self, rhs: u8) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<u8> for Version {
    fn sub_assign(&mut self, rhs: u8) {
        self.0 -= rhs;
    }
}
