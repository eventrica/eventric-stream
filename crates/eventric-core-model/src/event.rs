pub mod insertion;

use fancy_constructor::new;

// =================================================================================================
// Event
// =================================================================================================

// Data

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct Data(#[new(into)] Vec<u8>);

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

// -------------------------------------------------------------------------------------------------

// Descriptor

#[derive(new, Debug, Eq, PartialEq)]
#[new(vis(pub))]
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
}

// Identifier

#[derive(new, Debug, Eq, PartialEq)]
#[new(vis(pub))]
pub struct Identifier(#[new(into)] String);

impl Identifier {
    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

// Version

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(vis(pub))]
pub struct Version(#[new(into)] u8);

impl Version {
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }
}

// -------------------------------------------------------------------------------------------------

// Tag

#[derive(new, Debug, Eq, PartialEq)]
#[new(vis(pub))]
pub struct Tag(#[new(into)] String);

impl Tag {
    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}
