//! See the `eventric-stream` crate for full documentation, including
//! module-level documentation.

pub(crate) mod cache;
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
            Query,
            QueryHash,
            QueryHashRef,
            filter::Filter,
        },
    },
};

// =================================================================================================
// Iterate
// =================================================================================================

// Iterate

/// .
pub trait Iterate {
    /// .
    fn iterate(&self, from: Option<Position>) -> Iter;

    /// .
    fn iterate_with_options(&self, from: Option<Position>, options: Options) -> Iter;
}

impl Iterate for Stream {
    fn iterate(&self, from: Option<Position>) -> Iter {
        self.iterate_with_options(from, Options::default())
    }

    fn iterate_with_options(&self, from: Option<Position>, options: Options) -> Iter {
        let references = self.data.references.clone();

        let iter = self.iter_events(from);
        let iter = Exclusive::new(iter);

        Iter::new(options, references, iter)
    }
}

// -------------------------------------------------------------------------------------------------

// Iterate Query

/// The [`IterateQuery`] trait defines the logical operation of iterating over a
/// stream or stream-like type, using a supplied [`Query`]) to determine
/// which matching events should be returned, and an optional [`Position`] at
/// which iteration should begin.
pub trait IterateQuery {
    /// Iterates over the stream or stream-like instance using the given
    /// [`Query`] to determine which matching events should be returned. Will
    /// begin iteration at given `from` [`Position`] if one is supplied.
    ///
    /// TODO: [Full query documentation + examples][issue]
    ///
    /// # Errors
    ///
    /// Returns an error in the case of an underlying IO/database error.
    ///
    /// [identifier]: crate::event::identifier::Identifier
    /// [tag]: crate::event::tag::Tag
    /// [issue]: https://github.com/eventrica/eventric-stream/issues/21
    fn iterate_query(&self, query: Query, from: Option<Position>) -> (Iter, QueryHash);

    /// Iterates over the stream or stream-like instance using the given
    /// [`Query`] to determine which matching events should be returned. Will
    /// begin iteration at given `from` [`Position`] if one is supplied. The
    /// supplied [`Options`] determine which data should be retrived and
    /// included during iteration, and also allows for the sharing of a
    /// retrieval cache across multiple iterations.
    ///
    /// TODO: [Full query documentation + examples][issue]
    ///
    /// # Errors
    ///
    /// Returns an error in the case of an underlying IO/database error.
    ///
    /// [issue]: https://github.com/eventrica/eventric-stream/issues/21
    fn iterate_query_with_options(
        &self,
        query: Query,
        from: Option<Position>,
        options: Options,
    ) -> (Iter, QueryHash);
}

impl IterateQuery for Stream {
    fn iterate_query(&self, query: Query, from: Option<Position>) -> (Iter, QueryHash) {
        self.iterate_query_with_options(query, from, Options::default())
    }

    fn iterate_query_with_options(
        &self,
        query: Query,
        from: Option<Position>,
        options: Options,
    ) -> (Iter, QueryHash) {
        let references = self.data.references.clone();

        let query_hash_ref: QueryHashRef<'_> = (&query).into();
        let query_hash: QueryHash = (&query_hash_ref).into();

        options.cache.populate(&query_hash_ref);

        let iter = self.iter_indices(&query_hash, from);
        let iter = Exclusive::new(iter);
        let iter = Iter::new(options, references, iter);

        (iter, query_hash)
    }
}

// -------------------------------------------------------------------------------------------------

// Iterate Query (Multiple)

/// The [`IterateMulti`] trait defines the logical operation of iterating over a
/// stream or stream-like type, using a supplied condition to determine which
/// events should be returned (filtering by an optional vector of
/// [`Query`][query] instances), and whether the iteration should begin at a
/// particular [`Position`].
///
/// [query]: crate::stream::query::Query
pub trait IterateQueryMulti {
    /// Iterates over the stream or stream-like instance based on the supplied
    /// [`ConditionMulti`], using the [`Options`] [`Cache`] to avoid re-fetching
    /// intermediate components such as [`Identifier`][identifier]s and
    /// [`Tag`][tag]s, and optionally configured by to determine what event
    /// metadata is returned.
    ///
    /// TODO: [Full query documentation + examples][issue]
    ///
    /// # Errors
    ///
    /// Returns an error in the case of an underlying IO/database error.
    ///
    /// [identifier]: crate::event::identifier::Identifier
    /// [tag]: crate::event::tag::Tag
    /// [issue]: https://github.com/eventrica/eventric-stream/issues/21
    fn iterate_query_multi(
        &self,
        queries: Vec<Query>,
        from: Option<Position>,
    ) -> (IterMulti, QueryHash);

    /// .
    fn iterate_query_multi_with_options(
        &self,
        queries: Vec<Query>,
        from: Option<Position>,
        options: Options,
    ) -> (IterMulti, QueryHash);
}

impl IterateQueryMulti for Stream {
    fn iterate_query_multi(
        &self,
        queries: Vec<Query>,
        from: Option<Position>,
    ) -> (IterMulti, QueryHash) {
        self.iterate_query_multi_with_options(queries, from, Options::default())
    }

    fn iterate_query_multi_with_options(
        &self,
        queries: Vec<Query>,
        from: Option<Position>,
        options: Options,
    ) -> (IterMulti, QueryHash) {
        let references = self.data.references.clone();

        let filters = queries
            .iter()
            .map(|query| Filter::new(&query.into()))
            .collect();

        let selectors = queries
            .into_iter()
            .flat_map(|query| query.selectors)
            .collect::<Vec<_>>();

        // TODO: Need to do some kind of merge/optimisation pass here, not simply bodge
        // all the selectors together, even though that will technically work,
        // it could be horribly inefficient.

        let query = Query::new_unvalidated(selectors);
        let query_hash_ref: QueryHashRef<'_> = (&query).into();
        let query_hash: QueryHash = (&query_hash_ref).into();

        options.cache.populate(&query_hash_ref);

        let iter = self.iter_indices(&query_hash, from);
        let iter = Exclusive::new(iter);
        let iter = IterMulti::new(options, references, filters, iter);

        (iter, query_hash)
    }
}

// -------------------------------------------------------------------------------------------------

// Stream

impl Stream {
    fn iter_events(&self, from: Option<Position>) -> PersistentEventHashIterator {
        let iter = self.data.events.iterate(from);

        PersistentEventHashIterator::Direct(iter)
    }

    fn iter_indices(
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
    iter::{
        Iter,
        IterMulti,
    },
    options::Options,
};
