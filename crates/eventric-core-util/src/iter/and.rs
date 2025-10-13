//! Utilities for set-like union combinations of streams (given sequential input
//! streams).

use derive_more::Debug;
use fancy_constructor::new;

use crate::iter::{
    CachingIterator,
    CachingIterators,
    Item,
};

// =================================================================================================
// And
// =================================================================================================

/// The [`SequentialAnd`] type represents an iterator over the combined values
/// of a set of sequential iterators (such as the
/// [`SequentialIterator`][seq_int] type found in the [`index`][index] module).
/// The resulting iterator is equivalent to an ordered intersection (∩) of the
/// underlying iterators (i.e. values appear only once, and are totally
/// ordered).
///
/// See local unit tests for simple examples.
///
/// [index]: crate::persistence::index
/// [seq_int]: crate::persistence::index::SequentialIterator
#[derive(new, Debug)]
#[new(vis())]
pub struct SequentialAnd<I, T>(CachingIterators<I, T>)
where
    I: Iterator<Item = T>,
    T: Item;

impl<I, T> Iterator for SequentialAnd<I, T>
where
    I: Iterator<Item = T>,
    T: Item,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        iterate(&mut self.0)
    }
}

// Iterator

/// The [`sequential_and`] function creates an iterator of the same type (`I`)
/// as the parameter of the `iterators` argument, given an input iterator type
/// `I` implementing [`From<SequentialAnd>`]. The behaviour of the iterator is
/// described in [`SequentialAnd`] - an ordered intersection of values from the
/// input iterators.
pub fn sequential_and<IS, I, T>(iterators: IS) -> I
where
    IS: IntoIterator<Item = I>,
    I: Iterator<Item = T> + From<SequentialAnd<I, T>>,
    T: Item,
{
    let iterators = iterators.into();
    let iterator = SequentialAnd::new(iterators);

    I::from(iterator)
}

// Iterate

fn iterate<I, T>(iterators: &mut CachingIterators<I, T>) -> Option<T>
where
    I: Iterator<Item = T>,
    T: Item,
{
    fn seek<I, T>(iter: &mut CachingIterator<I, T>, value: T) -> Option<bool>
    where
        I: Iterator<Item = T>,
        T: Item,
    {
        match iter.next_cached() {
            Some(iter_value) if iter_value > value => Some(false),
            Some(iter_value) if iter_value == value => Some(true),
            Some(_) => seek(iter.next_self_ref(), value),
            None => None,
        }
    }

    let mut new: Option<T> = None;

    'seek_match: loop {
        let mut matched = false;

        for iter in &mut iterators.iterators {
            match new {
                Some(new_value) => match seek(iter, new_value) {
                    Some(new_matched) => matched = new_matched,
                    None => break 'seek_match,
                },
                None => match iter.next() {
                    Some(iter_value) => new = Some(iter_value),
                    None => break 'seek_match,
                },
            }
        }

        if matched {
            break 'seek_match;
        }

        new = None;
    }

    iterators.value = new;
    iterators.value
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod test {
    use crate::iter::{
        and::sequential_and,
        test::TestIterator,
    };

    #[test]
    fn empty_iterators_combine_as_empty() {
        let empty: Vec<u64> = vec![];

        let a = TestIterator::new([]);
        let b = TestIterator::new([]);

        assert_eq!(empty, sequential_and([a, b]).collect::<Vec<_>>());
    }

    #[test]
    fn iterators_combine_sequentially() {
        let combined = vec![2, 3];

        let a = TestIterator::new([0, 1, 2, 3]);
        let b = TestIterator::new([1, 2, 3, 4]);
        let c = TestIterator::new([2, 3, 4, 5]);

        assert_eq!(combined, sequential_and([a, b, c]).collect::<Vec<_>>());
    }
}
