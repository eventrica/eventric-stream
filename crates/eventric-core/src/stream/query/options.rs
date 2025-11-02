use std::sync::Arc;

use fancy_constructor::new;

use crate::stream::query::cache::Cache;

// =================================================================================================
// Options
// =================================================================================================

/// The [`Options`] type supplies additional options to the
/// [`Stream::query`][query] operation, which modify the behaviour of the query
/// (such as what data is returned, etc.).
///
/// [query]: crate::stream::Stream::query
#[derive(new, Clone, Debug)]
#[new(name(new_inner), vis())]
pub struct Options {
    #[new(default)]
    pub(crate) cache: Option<Arc<Cache>>,
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

    /// Sets whether queries should use a shared [`Cache`] instance as supplied.
    /// This can reduce the requirement for fetching data from the underlying
    /// databasse, particularly where multiple similar queries are executed in
    /// close proximity.
    #[must_use]
    pub fn with_shared_cache(mut self, cache: Arc<Cache>) -> Self {
        self.cache = Some(cache);
        self
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new_inner()
    }
}
