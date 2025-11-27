//! See the `eventric-stream` crate for full documentation, including
//! module-level documentation.

pub(crate) mod build;
pub(crate) mod cache;
pub(crate) mod iter;

use std::sync::{
    Arc,
    Exclusive,
};

use crate::{
    event::position::Position,
    stream::{
        Single,
        Stream,
        data::events::{
            MappedPersistentEventHashIterator,
            PersistentEventHashIterator,
        },
        iterate::cache::Cache,
        query::{
            QueryHash,
            Source,
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
    fn iterate(&self, from: Option<Position>) -> Iter<Single>;
}

impl Iterate for Stream {
    fn iterate(&self, from: Option<Position>) -> Iter<Single> {
        let cache = Arc::new(Cache::default());
        let references = self.data.references.clone();

        let iter = self.iter_events(from);
        let iter = Exclusive::new(iter);

        Iter::<Single>::new(cache, true, references, (), iter)
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
    fn iterate_query<Q>(&self, query: Q, from: Option<Position>) -> (Q::Iterator, Q::Prepared)
    where
        Q: Source,
        Q::Iterator: Build<Q::Prepared>;
}

impl IterateQuery for Stream {
    fn iterate_query<Q>(&self, query: Q, from: Option<Position>) -> (Q::Iterator, Q::Prepared)
    where
        Q: Source,
        Q::Iterator: Build<Q::Prepared>,
    {
        let references = self.data.references.clone();
        let optimized = query.prepare();

        let iter = self.iter_indices(optimized.as_ref(), from);
        let iter = Q::Iterator::build(&optimized, iter, references);

        (iter, optimized)
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
    build::Build,
    iter::Iter,
};
