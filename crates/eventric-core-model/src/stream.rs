use derive_more::Debug;
use fancy_constructor::new;

// =================================================================================================
// Stream
// =================================================================================================

// Position

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(vis())]
pub struct Position(#[new(into)] u64);

impl Position {
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

impl<T> From<T> for Position
where
    T: Into<u64>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
