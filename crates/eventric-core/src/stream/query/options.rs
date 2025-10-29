use fancy_constructor::new;

// =================================================================================================
// Options
// =================================================================================================

/// The [`Options`] type supplies additional options to the
/// [`Stream::query`][query] operation, which modify the behaviour of the query
/// (such as what data is returned, etc.).
///
/// [query]: crate::stream::Stream::query
#[derive(new, Debug)]
#[new(name(new_inner), vis())]
pub struct Options {
    #[new(default)]
    pub(crate) retrieve_tags: bool,
}

impl Options {
    /// Sets whether queries should fetch event tags that are *not already
    /// present in the query or cache*. If this is set to true, all tags on
    /// events will be fetched from the data store - this has a memory and
    /// performance implication, and is rarely required (tags should generally
    /// not be used as a source of information, and generally any data implied
    /// by a tag is already present in the event).
    ///
    /// This defaults to *false*.
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
