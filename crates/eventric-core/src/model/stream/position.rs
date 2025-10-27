use derive_more::with_trait::{
    Add,
    AddAssign,
    Sub,
    SubAssign,
};
use fancy_constructor::new;

// =================================================================================================
// Position
// =================================================================================================

#[rustfmt::skip]
#[derive(new, Add, AddAssign, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Sub, SubAssign)]
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

impl Add<u64> for Position {
    type Output = Position;

    fn add(self, rhs: u64) -> Self::Output {
        Position(self.0 + rhs)
    }
}

impl AddAssign<u64> for Position {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl Sub<u64> for Position {
    type Output = Position;

    fn sub(self, rhs: u64) -> Self::Output {
        Position(self.0 - rhs)
    }
}

impl SubAssign<u64> for Position {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 -= rhs;
    }
}
