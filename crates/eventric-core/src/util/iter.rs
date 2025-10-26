use std::cmp::Ordering;

use derive_more::with_trait::Debug;
use fancy_constructor::new;

use crate::error::{
    Error,
    Result,
};

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
    I: Iterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[new(default)]
    errored: bool,
    #[new(val(iterators.into_iter().map(CachingIterator::new).collect()))]
    iterators: Vec<CachingIterator<I, T>>,
    #[new(default)]
    value: Option<T>,
}

impl<I, T> CachingIterators<I, T>
where
    I: Iterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[allow(clippy::unnecessary_wraps)]
    fn return_err(&mut self, err: Error) -> Option<Result<T>> {
        self.errored = true;
        self.iterators.clear();
        self.value = None;

        Some(Err(err))
    }

    #[allow(clippy::unnecessary_wraps)]
    fn return_ok_some(&mut self, value: T) -> Option<Result<T>> {
        self.errored = false;
        self.iterators.retain(|iter| iter.value.is_some());
        self.value = Some(value);

        Some(Ok(value))
    }

    fn return_ok_none(&mut self) -> Option<Result<T>> {
        self.errored = false;
        self.iterators.clear();
        self.value = None;

        None
    }
}

impl<S, I, T> From<S> for CachingIterators<I, T>
where
    S: IntoIterator<Item = I>,
    I: Iterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    fn from(iterators: S) -> Self {
        CachingIterators::new(iterators)
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
    I: Iterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[new(default)]
    errored: bool,
    iterator: I,
    #[new(default)]
    value: Option<T>,
}

impl<I, T> Iterator for CachingIterator<I, T>
where
    I: Iterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.errored {
            return None;
        }

        if let Some(result) = self.iterator.next() {
            match result {
                Ok(value) => {
                    self.value = Some(value);

                    Some(Ok(value))
                }
                Err(err) => {
                    self.errored = true;
                    self.value = None;

                    Some(Err(err))
                }
            }
        } else {
            self.value = None;

            None
        }
    }
}

impl<I, T> IteratorCached for CachingIterator<I, T>
where
    I: Iterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    /// Returns the cached item value if it's not [`None`], otherwise returns
    /// (and caches) the value of [`Self::next`]. Note that if the underlying
    /// iterator returns [`None`], this will return cache and return that value.
    fn next_cached(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.value {
            Some(value) => Some(Ok(value)),
            _ => self.next(),
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
    I: Iterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T> SequentialAndIterator<I, T>
where
    I: Iterator<Item = Result<T>> + From<SequentialAndIterator<I, T>>,
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
    I: Iterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T>;

    #[allow(clippy::redundant_at_rest_pattern)]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0.iterators[..] {
            [] => None,
            [iter] => iter.next(),
            [iters @ ..] => {
                let mut current: Option<T> = None;

                'a: loop {
                    'b: for iter in iters.iter_mut() {
                        match (iter.next_cached()?, current) {
                            (Ok(iter_val), Some(current_val)) => {
                                let iter_val = match self.0.value {
                                    Some(previous_val) if iter_val == previous_val => {
                                        iter.next()?
                                    }
                                    _ => Ok(iter_val),
                                };

                                match iter_val {
                                    Ok(iter_val) => match iter_val.cmp(&current_val) {
                                        Ordering::Less => loop {
                                            match iter.next()? {
                                                Ok(iter_val) => match iter_val.cmp(&current_val) {
                                                    Ordering::Less => {}
                                                    Ordering::Equal => continue 'b,
                                                    Ordering::Greater => {
                                                        current = Some(iter_val);
                                                        continue 'a;
                                                    }
                                                },
                                                Err(err) => return self.0.return_err(err),
                                            }
                                        },
                                        Ordering::Equal => {}
                                        Ordering::Greater => {
                                            current = Some(iter_val);
                                            continue 'a;
                                        }
                                    },
                                    Err(err) => return self.0.return_err(err),
                                }
                            }
                            (Ok(iter_val), _) => {
                                current = Some(match self.0.value {
                                    Some(previous_val) if iter_val == previous_val => {
                                        match iter.next()? {
                                            Ok(iter_val) => iter_val,
                                            Err(err) => return self.0.return_err(err),
                                        }
                                    }
                                    _ => iter_val,
                                });
                            }
                            (Err(err), _) => return self.0.return_err(err),
                        }
                    }

                    break 'a;
                }

                match current {
                    Some(value) => self.0.return_ok_some(value),
                    None => self.0.return_ok_none(),
                }
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
    I: Iterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T> SequentialOrIterator<I, T>
where
    I: Iterator<Item = Result<T>> + From<SequentialOrIterator<I, T>>,
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
    I: Iterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T>;

    #[allow(clippy::redundant_at_rest_pattern)]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0.iterators[..] {
            [] => None,
            [iter] => iter.next(),
            [iters @ ..] => {
                let mut current: Option<T> = None;

                for iter in iters.iter_mut() {
                    match (iter.next_cached(), current) {
                        (Some(Ok(iter_val)), Some(current_val)) => {
                            let iter_val = match self.0.value {
                                Some(previous_val) if iter_val == previous_val => iter.next(),
                                _ => Some(Ok(iter_val)),
                            };

                            match iter_val {
                                Some(Ok(iter_val)) => match iter_val.cmp(&current_val) {
                                    Ordering::Less => current = Some(iter_val),
                                    Ordering::Equal | Ordering::Greater => {}
                                },
                                Some(Err(err)) => return self.0.return_err(err),
                                _ => {}
                            }
                        }
                        (Some(Ok(iter_val)), None) => match self.0.value {
                            Some(previous_val) if iter_val == previous_val => match iter.next() {
                                Some(Ok(iter_val)) => current = Some(iter_val),
                                Some(Err(err)) => return self.0.return_err(err),
                                None => current = None,
                            },
                            _ => current = Some(iter_val),
                        },
                        (Some(Err(err)), _) => return self.0.return_err(err),
                        _ => {}
                    }
                }

                match current {
                    Some(value) => self.0.return_ok_some(value),
                    None => self.0.return_ok_none(),
                }
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

    use crate::{
        error::{
            Error,
            Result,
        },
        util::iter::{
            SequentialAndIterator,
            SequentialOrIterator,
        },
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
        fn from(iter: SequentialAndIterator<TestIterator, u64>) -> Self {
            Self::And(iter)
        }
    }

    impl From<SequentialOrIterator<TestIterator, u64>> for TestIterator {
        fn from(iter: SequentialOrIterator<TestIterator, u64>) -> Self {
            Self::Or(iter)
        }
    }

    impl From<Vec<u64>> for TestIterator {
        fn from(vec: Vec<u64>) -> Self {
            Self::Vec(0, vec)
        }
    }

    impl Iterator for TestIterator {
        type Item = Result<u64>;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Self::And(iterator) => iterator.next(),
                Self::Or(iterator) => iterator.next(),
                Self::Vec(pos, values) => {
                    if *pos >= values.len() {
                        None
                    } else {
                        *pos += 1;

                        Some(Ok(*values.get(*pos - 1).unwrap()))
                    }
                }
            }
        }
    }

    // Test Iterator Tests

    #[test]
    fn test_iterator_returns_supplied_empty_vec() {
        assert_eq!(
            Vec::<Result<u64>>::new(),
            TestIterator::new([]).collect::<Vec<_>>()
        );
    }

    #[rustfmt::skip]
    #[test]
    fn test_iterator_returns_supplied_vec() {
        let input = Vec::from_iter([0, 1, 2, 3, 4, 5]);

        assert_eq!(
            input.clone().into_iter().map(Ok::<u64, Error>).collect::<Vec<_>>(),
            TestIterator::new(input).collect::<Vec<_>>()
        );
    }

    // Sequential And Iterator Tests

    #[test]
    fn sequential_and_empty_iterators_combine_as_empty() {
        let empty: Vec<Result<u64>> = Vec::from_iter([]);

        let a = TestIterator::new([]);
        let b = TestIterator::new([]);

        assert_eq!(
            empty,
            SequentialAndIterator::combine([a, b]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn sequential_and_iterators_combine_sequentially_same_lengths() {
        let combined = Vec::from_iter([Ok(2), Ok(3)]);

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
        let combined = Vec::from_iter([Ok(0), Ok(1), Ok(2), Ok(3)]);

        let a = TestIterator::new([0, 1, 2, 3]);

        assert_eq!(
            combined,
            SequentialAndIterator::combine([a]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn sequential_and_iterators_combine_sequentially_differing_lengths() {
        let combined = Vec::from_iter([Ok(2), Ok(3)]);

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
        let empty: Vec<Result<u64>> = Vec::new();

        let a = TestIterator::new([]);
        let b = TestIterator::new([]);

        assert_eq!(
            empty,
            SequentialOrIterator::combine([a, b]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn iterators_combine_sequentially() {
        let combined = Vec::from_iter([Ok(0), Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)]);

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
        let combined = Vec::from_iter([Ok(0), Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)]);

        let a = TestIterator::new([0, 3, 4]);
        let b = TestIterator::new([1, 2, 3]);
        let c = TestIterator::new([0, 1, 4, 5]);

        assert_eq!(
            combined,
            SequentialOrIterator::combine([a, b, c]).collect::<Vec<_>>()
        );
    }
}
