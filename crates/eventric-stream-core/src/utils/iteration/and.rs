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
    use crate::{
        error::Error,
        utils::iteration::{
            and::SequentialAndIterator,
            tests::TestIterator,
        },
    };

    #[test]
    fn impl_iterator() {
        // Empty
        let a = TestIterator::from([]);
        let b = TestIterator::from([]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        assert_eq!(None, iter.next());

        // No overlap - disjoint sets
        let a = TestIterator::from([Ok(0), Ok(1), Ok(2)]);
        let b = TestIterator::from([Ok(3), Ok(4), Ok(5)]);
        let c = TestIterator::from([Ok(6), Ok(7), Ok(8)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(None, iter.next());

        // Partial overlap - only some values in all iterators
        let a = TestIterator::from([Ok(0), Ok(2), Ok(4), Ok(6)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(4), Ok(5)]);
        let c = TestIterator::from([Ok(1), Ok(2), Ok(4), Ok(7)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(None, iter.next());

        // Complete overlap - all iterators have same values
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let c = TestIterator::from([Ok(1), Ok(2), Ok(3)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());

        // Variable lengths with overlap
        let a = TestIterator::from([Ok(0), Ok(3), Ok(4), Ok(5)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(4)]);
        let c = TestIterator::from([Ok(0), Ok(3), Ok(4), Ok(5), Ok(6)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(None, iter.next());

        // Single iterator
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);

        let mut iter = SequentialAndIterator::combine([a]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());

        // Two iterators with one empty
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        assert_eq!(None, iter.next());

        // Complex overlap pattern
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(5), Ok(7), Ok(9)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(4), Ok(5), Ok(6), Ok(7)]);
        let c = TestIterator::from([Ok(3), Ok(5), Ok(7), Ok(8)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next());
        assert_eq!(Some(Ok(7)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn impl_double_ended_iterator() {
        // Empty
        let a = TestIterator::from([]);
        let b = TestIterator::from([]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        assert_eq!(None, iter.next_back());

        // No overlap - disjoint sets
        let a = TestIterator::from([Ok(0), Ok(1), Ok(2)]);
        let b = TestIterator::from([Ok(3), Ok(4), Ok(5)]);
        let c = TestIterator::from([Ok(6), Ok(7), Ok(8)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(None, iter.next_back());

        // Partial overlap - only some values in all iterators
        let a = TestIterator::from([Ok(0), Ok(2), Ok(4), Ok(6)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(4), Ok(5)]);
        let c = TestIterator::from([Ok(1), Ok(2), Ok(4), Ok(7)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Complete overlap - all iterators have same values
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let c = TestIterator::from([Ok(1), Ok(2), Ok(3)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Variable lengths with overlap
        let a = TestIterator::from([Ok(0), Ok(3), Ok(4), Ok(5)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(4)]);
        let c = TestIterator::from([Ok(0), Ok(3), Ok(4), Ok(5), Ok(6)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Single iterator
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);

        let mut iter = SequentialAndIterator::combine([a]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Two iterators with one empty
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        assert_eq!(None, iter.next_back());

        // Complex overlap pattern
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(5), Ok(7), Ok(9)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(4), Ok(5), Ok(6), Ok(7)]);
        let c = TestIterator::from([Ok(3), Ok(5), Ok(7), Ok(8)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(7)), iter.next_back());
        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(None, iter.next_back());
    }

    #[test]
    fn impl_iterator_and_double_ended_iterator() {
        // Partial overlap - alternating next and next_back
        let a = TestIterator::from([Ok(0), Ok(2), Ok(4), Ok(6), Ok(8)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(4), Ok(5), Ok(6)]);
        let c = TestIterator::from([Ok(1), Ok(2), Ok(4), Ok(6), Ok(7)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(6)), iter.next_back());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(None, iter.next());

        // Complete overlap - mixed iteration
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)]);
        let c = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());

        // Variable lengths - mixed iteration
        let a = TestIterator::from([Ok(0), Ok(3), Ok(4), Ok(5), Ok(8)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)]);
        let c = TestIterator::from([Ok(0), Ok(3), Ok(4), Ok(5), Ok(6)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(None, iter.next());

        // Complex pattern - ensure correctness
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(5), Ok(7), Ok(9), Ok(11)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(4), Ok(5), Ok(6), Ok(7), Ok(10)]);
        let c = TestIterator::from([Ok(3), Ok(5), Ok(7), Ok(8), Ok(9)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(7)), iter.next_back());
        assert_eq!(Some(Ok(5)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn impl_iterator_with_errors() {
        // Error in first iterator - should return error immediately
        let a = TestIterator::from([Ok(1), Err(Error::Concurrency), Ok(3)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3)]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));

        // Error in second iterator
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([Ok(1), Err(Error::Concurrency), Ok(3)]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));

        // Error in third iterator
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let c = TestIterator::from([Ok(1), Err(Error::Concurrency), Ok(3)]);

        let mut iter = SequentialAndIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));

        // Multiple errors - should return first encountered
        let a = TestIterator::from([Ok(1), Err(Error::Concurrency)]);
        let b = TestIterator::from([Ok(1), Err(Error::Concurrency)]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));
    }

    #[test]
    fn impl_double_ended_iterator_with_errors() {
        // Error at end of first iterator
        let a = TestIterator::from([Ok(1), Ok(2), Err(Error::Concurrency)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3)]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        assert!(matches!(iter.next_back(), Some(Err(Error::Concurrency))));

        // Error at end of second iterator
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([Ok(1), Ok(2), Err(Error::Concurrency)]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        assert!(matches!(iter.next_back(), Some(Err(Error::Concurrency))));

        // Error in middle - mixed iteration
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(4)]);
        let b = TestIterator::from([Ok(1), Err(Error::Concurrency), Ok(3), Ok(4)]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next());
        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));
    }
}
