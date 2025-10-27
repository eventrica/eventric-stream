use std::ops::{
    Add,
    AddAssign,
};

use fancy_constructor::new;

// =================================================================================================
// Position
// =================================================================================================

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(const_fn)]
pub struct Position(u64);

impl Position {
    pub const MAX: Position = Position::new(u64::MAX);
    pub const MIN: Position = Position::new(u64::MIN);
}

impl Position {
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Self) -> Self::Output {
        Position(self.0 + rhs.0)
    }
}

impl Add<u64> for Position {
    type Output = Position;

    fn add(self, rhs: u64) -> Self::Output {
        Position(self.0 + rhs)
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl AddAssign<u64> for Position {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}
