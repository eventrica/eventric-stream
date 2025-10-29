use derive_more::Deref;

// =================================================================================================
// Version
// =================================================================================================

/// The [`Version`] type is a typed wrapper around a `u8` version value, which
/// should be used as a monotonic indicator of the *type version* of the event.
/// When paired with the [`Identifier`][ident] value, the pair forms a
/// specification of the logical versioned *type* of the event.
///
/// [ident]: crate::event::identifier::Identifier
#[derive(Clone, Copy, Debug, Deref, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version(u8);

impl Version {
    /// Constructs a new instance of [`Version`] from a given `u8` version
    /// value.
    #[must_use]
    pub const fn new(version: u8) -> Self {
        Self(version)
    }
}
