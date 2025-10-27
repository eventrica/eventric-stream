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

#[rustfmt::skip]
#[derive(new, Add, AddAssign, Clone, Copy, Debug, Deref, Eq, Ord, PartialEq, PartialOrd, Sub, SubAssign)]
#[new(args(position: u64), const_fn)]
pub struct Position(#[new(val(position))] u64);

impl Position {
    pub const MAX: Position = Position::new(u64::MAX);
    pub const MIN: Position = Position::new(u64::MIN);
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
