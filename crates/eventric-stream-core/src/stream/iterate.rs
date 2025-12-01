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
        Stream,
        data::events::{
            EventHashIter,
            MappedEventHashIter,
        },
        iterate::{
            build::Build,
            cache::Cache,
        },
        select::{
            SelectionHash,
            source::Source,
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
    fn iter(&self, from: Option<Position>) -> Iter<()>;
}

impl Iterate for Stream {
    fn iter(&self, from: Option<Position>) -> Iter<()> {
        let cache = Arc::new(Cache::default());
        let references = self.data.references.clone();

        let iter = self.iterate_events(from);
        let iter = Exclusive::new(iter);

        Iter::<()>::new(cache, true, references, (), iter)
    }
}

// -------------------------------------------------------------------------------------------------

// Iterate Select

/// The [`IterateSelect`] trait defines the logical operation of iterating over
/// a stream or stream-like type, using a supplied [`Selection`]) to determine
/// which matching events should be returned, and an optional [`Position`] at
/// which iteration should begin.
#[allow(private_bounds)]
pub trait IterateSelect {
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
    fn iter_select<S>(&self, source: S, from: Option<Position>) -> (S::Iterator, S::Prepared)
    where
        S: Source,
        S::Iterator: Build<S::Prepared>;
}

#[allow(private_bounds)]
impl IterateSelect for Stream {
    fn iter_select<S>(&self, source: S, from: Option<Position>) -> (S::Iterator, S::Prepared)
    where
        S: Source,
        S::Iterator: Build<S::Prepared>,
    {
        let references = self.data.references.clone();
        let prepared = source.prepare();

        let iter = self.iterate_indices(prepared.as_ref(), from);
        let iter = S::Iterator::build(iter, &prepared, references);

        (iter, prepared)
    }
}

// -------------------------------------------------------------------------------------------------

// Stream

impl Stream {
    fn iterate_events(&self, from: Option<Position>) -> EventHashIter {
        let iter = self.data.events.iterate(from);

        EventHashIter::Direct(iter)
    }

    fn iterate_indices(&self, selection: &SelectionHash, from: Option<Position>) -> EventHashIter {
        let events = self.data.events.clone();

        let iter = self.data.indices.iterate(selection, from);
        let iter = MappedEventHashIter::new(events, iter);

        EventHashIter::Mapped(iter)
    }
}

// -------------------------------------------------------------------------------------------------

// Re-Export

pub use self::iter::Iter;
