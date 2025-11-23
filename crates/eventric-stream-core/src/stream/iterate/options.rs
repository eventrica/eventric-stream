use std::sync::Arc;

use fancy_constructor::new;

use crate::stream::iterate::cache::Cache;

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
    pub(crate) cache: Arc<Cache>,
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
        self.cache = cache;
        self
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new_inner()
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::stream::iterate::{
        cache::Cache,
        options::Options,
    };

    // Options::default

    #[test]
    fn default_creates_options_with_defaults() {
        let options = Options::default();

        assert!(!options.retrieve_tags);
        assert_eq!(0, options.cache.identifiers.len());
        assert_eq!(0, options.cache.tags.len());
    }

    // Options::retrieve_tags

    #[test]
    fn retrieve_tags_sets_flag_to_true() {
        let options = Options::default().retrieve_tags(true);

        assert!(options.retrieve_tags);
    }

    #[test]
    fn retrieve_tags_sets_flag_to_false() {
        let options = Options::default().retrieve_tags(true).retrieve_tags(false);

        assert!(!options.retrieve_tags);
    }

    #[test]
    fn retrieve_tags_defaults_to_false() {
        let options = Options::default();

        assert!(!options.retrieve_tags);
    }

    // Options::with_shared_cache

    #[test]
    fn with_shared_cache_sets_cache() {
        let cache = Arc::new(Cache::default());
        let cache_clone = Arc::clone(&cache);

        let options = Options::default().with_shared_cache(cache);

        // Verify it's the same Arc (same pointer)
        assert!(Arc::ptr_eq(&options.cache, &cache_clone));
    }

    #[test]
    fn with_shared_cache_replaces_existing_cache() {
        let cache1 = Arc::new(Cache::default());
        let cache2 = Arc::new(Cache::default());
        let cache2_clone = Arc::clone(&cache2);

        let options = Options::default()
            .with_shared_cache(cache1)
            .with_shared_cache(cache2);

        assert!(Arc::ptr_eq(&options.cache, &cache2_clone));
    }

    // Chaining

    #[test]
    fn methods_can_be_chained() {
        let cache = Arc::new(Cache::default());
        let cache_clone = Arc::clone(&cache);

        let options = Options::default()
            .retrieve_tags(true)
            .with_shared_cache(cache);

        assert!(options.retrieve_tags);
        assert!(Arc::ptr_eq(&options.cache, &cache_clone));
    }

    #[test]
    fn methods_can_be_chained_in_any_order() {
        let cache = Arc::new(Cache::default());
        let cache_clone = Arc::clone(&cache);

        let options = Options::default()
            .with_shared_cache(cache)
            .retrieve_tags(true);

        assert!(options.retrieve_tags);
        assert!(Arc::ptr_eq(&options.cache, &cache_clone));
    }

    // Clone

    #[test]
    fn clone_creates_copy_with_same_values() {
        let cache = Arc::new(Cache::default());
        let options = Options::default()
            .retrieve_tags(true)
            .with_shared_cache(cache);

        let cloned = options.clone();

        assert_eq!(options.retrieve_tags, cloned.retrieve_tags);
        assert!(Arc::ptr_eq(&options.cache, &cloned.cache));
    }

    #[test]
    fn clone_shares_same_cache_arc() {
        let cache = Arc::new(Cache::default());
        let options = Options::default().with_shared_cache(cache);

        let cloned = options.clone();

        // Both should point to the same Arc
        assert!(Arc::ptr_eq(&options.cache, &cloned.cache));
        assert_eq!(2, Arc::strong_count(&options.cache));
    }
}
