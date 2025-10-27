use derive_more::Deref;
use fancy_constructor::new;

// =================================================================================================
// Version
// =================================================================================================

#[derive(new, Clone, Copy, Debug, Deref, Eq, Ord, PartialEq, PartialOrd)]
#[new(args(version: u8), const_fn)]
pub struct Version(#[new(val(version))] u8);
