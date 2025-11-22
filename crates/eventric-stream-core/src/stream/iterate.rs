//! See the `eventric-stream` crate for full documentation, including
//! module-level documentation.

pub(crate) mod cache;
pub(crate) mod condition;
pub(crate) mod iter;
pub(crate) mod options;

use std::sync::Exclusive;

use crate::{
    event::position::Position,
    stream::{
        Stream,
        data::events::{
            MappedPersistentEventHashIterator,
            PersistentEventHashIterator,
        },
        query::{
            QueryHash,
            QueryHashRef,
        },
    },
};

// =================================================================================================
// Iterate
// =================================================================================================

impl Stream {
    /// Queries the [`Stream`] based on the supplied [`Condition`], using the
    /// [`Cache`] to avoid re-fetching intermediate components such as
    /// [`Identifier`][identifier]s and [`Tag`]s, and optionally configured by
    /// [`Options`] to determine which event data is returned.
    ///
    /// TODO: [Full query documentation + examples][issue]
    ///
    /// # Errors
    ///
    /// Returns an error in the case of an underlying IO/database error.
    ///
    /// [identifier]: crate::event::Identifier
    /// [issue]: https://github.com/eventrica/eventric-stream/issues/21
    #[must_use]
    pub fn query(&self, condition: &Condition<'_>, options: Option<Options>) -> Iter {
        let options = options.unwrap_or_default();
        let references = self.data.references.clone();

        let iter = condition.matches.map_or_else(
            || self.query_events(condition.from),
            |query| {
                let query_hash_ref: QueryHashRef<'_> = query.into();
                let query_hash: QueryHash = (&query_hash_ref).into();

                options.cache.populate(&query_hash_ref);

                self.query_indices(&query_hash, condition.from)
            },
        );
        let iter = Exclusive::new(iter);

        Iter::new(options, references, iter)
    }

    fn query_events(&self, from: Option<Position>) -> PersistentEventHashIterator {
        let iter = self.data.events.iterate(from);

        PersistentEventHashIterator::Direct(iter)
    }

    fn query_indices(
        &self,
        query: &QueryHash,
        from: Option<Position>,
    ) -> PersistentEventHashIterator {
        let events = self.data.events.clone();
        let iter = self.data.indices.query(query, from);
        let iter = MappedPersistentEventHashIterator::new(events, iter);

        PersistentEventHashIterator::Mapped(iter)
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Export

pub use self::{
    cache::Cache,
    condition::Condition,
    iter::Iter,
    options::Options,
};
