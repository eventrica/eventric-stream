use fancy_constructor::new;
use rapidhash::v3::{
    self,
    RapidSecrets,
};

// =================================================================================================
// Event
// =================================================================================================

// Configuration

static SEED: RapidSecrets = RapidSecrets::seed(0x2811_2017);

// -------------------------------------------------------------------------------------------------

// Data

#[derive(new, Debug)]
#[new(vis(pub))]
pub struct Data(Vec<u8>);

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

// -------------------------------------------------------------------------------------------------

// Descriptor

#[derive(new, Debug, Eq, PartialEq)]
#[new(vis(pub))]
pub struct Descriptor(Identifier, Version);

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

// -------------------------------------------------------------------------------------------------

// Identifier

#[derive(new, Clone, Debug, Eq, PartialEq)]
#[new(vis(pub))]
pub struct Identifier(String);

impl Identifier {
    #[must_use]
    pub fn hash(&self) -> u64 {
        v3::rapidhash_v3_seeded(self.0.as_bytes(), &SEED)
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

// -------------------------------------------------------------------------------------------------

// Position

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(vis(pub))]
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

// -------------------------------------------------------------------------------------------------

// Tag

#[derive(new, Clone, Debug, Eq, PartialEq)]
#[new(vis(pub))]
pub struct Tag(String);

impl Tag {
    #[must_use]
    pub fn hash(&self) -> u64 {
        v3::rapidhash_v3_seeded(self.0.as_bytes(), &SEED)
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

// -------------------------------------------------------------------------------------------------

// Version

#[derive(new, Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[new(vis(pub))]
pub struct Version(u8);

impl Version {
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }
}
