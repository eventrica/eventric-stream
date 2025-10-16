use fancy_constructor::new;

// =================================================================================================
// Version
// =================================================================================================

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(vis(pub))]
pub struct Version(u8);

impl Version {
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }
}
