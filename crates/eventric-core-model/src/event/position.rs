use fancy_constructor::new;

// =================================================================================================
// Position
// =================================================================================================

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(const_fn)]
pub struct Position(u64);

impl Position {
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}
