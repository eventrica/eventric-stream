//! Utilities for set-like intersection combinations of streams (given
//! sequential input streams).

use derive_more::Debug;
use fancy_constructor::new;

use crate::iter::{
    CachingIterators,
    Item,
};

// =================================================================================================
// Or
// =================================================================================================

/// The [`SequentialOr`] type represents an iterator over the combined values of
/// a set of sequential iterators (such as the [`SequentialIterator`][seq_int]
/// type found in the [`index`][index] module). The resulting iterator is
/// equivalent to an ordered union (∪) of the underlying iterators (i.e. values
/// appear only once, and are totally ordered).
///
/// See local unit tests for simple examples.
///
/// [index]: crate::persistence::index
/// [seq_int]: crate::persistence::index::SequentialIterator
#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct SequentialOr<I, T>(CachingIterators<I, T>)
where
    I: Iterator<Item = T>,
    T: Item;

impl<I, T> Iterator for SequentialOr<I, T>
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

/// The [`sequential_or`] function creates an iterator of the same type (`I`)
/// as the parameter of the `iterators` argument, given an input iterator type
/// `I` implementing [`From<SequentialOr>`]. The behaviour of the iterator is
/// described in [`SequentialOr`] - an ordered union of values from the input
/// iterators.
pub fn sequential_or<S, I, T>(iterators: S) -> I
where
    S: IntoIterator<Item = I>,
    I: Iterator<Item = T> + From<SequentialOr<I, T>>,
    T: Item,
{
    let iterators = iterators.into();
    let iterator = SequentialOr::new(iterators);

    I::from(iterator)
}

// Iterate

fn iterate<I, T>(iterators: &mut CachingIterators<I, T>) -> Option<T>
where
    I: Iterator<Item = T>,
    T: Item,
{
    fn update<T>(new: &mut Option<T>, value: Option<T>)
    where
        T: Item,
    {
        match (new.as_mut(), value) {
            (Some(new), Some(value)) if value < *new => *new = value,
            (None, Some(value)) => *new = Some(value),
            _ => {}
        }
    }

    let mut new = None;

    if let Some(iterators_value) = iterators.value {
        for iter in &mut iterators.iterators {
            match iter.next_cached() {
                Some(iter_value) if iter_value <= iterators_value => update(&mut new, iter.next()),
                Some(_) => update(&mut new, iter.next_cached()),
                _ => {}
            }
        }
    } else {
        for iter in &mut iterators.iterators {
            update(&mut new, iter.next());
        }
    }

    iterators.iterators.retain(|iter| iter.value.is_some());
    iterators.value = new;
    iterators.value
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod test {
    use crate::iter::{
        or::sequential_or,
        test::TestIterator,
    };

    #[test]
    fn empty_iterators_combine_as_empty() {
        let empty: Vec<u64> = vec![];

        let a = TestIterator::new([]);
        let b = TestIterator::new([]);

        assert_eq!(empty, sequential_or([a, b]).collect::<Vec<_>>());
    }

    #[test]
    fn iterators_combine_sequentially() {
        let combined = vec![0, 1, 2, 3, 4, 5];

        let a = TestIterator::new([0, 4]);
        let b = TestIterator::new([1, 5]);
        let c = TestIterator::new([2, 3]);

        assert_eq!(combined, sequential_or([a, b, c]).collect::<Vec<_>>());
    }

    #[test]
    fn iterators_combine_sequentially_without_duplication() {
        let combined = vec![0, 1, 2, 3, 4, 5];

        let a = TestIterator::new([0, 2, 3, 4]);
        let b = TestIterator::new([1, 2, 3, 5]);
        let c = TestIterator::new([0, 1, 4, 5]);

        assert_eq!(combined, sequential_or([a, b, c]).collect::<Vec<_>>());
    }
}
