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

#[rustfmt::skip]
#[derive(Add, AddAssign, Clone, Copy, Debug, Deref, Eq, Ord, PartialEq, PartialOrd, Sub, SubAssign)]
pub struct Position(u64);

impl Position {
    #[must_use]
    pub const fn new(position: u64) -> Self {
        Self(position)
    }
}

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
