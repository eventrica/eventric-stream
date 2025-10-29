use fancy_constructor::new;

// =================================================================================================
// Options
// =================================================================================================

#[derive(new, Debug)]
#[new(name(new_inner), vis())]
pub struct Options {
    #[new(default)]
    pub(crate) retrieve_tags: bool,
}

impl Options {
    #[must_use]
    pub fn retrieve_tags(mut self, retrieve_tags: bool) -> Self {
        self.retrieve_tags = retrieve_tags;
        self
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new_inner()
    }
}
