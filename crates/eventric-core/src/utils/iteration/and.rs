//! The [`and`][and] module provides an iterator which provides the boolean AND
//! operation over a collection of sequential iterators, such that an item will
//! only appear in the output if it occurs in all of the input iterators.
//!
//! [and]: self]

use std::cmp::Ordering;

use derive_more::with_trait::Debug;
use double_ended_peekable::{
    DoubleEndedPeekable,
    DoubleEndedPeekableExt,
};
use fancy_constructor::new;

use crate::error::Error;

// =================================================================================================
// And
// =================================================================================================

/// The [`SequentialAndIterator`] type represents an iterator over the combined
/// values of a set of sequential iterators The resulting iterator is equivalent
/// to an ordered intersection (∩) of the underlying iterators (i.e. values
/// appear only once, and are totally ordered).
///
/// See local unit tests for simple examples.
#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct SequentialAndIterator<I, T>(Vec<DoubleEndedPeekable<I>>)
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T> SequentialAndIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>> + From<SequentialAndIterator<I, T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    /// Take a an iterable value of iterators, and return an iterator of the
    /// same type which will implement the boolean AND operation on the input
    /// iterators.
    pub fn combine<S>(iters: S) -> I
    where
        S: IntoIterator<Item = I>,
    {
        let iters = iters
            .into_iter()
            .map(DoubleEndedPeekableExt::double_ended_peekable)
            .collect();

        I::from(SequentialAndIterator::new(iters))
    }
}

impl<I, T> DoubleEndedIterator for SequentialAndIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut current = None;

        'seek: loop {
            for iter in &mut self.0 {
                match iter.peek_back() {
                    Some(Ok(next)) => match &mut current {
                        Some(current) => {
                            match next.cmp(current) {
                                Ordering::Greater => drop(iter.next_back()),
                                Ordering::Less => *current = *next,
                                Ordering::Equal => continue,
                            }

                            continue 'seek;
                        }
                        None => current = Some(*next),
                    },
                    Some(Err(_)) => return iter.next_back(),
                    None => return None,
                }
            }

            break 'seek;
        }

        current.map(Ok).inspect(|item| {
            for iter in &mut self.0 {
                iter.next_back_if_eq(item);
            }
        })
    }
}

impl<I, T> Iterator for SequentialAndIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T, Error>;

    #[allow(clippy::redundant_at_rest_pattern)]
    fn next(&mut self) -> Option<Self::Item> {
        let mut current = None;

        'seek: loop {
            for iter in &mut self.0 {
                match iter.peek() {
                    Some(Ok(next)) => match &mut current {
                        Some(current) => {
                            match next.cmp(current) {
                                Ordering::Greater => *current = *next,
                                Ordering::Less => drop(iter.next()),
                                Ordering::Equal => continue,
                            }

                            continue 'seek;
                        }
                        None => current = Some(*next),
                    },
                    Some(Err(_)) => return iter.next(),
                    None => return None,
                }
            }

            break 'seek;
        }

        current.map(Ok).inspect(|item| {
            for iter in &mut self.0 {
                iter.next_if_eq(item);
            }
        })
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use crate::utils::iteration::{
        and::SequentialAndIterator,
        tests::TestIterator,
    };

    #[test]
    fn sequential_and_impl_iterator() {
        // Empty

        let a = TestIterator::from([]);
        let b = TestIterator::from([]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        assert_eq!(None, iter.next());

        let a = TestIterator::from([0, 1, 2, 3]);
        let b = TestIterator::from([]);
        let c = TestIterator::from([2, 3, 4, 5]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(None, iter.next());

        // Single

        let a = TestIterator::from([0, 1, 2, 3]);

        let mut iter = SequentialAndIterator::combine([a]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());

        // Matched Lengths

        let a = TestIterator::from([0, 1, 2, 3]);
        let b = TestIterator::from([1, 2, 3, 4]);
        let c = TestIterator::from([2, 3, 4, 5]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());

        // Variable Lengths

        let a = TestIterator::from([2, 3]);
        let b = TestIterator::from([1, 2, 3, 4]);
        let c = TestIterator::from([2, 3, 4]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());

        let a = TestIterator::from([1, 2, 3, 4]);
        let b = TestIterator::from([2, 3]);
        let c = TestIterator::from([2, 3, 4]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());

        let a = TestIterator::from([2, 3, 4]);
        let b = TestIterator::from([2, 3]);
        let c = TestIterator::from([1, 2, 3, 4]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn sequential_and_impl_double_ended_iterator() {
        // Empty

        let a = TestIterator::from([]);
        let b = TestIterator::from([]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        assert_eq!(None, iter.next_back());

        let a = TestIterator::from([0, 1, 2, 3]);
        let b = TestIterator::from([]);
        let c = TestIterator::from([2, 3, 4, 5]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(None, iter.next_back());

        // Single

        let a = TestIterator::from([0, 1, 2, 3]);

        let mut iter = SequentialAndIterator::combine([a]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(Some(Ok(0)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Matched Lengths

        let a = TestIterator::from([0, 1, 2, 3]);
        let b = TestIterator::from([1, 2, 3, 4]);
        let c = TestIterator::from([2, 3, 4, 5]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Variable Lengths

        let a = TestIterator::from([2, 3]);
        let b = TestIterator::from([1, 2, 3, 4]);
        let c = TestIterator::from([2, 3, 4]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(None, iter.next_back());

        let a = TestIterator::from([1, 2, 3, 4]);
        let b = TestIterator::from([2, 3]);
        let c = TestIterator::from([2, 3, 4]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(None, iter.next_back());

        let a = TestIterator::from([2, 3, 4]);
        let b = TestIterator::from([2, 3]);
        let c = TestIterator::from([1, 2, 3, 4]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(None, iter.next_back());
    }
}
