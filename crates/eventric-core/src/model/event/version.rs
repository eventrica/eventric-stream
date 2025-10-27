use derive_more::Deref;
use fancy_constructor::new;

// =================================================================================================
// Version
// =================================================================================================

#[derive(new, Clone, Copy, Debug, Deref, Eq, Ord, PartialEq, PartialOrd)]
#[new(const_fn, name(new_inner), vis())]
pub struct Version(u8);

impl Version {
    #[must_use]
    pub const fn new(version: u8) -> Self {
        Self::new_inner(version)
    }
}
