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
        data::{
            Data,
            events::{
                EventHashIter,
                MappedEventHashIter,
            },
        },
        iterate::{
            build::Build,
            cache::Cache,
        },
        select::{
            Prepared,
            Selection,
            Selections,
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
    fn iter_select<S>(
        &self,
        selection: S,
        from: Option<Position>,
    ) -> (Iter<Selection>, Prepared<Selection>)
    where
        S: Into<Prepared<Selection>>;

    /// .
    fn iter_select_multi<S>(
        &self,
        selections: S,
        from: Option<Position>,
    ) -> (Iter<Selections>, Prepared<Selections>)
    where
        S: Into<Prepared<Selections>>;
}

// Implementations

pub(crate) fn iter(data: &Data, from: Option<Position>) -> Iter<()> {
    let cache = Arc::new(Cache::default());
    let references = data.references.clone();

    let iter = data.events.iterate(from);
    let iter = EventHashIter::Direct(iter);
    let iter = Exclusive::new(iter);

    Iter::<()>::new(cache, true, references, (), iter)
}

pub(crate) fn iter_select<S>(
    data: &Data,
    selection: S,
    from: Option<Position>,
) -> (Iter<Selection>, Prepared<Selection>)
where
    S: Into<Prepared<Selection>>,
{
    let events = data.events.clone();
    let references = data.references.clone();

    let prepared = selection.into();

    let iter = data.indices.iterate(prepared.as_ref(), from);
    let iter = MappedEventHashIter::new(events, iter);
    let iter = EventHashIter::Mapped(iter);
    let iter = Iter::<Selection>::build(iter, &prepared, references);

    (iter, prepared)
}

pub(crate) fn iter_select_multi<S>(
    data: &Data,
    selection: S,
    from: Option<Position>,
) -> (Iter<Selections>, Prepared<Selections>)
where
    S: Into<Prepared<Selections>>,
{
    let events = data.events.clone();
    let references = data.references.clone();

    let prepared = selection.into();

    let iter = data.indices.iterate(prepared.as_ref(), from);
    let iter = MappedEventHashIter::new(events, iter);
    let iter = EventHashIter::Mapped(iter);
    let iter = Iter::<Selections>::build(iter, &prepared, references);

    (iter, prepared)
}

// -------------------------------------------------------------------------------------------------

// Re-Export

pub use self::iter::Iter;
