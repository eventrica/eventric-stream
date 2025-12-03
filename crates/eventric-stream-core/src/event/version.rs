use std::{
    cmp::Ordering,
    ops::{
        Add,
        AddAssign,
        Range,
        Sub,
        SubAssign,
    },
};

use fancy_constructor::new;

// =================================================================================================
// Version
// =================================================================================================

/// The [`Version`] type is a typed wrapper around a `u8` version value, which
/// should be used as a monotonic indicator of the *type version* of the event.
/// When paired with the [`Identifier`][ident] value, the pair forms a
/// specification of the logical versioned *type* of the event.
///
/// [ident]: crate::event::identifier::Identifier
#[derive(new, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[new(const_fn)]
pub struct Version {
    #[new(name(version))]
    pub(crate) value: u8,
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
        Self::new(self.value + rhs)
    }
}

impl AddAssign<u8> for Version {
    fn add_assign(&mut self, rhs: u8) {
        self.value += rhs;
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::MIN
    }
}

impl PartialEq<Range<Self>> for Version {
    fn eq(&self, other: &Range<Self>) -> bool {
        self >= &other.start && self < &other.end
    }
}

impl PartialOrd<Range<Self>> for Version {
    fn partial_cmp(&self, other: &Range<Self>) -> Option<Ordering> {
        match self {
            _ if self < &other.start => Some(Ordering::Less),
            _ if self >= &other.end => Some(Ordering::Greater),
            _ => Some(Ordering::Equal),
        }
    }
}

impl Sub<u8> for Version {
    type Output = Self;

    fn sub(self, rhs: u8) -> Self::Output {
        Self::new(self.value - rhs)
    }
}

impl SubAssign<u8> for Version {
    fn sub_assign(&mut self, rhs: u8) {
        self.value -= rhs;
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::event::version::Version;

    #[test]
    fn new_creates_version_with_given_value() {
        let ver = Version::new(42);

        assert_eq!(ver.value, 42);
    }

    #[test]
    fn min_constant_equals_zero() {
        assert_eq!(Version::MIN.value, 0);
        assert_eq!(Version::MIN.value, u8::MIN);
    }

    #[test]
    fn max_constant_equals_u8_max() {
        assert_eq!(Version::MAX.value, u8::MAX);
        assert_eq!(Version::MAX.value, 255);
    }

    #[test]
    fn default_equals_min() {
        assert_eq!(Version::default(), Version::MIN);
        assert_eq!(Version::default().value, 0);
    }

    #[test]
    fn deref_returns_inner_value() {
        let ver = Version::new(123);
        let value: &u8 = &ver.value;

        assert_eq!(*value, 123);
    }

    #[allow(clippy::clone_on_copy)]
    #[test]
    fn clone_creates_equal_copy() {
        let ver1 = Version::new(100);
        let ver2 = ver1.clone();

        assert_eq!(ver1, ver2);
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

        assert_eq!(result.value, 30);
    }

    #[test]
    fn add_assign_u8_to_version() {
        let mut ver = Version::new(10);
        ver += 20u8;

        assert_eq!(ver.value, 30);
    }

    #[test]
    fn sub_u8_from_version() {
        let ver = Version::new(50);
        let result = ver - 20u8;

        assert_eq!(result.value, 30);
    }

    #[test]
    fn sub_assign_u8_from_version() {
        let mut ver = Version::new(50);
        ver -= 20u8;

        assert_eq!(ver.value, 30);
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

        assert_eq!(result.value, 120);
    }

    #[test]
    fn min_plus_one() {
        let ver = Version::MIN + 1u8;

        assert_eq!(ver.value, 1);
    }

    #[test]
    fn max_minus_one() {
        let ver = Version::MAX - 1u8;

        assert_eq!(ver.value, 254);
    }

    #[test]
    fn version_zero_is_valid() {
        let ver = Version::new(0);

        assert_eq!(ver.value, 0);
        assert_eq!(ver, Version::MIN);
    }

    #[test]
    fn version_max_is_valid() {
        let ver = Version::new(255);

        assert_eq!(ver.value, 255);
        assert_eq!(ver, Version::MAX);
    }

    #[test]
    fn increment_version() {
        let mut ver = Version::new(1);
        ver += 1u8;

        assert_eq!(ver.value, 2);

        ver += 1u8;

        assert_eq!(ver.value, 3);
    }

    #[test]
    fn decrement_version() {
        let mut ver = Version::new(10);
        ver -= 1u8;

        assert_eq!(ver.value, 9);

        ver -= 1u8;

        assert_eq!(ver.value, 8);
    }

    // PartialEq<Range<Self>>

    #[test]
    fn version_equal_to_range_when_inside() {
        let ver = Version::new(5);
        let range = Version::new(3)..Version::new(10);

        assert_eq!(ver, range);
        assert!(ver == range);
    }

    #[test]
    fn version_not_equal_to_range_when_before_start() {
        let ver = Version::new(2);
        let range = Version::new(5)..Version::new(10);

        assert_ne!(ver, range);
        assert!(ver != range);
    }

    #[test]
    fn version_not_equal_to_range_when_at_or_after_end() {
        let ver = Version::new(10);
        let range = Version::new(5)..Version::new(10);

        assert_ne!(ver, range);
        assert!(ver != range);
    }

    #[test]
    fn version_equal_to_range_at_start() {
        let ver = Version::new(5);
        let range = Version::new(5)..Version::new(10);

        assert_eq!(ver, range);
    }

    #[test]
    fn version_not_equal_to_range_at_exclusive_end() {
        let ver = Version::new(10);
        let range = Version::new(5)..Version::new(10);

        assert_ne!(ver, range);
    }

    #[test]
    fn version_not_equal_to_range_one_before_start() {
        let ver = Version::new(4);
        let range = Version::new(5)..Version::new(10);

        assert_ne!(ver, range);
    }

    #[test]
    fn version_equal_to_range_one_after_start() {
        let ver = Version::new(6);
        let range = Version::new(5)..Version::new(10);

        assert_eq!(ver, range);
    }

    #[test]
    fn version_equal_to_range_one_before_end() {
        let ver = Version::new(9);
        let range = Version::new(5)..Version::new(10);

        assert_eq!(ver, range);
    }

    #[test]
    fn version_not_equal_to_range_one_after_end() {
        let ver = Version::new(11);
        let range = Version::new(5)..Version::new(10);

        assert_ne!(ver, range);
    }

    #[test]
    fn version_not_equal_to_empty_range() {
        let ver = Version::new(5);
        let range = Version::new(5)..Version::new(5);

        assert_ne!(ver, range);
    }

    #[test]
    fn version_equal_to_single_value_range() {
        let ver = Version::new(5);
        let range = Version::new(5)..Version::new(6);

        assert_eq!(ver, range);
    }

    #[test]
    fn version_not_equal_to_single_value_range_outside() {
        let ver = Version::new(6);
        let range = Version::new(5)..Version::new(6);

        assert_ne!(ver, range);
    }

    #[test]
    fn version_equal_to_full_range() {
        let ver = Version::new(100);
        let range = Version::MIN..Version::MAX;

        assert_eq!(ver, range);
    }

    #[test]
    fn version_max_not_equal_to_full_range() {
        let ver = Version::MAX;
        let range = Version::MIN..Version::MAX;

        assert_ne!(ver, range);
    }

    #[test]
    fn version_min_equal_to_range_starting_at_min() {
        let ver = Version::MIN;
        let range = Version::MIN..Version::new(10);

        assert_eq!(ver, range);
    }

    // PartialOrd<Range<Self>>

    #[test]
    fn version_less_than_range_when_before_start() {
        let ver = Version::new(2);
        let range = Version::new(5)..Version::new(10);

        assert!(ver < range);
        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Less));
    }

    #[test]
    fn version_equal_to_range_when_inside_for_ordering() {
        let ver = Version::new(7);
        let range = Version::new(5)..Version::new(10);

        assert!(ver >= range);
        assert!(ver <= range);
        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Equal));
    }

    #[test]
    fn version_greater_than_range_when_at_or_after_end() {
        let ver = Version::new(10);
        let range = Version::new(5)..Version::new(10);

        assert!(ver > range);
        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Greater));
    }

    #[test]
    fn version_equal_to_range_at_start_for_ordering() {
        let ver = Version::new(5);
        let range = Version::new(5)..Version::new(10);

        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Equal));
        assert!(ver >= range);
        assert!(ver <= range);
    }

    #[test]
    fn version_greater_than_range_at_exclusive_end_for_ordering() {
        let ver = Version::new(10);
        let range = Version::new(5)..Version::new(10);

        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Greater));
        assert!(ver > range);
    }

    #[test]
    fn version_less_than_range_one_before_start() {
        let ver = Version::new(4);
        let range = Version::new(5)..Version::new(10);

        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Less));
    }

    #[test]
    fn version_equal_to_range_one_after_start_for_ordering() {
        let ver = Version::new(6);
        let range = Version::new(5)..Version::new(10);

        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Equal));
    }

    #[test]
    fn version_equal_to_range_one_before_end_for_ordering() {
        let ver = Version::new(9);
        let range = Version::new(5)..Version::new(10);

        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Equal));
    }

    #[test]
    fn version_greater_than_range_one_after_end() {
        let ver = Version::new(11);
        let range = Version::new(5)..Version::new(10);

        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Greater));
    }

    #[test]
    fn version_greater_than_empty_range() {
        let ver = Version::new(5);
        let range = Version::new(5)..Version::new(5);

        assert!(ver > range);
        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Greater));
    }

    #[test]
    fn version_equal_to_single_value_range_for_ordering() {
        let ver = Version::new(5);
        let range = Version::new(5)..Version::new(6);

        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Equal));
    }

    #[test]
    fn version_greater_than_single_value_range_outside() {
        let ver = Version::new(6);
        let range = Version::new(5)..Version::new(6);

        assert!(ver > range);
        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Greater));
    }

    #[test]
    fn version_min_less_than_range_not_starting_at_min() {
        let ver = Version::MIN;
        let range = Version::new(1)..Version::new(10);

        assert!(ver < range);
        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Less));
    }

    #[test]
    fn version_max_greater_than_range_not_ending_at_max() {
        let ver = Version::MAX;
        let range = Version::new(0)..Version::new(100);

        assert!(ver > range);
        assert_eq!(ver.partial_cmp(&range), Some(Ordering::Greater));
    }

    #[test]
    fn version_boundary_tests_with_min_max() {
        assert!(Version::MIN < (Version::new(1)..Version::MAX));
        assert!(Version::MAX > (Version::MIN..Version::MAX));
        assert_eq!(
            Version::new(50).partial_cmp(&(Version::MIN..Version::MAX)),
            Some(Ordering::Equal)
        );
    }

    #[test]
    fn version_range_comparison_transitivity() {
        let range = Version::new(10)..Version::new(20);
        let ver_before = Version::new(5);
        let ver_inside = Version::new(15);
        let ver_after = Version::new(25);

        assert!(ver_before < range);
        assert_eq!(ver_inside.partial_cmp(&range), Some(Ordering::Equal));
        assert!(ver_after > range);
    }

    #[test]
    fn version_range_edge_cases_zero_and_max() {
        let ver_zero = Version::new(0);
        let ver_one = Version::new(1);
        let ver_max = Version::MAX;
        let ver_max_minus_one = Version::MAX - 1;

        assert_eq!(
            ver_zero.partial_cmp(&(Version::new(0)..Version::new(1))),
            Some(Ordering::Equal)
        );
        assert!(ver_one > (Version::new(0)..Version::new(1)));
        assert_eq!(
            ver_max_minus_one.partial_cmp(&(Version::new(254)..Version::MAX)),
            Some(Ordering::Equal)
        );
        assert!(ver_max > (Version::new(254)..Version::MAX));
    }
}
