//! The [`and`][and] module provides an iterator which provides the boolean AND
//! operation over a collection of sequential iterators, such that an item will
//! only appear in the output if it occurs in all of the input iterators.
//!
//! [and]: self]

use std::{
    cmp::Ordering,
    iter::FusedIterator,
};

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

/// Macro to implement `next()` or `next_back()` for [`SequentialAndIterator`].
///
/// The AND iterator uses a convergence algorithm where it seeks a value that
/// exists in all input iterators.
macro_rules! impl_and_next {
    ($peek:ident, $next:ident, $next_if_eq:ident, $update:path, $advance:path) => {
        #[inline]
        fn $next(&mut self) -> Option<Self::Item> {
            if self.0.is_empty() {
                return None;
            }

            let mut candidate = None;

            loop {
                let mut converged = true;

                for iter in &mut self.0 {
                    match iter.$peek() {
                        Some(Ok(next)) => match &mut candidate {
                            Some(current_candidate) => match next.cmp(current_candidate) {
                                $update => {
                                    *current_candidate = *next;
                                    converged = false;
                                }
                                $advance => {
                                    iter.$next();
                                    converged = false;
                                }
                                Ordering::Equal => {}
                            },
                            None => candidate = Some(*next),
                        },
                        _ => return iter.$next(),
                    }
                }

                if converged {
                    break;
                }
            }

            candidate.map(Ok).inspect(|item| {
                for iter in &mut self.0 {
                    iter.$next_if_eq(item);
                }
            })
        }
    };
}

/// The [`SequentialAndIterator`] implements a sorted set intersection.
///
/// This iterator type represents an iterator over the combined values of a set
/// of sequential iterators. The resulting iterator is equivalent to an ordered
/// intersection (∩) of the underlying iterators (i.e. values appear only once,
/// and are totally ordered).
///
/// # Algorithm
///
/// The iterator maintains a "candidate" value and advances through all input
/// iterators simultaneously. For each iteration:
/// - If an iterator has a value less than the candidate, advance that iterator
/// - If an iterator has a value greater than the candidate, update the
///   candidate
/// - When all iterators converge on the same value, that value is returned
///
/// This ensures O(n*m) complexity where n is the number of iterators and m is
/// the average number of elements per iterator.
///
/// # Requirements
///
/// Input iterators **MUST** be sorted in ascending order. Behavior is undefined
/// if this precondition is not met.
///
/// # Error Handling
///
/// Errors from any underlying iterator are propagated immediately. When an
/// error is encountered, iteration stops and the error is returned. The
/// iterator state after an error is unspecified - callers should not continue
/// iterating after receiving an error.
///
/// # Examples
///
/// ```ignore
/// let a = vec![Ok(1), Ok(3), Ok(5)];
/// let b = vec![Ok(3), Ok(5), Ok(7)];
///
/// let result: Vec<_> = SequentialAndIterator::combine([a.into_iter(), b.into_iter()])
///     .collect::<Result<Vec<_>, _>>()
///     .unwrap();
///
/// assert_eq!(result, vec![3, 5]); // Only values in both iterators
/// ```
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
    #[rustfmt::skip]
    impl_and_next!(peek_back, next_back, next_back_if_eq, Ordering::Less, Ordering::Greater);
}

impl<I, T> Iterator for SequentialAndIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T, Error>;

    #[rustfmt::skip]
    impl_and_next!(peek, next, next_if_eq, Ordering::Greater, Ordering::Less);

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.0.is_empty() {
            return (0, Some(0));
        }

        // Lower bound is 0 (might be no intersection)
        // Upper bound is the minimum of all iterator upper bounds
        let upper = self.0.iter().filter_map(|iter| iter.size_hint().1).min();

        (0, upper)
    }
}

impl<I, T> FusedIterator for SequentialAndIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>> + FusedIterator,
    T: Copy + Debug + Ord + PartialOrd,
{
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

    #[test]
    fn size_hint_empty_iterators() {
        // Empty iterator collection
        let iter = SequentialAndIterator::<TestIterator, u64>::new(vec![]);

        assert_eq!((0, Some(0)), iter.size_hint());
    }

    #[test]
    fn size_hint_single_iterator() {
        // Single iterator with known size
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);

        let iter = SequentialAndIterator::combine([a]);

        assert_eq!((0, Some(3)), iter.size_hint());
    }

    #[test]
    fn size_hint_multiple_iterators_same_size() {
        // Multiple iterators with same size
        let a = TestIterator::from([Ok(1), Ok(3), Ok(5)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(6)]);
        let c = TestIterator::from([Ok(3), Ok(4), Ok(5)]);

        let iter = SequentialAndIterator::combine([a, b, c]);

        // Upper bound is minimum of all (all are 3, so min is 3)
        assert_eq!((0, Some(3)), iter.size_hint());
    }

    #[test]
    fn size_hint_multiple_iterators_different_sizes() {
        // Multiple iterators with different sizes
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)]);
        let b = TestIterator::from([Ok(1), Ok(2)]);
        let c = TestIterator::from([Ok(1), Ok(2), Ok(3)]);

        let iter = SequentialAndIterator::combine([a, b, c]);

        // Upper bound is minimum of all (2 is the smallest)
        assert_eq!((0, Some(2)), iter.size_hint());
    }

    #[test]
    fn size_hint_one_empty_iterator() {
        // One empty iterator among non-empty ones
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([]);
        let c = TestIterator::from([Ok(1), Ok(2)]);

        let iter = SequentialAndIterator::combine([a, b, c]);

        // Upper bound is minimum, which is 0
        assert_eq!((0, Some(0)), iter.size_hint());
    }

    #[test]
    fn size_hint_after_partial_iteration() {
        // Test size_hint after consuming some elements
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(4)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(4)]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        // Initially: upper bound is 4
        assert_eq!((0, Some(4)), iter.size_hint());

        // Consume one element
        assert_eq!(Some(Ok(1)), iter.next());

        // After consuming, both internal iterators have 3 elements left
        assert_eq!((0, Some(3)), iter.size_hint());

        // Consume another
        assert_eq!(Some(Ok(2)), iter.next());

        // Both internal iterators have 2 elements left
        assert_eq!((0, Some(2)), iter.size_hint());
    }

    #[test]
    fn size_hint_after_exhaustion() {
        // Test size_hint after iterator is exhausted
        let a = TestIterator::from([Ok(1)]);
        let b = TestIterator::from([Ok(2)]);

        let mut iter = SequentialAndIterator::combine([a, b]);

        // Initially
        assert_eq!((0, Some(1)), iter.size_hint());

        // Exhaust the iterator (no intersection)
        assert_eq!(None, iter.next());

        // After exhaustion
        assert_eq!((0, Some(0)), iter.size_hint());
    }

    #[test]
    fn size_hint_lower_bound_always_zero() {
        // Verify lower bound is always 0 (since intersection might be empty)
        let a = TestIterator::from([Ok(1), Ok(3), Ok(5)]);
        let b = TestIterator::from([Ok(2), Ok(4), Ok(6)]);

        let iter = SequentialAndIterator::combine([a, b]);

        // Lower bound is 0 because there might be no intersection
        assert_eq!((0, Some(3)), iter.size_hint());
    }
}
