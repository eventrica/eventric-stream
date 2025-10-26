use fancy_constructor::new;

// =================================================================================================
// Data
// =================================================================================================

#[derive(new, Debug)]
#[new(const_fn)]
pub struct Data(Vec<u8>);

impl Data {
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.0[..]
    }
}

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
