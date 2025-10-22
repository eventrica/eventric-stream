//! Utilities for set-like union combinations of streams (given sequential input
//! streams).

use derive_more::with_trait::Debug;
use fancy_constructor::new;
use std::cmp::Ordering;

use crate::iter::CachingIterators;

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
#[new(const_fn, vis())]
pub struct SequentialAnd<I, T>(CachingIterators<I, T>)
where
    I: Iterator<Item = T>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T> Iterator for SequentialAnd<I, T>
where
    I: Iterator<Item = T>,
    T: Copy + Debug + Ord + PartialOrd,
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
    T: Copy + Debug + Ord + PartialOrd,
{
    let iterators = iterators.into();
    let iterator = SequentialAnd::new(iterators);

    I::from(iterator)
}

// Iterate

#[allow(clippy::redundant_at_rest_pattern)]
fn iterate<I, T>(iterators: &mut CachingIterators<I, T>) -> Option<T>
where
    I: Iterator<Item = T>,
    T: Copy + Debug + Ord + PartialOrd,
{
    match &mut iterators.iterators[..] {
        [] => None,
        [iter] => iter.next(),
        [iters @ ..] => {
            let mut current = None;

            'a: loop {
                'b: for iter in iters.iter_mut() {
                    match (iter.next_cached()?, current) {
                        (iter_val, Some(current_val)) => {
                            let iter_val = match iterators.value {
                                Some(previous_val) if iter_val == previous_val => iter.next()?,
                                _ => iter_val,
                            };

                            match iter_val.cmp(&current_val) {
                                Ordering::Less => loop {
                                    let iter_val = iter.next()?;

                                    match iter_val.cmp(&current_val) {
                                        Ordering::Less => {}
                                        Ordering::Equal => continue 'b,
                                        Ordering::Greater => {
                                            current = Some(iter_val);
                                            continue 'a;
                                        }
                                    }
                                },
                                Ordering::Equal => {}
                                Ordering::Greater => {
                                    current = Some(iter_val);
                                    continue 'a;
                                }
                            }
                        }
                        (iter_val, _) => {
                            current = Some(match iterators.value {
                                Some(previous_val) if iter_val == previous_val => iter.next()?,
                                _ => iter_val,
                            });
                        }
                    }
                }

                break 'a;
            }

            iterators.value = current;
            iterators.value
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod test {
    use crate::iter::{and::sequential_and, test::TestIterator};

    #[test]
    fn empty_iterators_combine_as_empty() {
        let empty: Vec<u64> = vec![];

        let a = TestIterator::new([]);
        let b = TestIterator::new([]);

        assert_eq!(empty, sequential_and([a, b]).collect::<Vec<_>>());
    }

    #[test]
    fn iterators_combine_sequentially_same_lengths() {
        let combined = vec![2, 3];

        let a = TestIterator::new([0, 1, 2, 3]);
        let b = TestIterator::new([1, 2, 3, 4]);
        let c = TestIterator::new([2, 3, 4, 5]);

        assert_eq!(combined, sequential_and([a, b, c]).collect::<Vec<_>>());
    }

    #[test]
    fn iterators_combine_single_iterator() {
        let combined = vec![0, 1, 2, 3];

        let a = TestIterator::new([0, 1, 2, 3]);

        assert_eq!(combined, sequential_and([a]).collect::<Vec<_>>());
    }

    #[test]
    fn iterators_combine_sequentially_differing_lengths() {
        let combined = vec![2, 3];

        let a = TestIterator::new([2, 3]);
        let b = TestIterator::new([1, 2, 3, 4]);
        let c = TestIterator::new([2, 3, 4]);

        assert_eq!(combined, sequential_and([a, b, c]).collect::<Vec<_>>());

        let a = TestIterator::new([1, 2, 3, 4]);
        let b = TestIterator::new([2, 3]);
        let c = TestIterator::new([2, 3, 4]);

        assert_eq!(combined, sequential_and([a, b, c]).collect::<Vec<_>>());

        let a = TestIterator::new([2, 3, 4]);
        let b = TestIterator::new([2, 3]);
        let c = TestIterator::new([1, 2, 3, 4]);

        assert_eq!(combined, sequential_and([a, b, c]).collect::<Vec<_>>());
    }
}
