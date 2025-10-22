use std::cmp::Ordering;

use derive_more::with_trait::Debug;
use fancy_constructor::new;

// =================================================================================================
// Iteration
// =================================================================================================

// Iterator Cached

trait IteratorCached: Iterator {
    fn next_cached(&mut self) -> Option<<Self as Iterator>::Item>;
}

// -------------------------------------------------------------------------------------------------

// Caching Iterator(s)

/// The [`CachingIterators`] type implements a collection of [`CachingIterator`]
/// instances, and an optional value of the iterator item type. This is used as
/// common internal state for various iteration utility structures/functions.
#[derive(new, Debug)]
#[new(args(iterators: impl IntoIterator<Item = I>))]
struct CachingIterators<I, T>
where
    I: Iterator<Item = T>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[new(val(iterators.into_iter().map(CachingIterator::new).collect()))]
    iterators: Vec<CachingIterator<I, T>>,
    #[new(default)]
    value: Option<T>,
}

impl<S, I, T> From<S> for CachingIterators<I, T>
where
    S: IntoIterator<Item = I>,
    I: Iterator<Item = T>,
    T: Copy + Debug + Ord + PartialOrd,
{
    fn from(value: S) -> Self {
        CachingIterators::new(value)
    }
}

/// The [`CachingIterator`] wraps another iterator to provide some more
/// convenient semantics for other iteration utilities, particularly caching of
/// the most recently produced value, and functions for reading the cached value
/// (if it exists).
#[derive(new, Debug)]
#[new(vis())]
struct CachingIterator<I, T>
where
    I: Iterator<Item = T>,
    T: Copy + Debug + Ord + PartialOrd,
{
    iterator: I,
    #[new(default)]
    value: Option<T>,
}

impl<I, T> Iterator for CachingIterator<I, T>
where
    I: Iterator<Item = T>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.value = self.iterator.next();
        self.value
    }
}

impl<I, T> IteratorCached for CachingIterator<I, T>
where
    I: Iterator<Item = T>,
    T: Copy + Debug + Ord + PartialOrd,
{
    /// Returns the cached item value if it's not [`None`], otherwise returns
    /// (and caches) the value of [`Self::next`]. Note that if the underlying
    /// iterator returns [`None`], this will return cache and return that value.
    fn next_cached(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.value {
            Some(value) => Some(value),
            None => self.next(),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// And

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
pub struct SequentialAndIterator<I, T>(CachingIterators<I, T>)
where
    I: Iterator<Item = T>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T> SequentialAndIterator<I, T>
where
    I: Iterator<Item = T> + From<SequentialAndIterator<I, T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    pub fn combine<S>(iterators: S) -> I
    where
        S: IntoIterator<Item = I>,
    {
        let iterators = iterators.into();
        let iterator = SequentialAndIterator::new(iterators);

        I::from(iterator)
    }
}

impl<I, T> Iterator for SequentialAndIterator<I, T>
where
    I: Iterator<Item = T>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = T;

    #[allow(clippy::redundant_at_rest_pattern)]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0.iterators[..] {
            [] => None,
            [iter] => iter.next(),
            [iters @ ..] => {
                let mut current = None;

                'a: loop {
                    'b: for iter in iters.iter_mut() {
                        match (iter.next_cached()?, current) {
                            (iter_val, Some(current_val)) => {
                                let iter_val = match self.0.value {
                                    Some(previous_val) if iter_val == previous_val => {
                                        iter.next()?
                                    }
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
                                current = Some(match self.0.value {
                                    Some(previous_val) if iter_val == previous_val => {
                                        iter.next()?
                                    }
                                    _ => iter_val,
                                });
                            }
                        }
                    }

                    break 'a;
                }

                self.0.value = current;
                self.0.value
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Or

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
pub struct SequentialOrIterator<I, T>(CachingIterators<I, T>)
where
    I: Iterator<Item = T>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T> SequentialOrIterator<I, T>
where
    I: Iterator<Item = T> + From<SequentialOrIterator<I, T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    pub fn combine<S>(iterators: S) -> I
    where
        S: IntoIterator<Item = I>,
    {
        let iterators = iterators.into();
        let iterator = SequentialOrIterator::new(iterators);

        I::from(iterator)
    }
}

impl<I, T> Iterator for SequentialOrIterator<I, T>
where
    I: Iterator<Item = T>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = T;

    #[allow(clippy::redundant_at_rest_pattern)]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0.iterators[..] {
            [] => None,
            [iter] => iter.next(),
            [iters @ ..] => {
                let mut current = None;

                for iter in iters.iter_mut() {
                    match (iter.next_cached(), current) {
                        (Some(iter_val), Some(current_val)) => {
                            let iter_val = match self.0.value {
                                Some(previous_val) if iter_val == previous_val => iter.next(),
                                _ => Some(iter_val),
                            };

                            if let Some(iter_val) = iter_val {
                                match iter_val.cmp(&current_val) {
                                    Ordering::Less => current = Some(iter_val),
                                    Ordering::Equal | Ordering::Greater => {}
                                }
                            }
                        }
                        (Some(iter_val), None) => {
                            current = match self.0.value {
                                Some(previous_val) if iter_val == previous_val => iter.next(),
                                _ => Some(iter_val),
                            }
                        }
                        _ => {}
                    }
                }

                self.0.iterators.retain(|iter| iter.value.is_some());
                self.0.value = current;
                self.0.value
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod test {
    use derive_more::Debug;
    use fancy_constructor::new;

    use crate::util::iter::{
        SequentialAndIterator,
        SequentialOrIterator,
    };

    // Test Iterator

    #[derive(new, Debug)]
    pub enum TestIterator {
        And(SequentialAndIterator<TestIterator, u64>),
        Or(SequentialOrIterator<TestIterator, u64>),
        #[new]
        Vec(#[new(default)] usize, #[new(into)] Vec<u64>),
    }

    impl From<SequentialAndIterator<TestIterator, u64>> for TestIterator {
        fn from(value: SequentialAndIterator<TestIterator, u64>) -> Self {
            Self::And(value)
        }
    }

    impl From<SequentialOrIterator<TestIterator, u64>> for TestIterator {
        fn from(value: SequentialOrIterator<TestIterator, u64>) -> Self {
            Self::Or(value)
        }
    }

    impl From<Vec<u64>> for TestIterator {
        fn from(value: Vec<u64>) -> Self {
            Self::Vec(0, value)
        }
    }

    impl Iterator for TestIterator {
        type Item = u64;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Self::And(iterator) => iterator.next(),
                Self::Or(iterator) => iterator.next(),
                Self::Vec(pos, values) => {
                    if *pos >= values.len() {
                        None
                    } else {
                        *pos += 1;

                        Some(*values.get(*pos - 1).unwrap())
                    }
                }
            }
        }
    }

    // Test Iterator Tests

    #[test]
    fn test_iterator_returns_supplied_empty_vec() {
        let empty: Vec<u64> = vec![];

        assert_eq!(empty, TestIterator::new([]).collect::<Vec<_>>());
    }

    #[test]
    fn test_iterator_returns_supplied_vec() {
        let seq: Vec<u64> = vec![0, 1, 2, 3, 4, 5];

        assert_eq!(seq.clone(), TestIterator::new(seq).collect::<Vec<_>>());
    }

    // Sequential And Iterator Tests

    #[test]
    fn sequential_and_empty_iterators_combine_as_empty() {
        let empty: Vec<u64> = vec![];

        let a = TestIterator::new([]);
        let b = TestIterator::new([]);

        assert_eq!(
            empty,
            SequentialAndIterator::combine([a, b]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn sequential_and_iterators_combine_sequentially_same_lengths() {
        let combined = vec![2, 3];

        let a = TestIterator::new([0, 1, 2, 3]);
        let b = TestIterator::new([1, 2, 3, 4]);
        let c = TestIterator::new([2, 3, 4, 5]);

        assert_eq!(
            combined,
            SequentialAndIterator::combine([a, b, c]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn sequential_and_iterators_combine_single_iterator() {
        let combined = vec![0, 1, 2, 3];

        let a = TestIterator::new([0, 1, 2, 3]);

        assert_eq!(
            combined,
            SequentialAndIterator::combine([a]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn sequential_and_iterators_combine_sequentially_differing_lengths() {
        let combined = vec![2, 3];

        let a = TestIterator::new([2, 3]);
        let b = TestIterator::new([1, 2, 3, 4]);
        let c = TestIterator::new([2, 3, 4]);

        assert_eq!(
            combined,
            SequentialAndIterator::combine([a, b, c]).collect::<Vec<_>>()
        );

        let a = TestIterator::new([1, 2, 3, 4]);
        let b = TestIterator::new([2, 3]);
        let c = TestIterator::new([2, 3, 4]);

        assert_eq!(
            combined,
            SequentialAndIterator::combine([a, b, c]).collect::<Vec<_>>()
        );

        let a = TestIterator::new([2, 3, 4]);
        let b = TestIterator::new([2, 3]);
        let c = TestIterator::new([1, 2, 3, 4]);

        assert_eq!(
            combined,
            SequentialAndIterator::combine([a, b, c]).collect::<Vec<_>>()
        );
    }

    // Sequential Or Iterator Tests

    #[test]
    fn empty_iterators_combine_as_empty() {
        let empty: Vec<u64> = vec![];

        let a = TestIterator::new([]);
        let b = TestIterator::new([]);

        assert_eq!(
            empty,
            SequentialOrIterator::combine([a, b]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn iterators_combine_sequentially() {
        let combined = vec![0, 1, 2, 3, 4, 5];

        let a = TestIterator::new([0, 4]);
        let b = TestIterator::new([1, 5]);
        let c = TestIterator::new([2, 3]);

        assert_eq!(
            combined,
            SequentialOrIterator::combine([a, b, c]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn iterators_combine_sequentially_without_duplication() {
        let combined = vec![0, 1, 2, 3, 4, 5];

        let a = TestIterator::new([0, 3, 4]);
        let b = TestIterator::new([1, 2, 3]);
        let c = TestIterator::new([0, 1, 4, 5]);

        assert_eq!(
            combined,
            SequentialOrIterator::combine([a, b, c]).collect::<Vec<_>>()
        );
    }
}
