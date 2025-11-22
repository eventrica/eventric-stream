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

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use crate::event::version::Version;

    #[test]
    fn new_creates_version_with_given_value() {
        let ver = Version::new(42);

        assert_eq!(*ver, 42);
    }

    #[test]
    fn min_constant_equals_zero() {
        assert_eq!(*Version::MIN, 0);
        assert_eq!(*Version::MIN, u8::MIN);
    }

    #[test]
    fn max_constant_equals_u8_max() {
        assert_eq!(*Version::MAX, u8::MAX);
        assert_eq!(*Version::MAX, 255);
    }

    #[test]
    fn default_equals_min() {
        assert_eq!(Version::default(), Version::MIN);
        assert_eq!(*Version::default(), 0);
    }

    #[test]
    fn deref_returns_inner_value() {
        let ver = Version::new(123);
        let value: &u8 = &ver;

        assert_eq!(*value, 123);
    }

    #[allow(clippy::clone_on_copy)]
    #[test]
    fn clone_creates_equal_copy() {
        let ver1 = Version::new(100);
        let ver2 = ver1.clone();

        assert_eq!(ver1, ver2);
        assert_eq!(*ver1, *ver2);
    }

    #[test]
    fn copy_creates_equal_copy() {
        let ver1 = Version::new(100);
        let ver2 = ver1; // Copy trait

        assert_eq!(ver1, ver2);
    }

    #[test]
    fn add_u8_to_version() {
        let ver = Version::new(10);
        let result = ver + 20u8;

        assert_eq!(*result, 30);
    }

    #[test]
    fn add_assign_u8_to_version() {
        let mut ver = Version::new(10);
        ver += 20u8;

        assert_eq!(*ver, 30);
    }

    #[test]
    fn sub_u8_from_version() {
        let ver = Version::new(50);
        let result = ver - 20u8;

        assert_eq!(*result, 30);
    }

    #[test]
    fn sub_assign_u8_from_version() {
        let mut ver = Version::new(50);
        ver -= 20u8;

        assert_eq!(*ver, 30);
    }

    #[test]
    fn equality_comparison() {
        let ver1 = Version::new(42);
        let ver2 = Version::new(42);
        let ver3 = Version::new(43);

        assert_eq!(ver1, ver2);
        assert_ne!(ver1, ver3);
    }

    #[test]
    fn ordering_comparison() {
        let ver1 = Version::new(10);
        let ver2 = Version::new(20);
        let ver3 = Version::new(30);

        assert!(ver1 < ver2);
        assert!(ver2 > ver1);
        assert!(ver1 <= ver2);
        assert!(ver2 >= ver1);
        assert!(ver1 <= ver1);
        assert!(ver1 >= ver1);

        assert!(ver1 < ver3);
        assert!(ver2 < ver3);
    }

    #[test]
    fn debug_format() {
        let ver = Version::new(42);
        let debug_str = format!("{ver:?}");

        assert!(debug_str.contains("Version"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn arithmetic_chain() {
        let ver = Version::new(100);
        let result = ver + 50u8 - 30u8;

        assert_eq!(*result, 120);
    }

    #[test]
    fn min_plus_one() {
        let ver = Version::MIN + 1u8;

        assert_eq!(*ver, 1);
    }

    #[test]
    fn max_minus_one() {
        let ver = Version::MAX - 1u8;

        assert_eq!(*ver, 254);
    }

    #[test]
    fn version_zero_is_valid() {
        let ver = Version::new(0);

        assert_eq!(*ver, 0);
        assert_eq!(ver, Version::MIN);
    }

    #[test]
    fn version_max_is_valid() {
        let ver = Version::new(255);

        assert_eq!(*ver, 255);
        assert_eq!(ver, Version::MAX);
    }

    #[test]
    fn increment_version() {
        let mut ver = Version::new(1);
        ver += 1u8;

        assert_eq!(*ver, 2);

        ver += 1u8;

        assert_eq!(*ver, 3);
    }

    #[test]
    fn decrement_version() {
        let mut ver = Version::new(10);
        ver -= 1u8;

        assert_eq!(*ver, 9);

        ver -= 1u8;

        assert_eq!(*ver, 8);
    }
}
