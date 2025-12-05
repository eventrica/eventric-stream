//! See the `eventric-stream` crate for full documentation, including
//! module-level documentation.

pub(crate) mod cache;
pub(crate) mod iter;

use std::sync::Arc;

use crate::{
    event::position::Position,
    stream::{
        data::{
            Data,
            events::{
                EventHashIter,
                MappedEventHashIter,
            },
        },
        iterate::cache::Cache,
        select::{
            Prepared,
            prepared::MultiPrepared,
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
    fn iter(&self, from: Option<Position>) -> Iter;

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
    fn iter_select<S>(&self, selection: S, from: Option<Position>) -> (IterSelect, Prepared)
    where
        S: Into<Prepared>;

    /// .
    fn iter_multi_select<S>(
        &self,
        selections: S,
        from: Option<Position>,
    ) -> (IterMultiSelect, MultiPrepared)
    where
        S: Into<MultiPrepared>;
}

// Implementations

pub(crate) fn iter(data: &Data, from: Option<Position>) -> Iter {
    let cache = Arc::new(Cache::default());
    let references = data.references.clone();

    let iter = data.events.iterate(from);
    let iter = EventHashIter::Direct(iter);

    Iter::new(cache, iter, references)
}

pub(crate) fn iter_select<S>(
    data: &Data,
    selection: S,
    from: Option<Position>,
) -> (IterSelect, Prepared)
where
    S: Into<Prepared>,
{
    let events = data.events.clone();
    let references = data.references.clone();

    let prepared = selection.into();

    let iter = data.indices.iterate(prepared.as_ref(), from);
    let iter = MappedEventHashIter::new(events, iter);
    let iter = EventHashIter::Mapped(iter);
    let iter = IterSelect::new(iter, &prepared, references);

    (iter, prepared)
}

pub(crate) fn iter_multi_select<S>(
    data: &Data,
    selection: S,
    from: Option<Position>,
) -> (IterMultiSelect, MultiPrepared)
where
    S: Into<MultiPrepared>,
{
    let events = data.events.clone();
    let references = data.references.clone();

    let prepared = selection.into();

    let iter = data.indices.iterate(prepared.as_ref(), from);
    let iter = MappedEventHashIter::new(events, iter);
    let iter = EventHashIter::Mapped(iter);
    let iter = IterMultiSelect::new(iter, &prepared, references);

    (iter, prepared)
}

// -------------------------------------------------------------------------------------------------

// Re-Export

pub use self::iter::{
    Iter,
    IterMultiSelect,
    IterSelect,
};
