use fancy_constructor::new;

use crate::event::{
    identifier::{
        Identifier,
        IdentifierHash,
        IdentifierHashRef,
    },
    version::Version,
};

// =================================================================================================
// Descriptor
// =================================================================================================

#[derive(new, Debug, Eq, PartialEq)]
#[new(const_fn)]
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

// Hash

#[derive(new, Debug)]
#[new(const_fn)]
pub struct DescriptorHash(IdentifierHash, Version);

impl DescriptorHash {
    #[must_use]
    pub fn take(self) -> (IdentifierHash, Version) {
        (self.0, self.1)
    }
}

// Hash Ref

#[derive(new, Debug)]
#[new(const_fn)]
pub struct DescriptorHashRef<'a>(IdentifierHashRef<'a>, Version);

impl<'a> DescriptorHashRef<'a> {
    #[must_use]
    pub fn identifer(&self) -> &IdentifierHashRef<'a> {
        &self.0
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        &self.1
    }
}

impl<'a> From<&'a Descriptor> for DescriptorHashRef<'a> {
    fn from(descriptor: &'a Descriptor) -> Self {
        let identifier = descriptor.identifier().into();
        let version = *descriptor.version();

        Self::new(identifier, version)
    }
}

// Ref

#[derive(new, Debug)]
#[new(const_fn)]
pub struct DescriptorRef<'a>(&'a Identifier, Version);

impl DescriptorRef<'_> {
    #[must_use]
    pub fn identifer(&self) -> &Identifier {
        self.0
    }

    #[must_use]
    pub fn version(&self) -> &Version {
        &self.1
    }
}
