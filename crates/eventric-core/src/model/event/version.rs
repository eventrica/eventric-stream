use derive_more::Deref;

// =================================================================================================
// Version
// =================================================================================================

#[derive(Clone, Copy, Debug, Deref, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version(u8);

impl Version {
    #[must_use]
    pub const fn new(version: u8) -> Self {
        Self(version)
    }
}
