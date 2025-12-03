use derive_more::{
    Deref,
    with_trait::{
        Add,
        AddAssign,
        Sub,
        SubAssign,
    },
};
use fancy_constructor::new;

// =================================================================================================
// Position
// =================================================================================================

/// The [`Position`] type is a typed wrapper around a `u64` value representing
/// the ordinal position of an event in a stream.
#[rustfmt::skip]
#[derive(new, Add, AddAssign, Clone, Copy, Debug, Deref, Eq, Ord, PartialEq, PartialOrd, Sub, SubAssign)]
#[new(const_fn)]
pub struct Position {
    value: u64
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
        Self {
            value: self.value + rhs,
        }
    }
}

impl AddAssign<u64> for Position {
    fn add_assign(&mut self, rhs: u64) {
        self.value += rhs;
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
        Self {
            value: self.value - rhs,
        }
    }
}

impl SubAssign<u64> for Position {
    fn sub_assign(&mut self, rhs: u64) {
        self.value -= rhs;
    }
}

// =================================================================================================
// Tests
// =================================================================================================

#[cfg(test)]
mod tests {
    use crate::event::position::Position;

    #[test]
    fn new_creates_position_with_given_value() {
        let pos = Position::new(42);

        assert_eq!(*pos, 42);
    }

    #[test]
    fn min_constant_equals_zero() {
        assert_eq!(*Position::MIN, 0);
        assert_eq!(*Position::MIN, u64::MIN);
    }

    #[test]
    fn max_constant_equals_u64_max() {
        assert_eq!(*Position::MAX, u64::MAX);
    }

    #[test]
    fn default_equals_min() {
        assert_eq!(Position::default(), Position::MIN);
        assert_eq!(*Position::default(), 0);
    }

    #[test]
    fn deref_returns_inner_value() {
        let pos = Position::new(123);
        let value: &u64 = &pos;

        assert_eq!(*value, 123);
    }

    #[allow(clippy::clone_on_copy)]
    #[test]
    fn clone_creates_equal_copy() {
        let pos1 = Position::new(100);
        let pos2 = pos1.clone();

        assert_eq!(pos1, pos2);
        assert_eq!(*pos1, *pos2);
    }

    #[test]
    fn copy_creates_equal_copy() {
        let pos1 = Position::new(100);
        let pos2 = pos1; // Copy trait

        assert_eq!(pos1, pos2);
    }

    #[test]
    fn add_position_to_position() {
        let pos1 = Position::new(10);
        let pos2 = Position::new(20);
        let result = pos1 + pos2;

        assert_eq!(*result, 30);
    }

    #[test]
    fn add_u64_to_position() {
        let pos = Position::new(10);
        let result = pos + 20u64;

        assert_eq!(*result, 30);
    }

    #[test]
    fn add_assign_position_to_position() {
        let mut pos = Position::new(10);
        pos += Position::new(20);

        assert_eq!(*pos, 30);
    }

    #[test]
    fn add_assign_u64_to_position() {
        let mut pos = Position::new(10);
        pos += 20u64;

        assert_eq!(*pos, 30);
    }

    #[test]
    fn sub_position_from_position() {
        let pos1 = Position::new(50);
        let pos2 = Position::new(20);
        let result = pos1 - pos2;

        assert_eq!(*result, 30);
    }

    #[test]
    fn sub_u64_from_position() {
        let pos = Position::new(50);
        let result = pos - 20u64;

        assert_eq!(*result, 30);
    }

    #[test]
    fn sub_assign_position_from_position() {
        let mut pos = Position::new(50);
        pos -= Position::new(20);

        assert_eq!(*pos, 30);
    }

    #[test]
    fn sub_assign_u64_from_position() {
        let mut pos = Position::new(50);
        pos -= 20u64;

        assert_eq!(*pos, 30);
    }

    #[test]
    fn equality_comparison() {
        let pos1 = Position::new(42);
        let pos2 = Position::new(42);
        let pos3 = Position::new(43);

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn ordering_comparison() {
        let pos1 = Position::new(10);
        let pos2 = Position::new(20);
        let pos3 = Position::new(30);

        assert!(pos1 < pos2);
        assert!(pos2 > pos1);
        assert!(pos1 <= pos2);
        assert!(pos2 >= pos1);
        assert!(pos1 <= pos1);
        assert!(pos1 >= pos1);

        assert!(pos1 < pos3);
        assert!(pos2 < pos3);
    }

    #[test]
    fn debug_format() {
        let pos = Position::new(42);
        let debug_str = format!("{pos:?}");

        assert!(debug_str.contains("Position"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn arithmetic_chain() {
        let pos = Position::new(100);
        let result = pos + 50u64 - 30u64 + Position::new(10);

        assert_eq!(*result, 130);
    }

    #[test]
    fn min_plus_one() {
        let pos = Position::MIN + 1u64;

        assert_eq!(*pos, 1);
    }

    #[test]
    fn max_minus_one() {
        let pos = Position::MAX - 1u64;

        assert_eq!(*pos, u64::MAX - 1);
    }
}
