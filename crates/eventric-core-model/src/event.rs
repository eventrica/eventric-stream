pub mod insertion;

use fancy_constructor::new;

// =================================================================================================
// Event
// =================================================================================================

// Descriptor

#[derive(new, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(vis())]
pub struct Descriptor(#[new(into)] Identifier, #[new(into)] Version);

impl Descriptor {
    #[must_use]
    pub fn identifier(&self) -> &Identifier {
        &self.0
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        &self.1
    }

    #[must_use]
    pub fn take(self) -> (Identifier, Version) {
        (self.0, self.1)
    }
}

impl<T, U> From<(T, U)> for Descriptor
where
    T: Into<Identifier>,
    U: Into<Version>,
{
    fn from(value: (T, U)) -> Self {
        Self::new(value.0, value.1)
    }
}

// Identifier

#[derive(new, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(vis())]
pub struct Identifier(#[new(into)] String);

impl Identifier {
    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl<T> From<T> for Identifier
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

// Version

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(vis())]
pub struct Version(#[new(into)] u8);

impl Version {
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }
}

impl<T> From<T> for Version
where
    T: Into<u8>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

// -------------------------------------------------------------------------------------------------

// Tag

#[derive(new, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(vis(pub))]
pub struct Tag(#[new(into)] String);

impl Tag {
    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl<T> From<T> for Tag
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
