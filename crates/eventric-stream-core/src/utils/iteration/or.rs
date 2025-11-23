//! The [`or`][or] module provides an iterator which provides the boolean OR
//! operation over a collection of sequential iterators, such that an item will
//! appear in the output if it occurs in any of the input iterators.
//!
//! [or]: self]

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
use smallvec::SmallVec;

use crate::error::Error;

// =================================================================================================
// Or
// =================================================================================================

/// Macro to implement `next()` or `next_back()` for [`SequentialOrIterator`].
///
/// The OR iterator uses a single-pass algorithm to find the minimum/maximum
/// value and collect indices of all iterators that have that value.
macro_rules! impl_or_next {
    ($peek:ident, $next:ident, $next_if_eq:ident, $winning_ordering:path) => {
        #[inline]
        fn $next(&mut self) -> Option<Self::Item> {
            if self.0.is_empty() {
                return None;
            }

            let mut candidate = None;
            let mut indices: SmallVec<[usize; 8]> = SmallVec::new();

            for (index, iter) in self.0.iter_mut().enumerate() {
                match iter.$peek() {
                    Some(Ok(next)) => {
                        if let Some(current_candidate) = &mut candidate {
                            match next.cmp(current_candidate) {
                                $winning_ordering => {
                                    *current_candidate = *next;
                                    indices.clear();
                                    indices.push(index);
                                }
                                Ordering::Equal => {
                                    indices.push(index);
                                }
                                _ => {}
                            }
                        } else {
                            candidate = Some(*next);
                            indices.push(index);
                        }
                    }
                    Some(Err(_)) => return iter.$next(),
                    None => {}
                }
            }

            candidate.map(Ok).inspect(|item| {
                for &index in &indices {
                    self.0[index].$next_if_eq(item);
                }
            })
        }
    };
}

/// The [`SequentialOrIterator`] implements a sorted set union.
///
/// This iterator type represents an iterator over the combined values of a set
/// of sequential iterators. The resulting iterator is equivalent to an ordered
/// union (∪) of the underlying iterators (i.e. values appear only once, and
/// are totally ordered).
///
/// # Algorithm
///
/// The iterator finds the minimum value across all input iterators and returns
/// it. For each iteration:
/// - Peek at the front/back of all iterators
/// - Find the minimum/maximum value
/// - Advance all iterators that have that value (to ensure uniqueness)
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
/// let b = vec![Ok(2), Ok(3), Ok(4)];
///
/// let result: Vec<_> = SequentialOrIterator::combine([a.into_iter(), b.into_iter()])
///     .collect::<Result<Vec<_>, _>>()
///     .unwrap();
///
/// assert_eq!(result, vec![1, 2, 3, 4, 5]); // All unique values from both iterators
/// ```
#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct SequentialOrIterator<I, T>(Vec<DoubleEndedPeekable<I>>)
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T> SequentialOrIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>> + From<SequentialOrIterator<I, T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    /// Take a an iterable value of iterators, and return an iterator of the
    /// same type which will implement the boolean OR operation on the input
    /// iterators.
    pub fn combine<S>(iters: S) -> I
    where
        S: IntoIterator<Item = I>,
    {
        let iters = iters
            .into_iter()
            .map(DoubleEndedPeekableExt::double_ended_peekable)
            .collect();

        I::from(SequentialOrIterator::new(iters))
    }
}

impl<I, T> DoubleEndedIterator for SequentialOrIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    impl_or_next!(peek_back, next_back, next_back_if_eq, Ordering::Greater);
}

impl<I, T> Iterator for SequentialOrIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T, Error>;

    impl_or_next!(peek, next, next_if_eq, Ordering::Less);

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.0.is_empty() {
            return (0, Some(0));
        }

        // Lower bound is the maximum of all iterator lower bounds
        let lower = self
            .0
            .iter()
            .map(|iter| iter.size_hint().0)
            .max()
            .unwrap_or(0);

        // Upper bound is the sum of all iterator upper bounds
        let upper = self.0.iter().try_fold(0usize, |acc, iter| {
            iter.size_hint().1.and_then(|n| acc.checked_add(n))
        });

        (lower, upper)
    }
}

impl<I, T> FusedIterator for SequentialOrIterator<I, T>
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
            or::SequentialOrIterator,
            tests::TestIterator,
        },
    };

    #[test]
    fn impl_iterator() {
        // Empty
        let a = TestIterator::from([]);
        let b = TestIterator::from([]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert_eq!(None, iter.next());

        // Disjoint sets - no overlap
        let a = TestIterator::from([Ok(0), Ok(1), Ok(2)]);
        let b = TestIterator::from([Ok(3), Ok(4), Ok(5)]);
        let c = TestIterator::from([Ok(6), Ok(7), Ok(8)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next());
        assert_eq!(Some(Ok(6)), iter.next());
        assert_eq!(Some(Ok(7)), iter.next());
        assert_eq!(Some(Ok(8)), iter.next());
        assert_eq!(None, iter.next());

        // Partial overlap
        let a = TestIterator::from([Ok(0), Ok(2), Ok(4), Ok(6)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(4), Ok(5)]);
        let c = TestIterator::from([Ok(1), Ok(2), Ok(4), Ok(7)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next());
        assert_eq!(Some(Ok(6)), iter.next());
        assert_eq!(Some(Ok(7)), iter.next());
        assert_eq!(None, iter.next());

        // Complete overlap - all iterators have same values
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let c = TestIterator::from([Ok(1), Ok(2), Ok(3)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());

        // Variable lengths
        let a = TestIterator::from([Ok(0), Ok(3), Ok(4)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let c = TestIterator::from([Ok(0), Ok(1), Ok(4), Ok(5)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next());
        assert_eq!(None, iter.next());

        // Single iterator
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);

        let mut iter = SequentialOrIterator::combine([a]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());

        // Two iterators with one empty
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());

        // Complex pattern with gaps
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(5), Ok(7), Ok(9)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(4), Ok(5), Ok(6), Ok(7)]);
        let c = TestIterator::from([Ok(3), Ok(5), Ok(7), Ok(8)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next());
        assert_eq!(Some(Ok(6)), iter.next());
        assert_eq!(Some(Ok(7)), iter.next());
        assert_eq!(Some(Ok(8)), iter.next());
        assert_eq!(Some(Ok(9)), iter.next());
        assert_eq!(None, iter.next());

        // Interleaved values
        let a = TestIterator::from([Ok(0), Ok(2), Ok(4), Ok(6), Ok(8)]);
        let b = TestIterator::from([Ok(1), Ok(3), Ok(5), Ok(7), Ok(9)]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next());
        assert_eq!(Some(Ok(6)), iter.next());
        assert_eq!(Some(Ok(7)), iter.next());
        assert_eq!(Some(Ok(8)), iter.next());
        assert_eq!(Some(Ok(9)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn impl_double_ended_iterator() {
        // Empty
        let a = TestIterator::from([]);
        let b = TestIterator::from([]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert_eq!(None, iter.next_back());

        // Disjoint sets - no overlap
        let a = TestIterator::from([Ok(0), Ok(1), Ok(2)]);
        let b = TestIterator::from([Ok(3), Ok(4), Ok(5)]);
        let c = TestIterator::from([Ok(6), Ok(7), Ok(8)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(8)), iter.next_back());
        assert_eq!(Some(Ok(7)), iter.next_back());
        assert_eq!(Some(Ok(6)), iter.next_back());
        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(Some(Ok(0)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Partial overlap
        let a = TestIterator::from([Ok(0), Ok(2), Ok(4), Ok(6)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(4), Ok(5)]);
        let c = TestIterator::from([Ok(1), Ok(2), Ok(4), Ok(7)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(7)), iter.next_back());
        assert_eq!(Some(Ok(6)), iter.next_back());
        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(Some(Ok(0)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Complete overlap - all iterators have same values
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let c = TestIterator::from([Ok(1), Ok(2), Ok(3)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Variable lengths
        let a = TestIterator::from([Ok(0), Ok(3), Ok(4)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let c = TestIterator::from([Ok(0), Ok(1), Ok(4), Ok(5)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(Some(Ok(0)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Single iterator
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);

        let mut iter = SequentialOrIterator::combine([a]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Two iterators with one empty
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Complex pattern with gaps
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(5), Ok(7), Ok(9)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(4), Ok(5), Ok(6), Ok(7)]);
        let c = TestIterator::from([Ok(3), Ok(5), Ok(7), Ok(8)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(9)), iter.next_back());
        assert_eq!(Some(Ok(8)), iter.next_back());
        assert_eq!(Some(Ok(7)), iter.next_back());
        assert_eq!(Some(Ok(6)), iter.next_back());
        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Interleaved values
        let a = TestIterator::from([Ok(0), Ok(2), Ok(4), Ok(6), Ok(8)]);
        let b = TestIterator::from([Ok(1), Ok(3), Ok(5), Ok(7), Ok(9)]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert_eq!(Some(Ok(9)), iter.next_back());
        assert_eq!(Some(Ok(8)), iter.next_back());
        assert_eq!(Some(Ok(7)), iter.next_back());
        assert_eq!(Some(Ok(6)), iter.next_back());
        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(Some(Ok(0)), iter.next_back());
        assert_eq!(None, iter.next_back());
    }

    #[test]
    fn impl_iterator_and_double_ended_iterator() {
        // Partial overlap - alternating next and next_back
        let a = TestIterator::from([Ok(0), Ok(2), Ok(4), Ok(6), Ok(8)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(4), Ok(5), Ok(6)]);
        let c = TestIterator::from([Ok(1), Ok(2), Ok(4), Ok(6), Ok(7)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(8)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(7)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(6)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(None, iter.next());

        // Complete overlap - mixed iteration
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)]);
        let c = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

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

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(8)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(6)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(None, iter.next());

        // Disjoint sets - mixed iteration
        let a = TestIterator::from([Ok(0), Ok(1), Ok(2)]);
        let b = TestIterator::from([Ok(3), Ok(4), Ok(5)]);
        let c = TestIterator::from([Ok(6), Ok(7), Ok(8)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(8)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(7)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(6)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(None, iter.next());

        // Interleaved values - mixed iteration
        let a = TestIterator::from([Ok(0), Ok(2), Ok(4), Ok(6), Ok(8)]);
        let b = TestIterator::from([Ok(1), Ok(3), Ok(5), Ok(7), Ok(9)]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(9)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(8)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(7)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(6)), iter.next_back());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(None, iter.next());

        // Complex pattern - ensure correctness
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3), Ok(5), Ok(7), Ok(9), Ok(11)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(4), Ok(5), Ok(6), Ok(7), Ok(10)]);
        let c = TestIterator::from([Ok(3), Ok(5), Ok(7), Ok(8), Ok(9)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(11)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(10)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(9)), iter.next_back());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(Some(Ok(8)), iter.next_back());
        assert_eq!(Some(Ok(5)), iter.next());
        assert_eq!(Some(Ok(7)), iter.next_back());
        assert_eq!(Some(Ok(6)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn impl_iterator_with_errors() {
        // Error in first iterator - should return error immediately
        let a = TestIterator::from([Ok(1), Err(Error::Concurrency), Ok(3)]);
        let b = TestIterator::from([Ok(2), Ok(3), Ok(4)]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));

        // Error in second iterator
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([Err(Error::Concurrency), Ok(2), Ok(3)]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));

        // Error in third iterator - should still return values from other iterators
        let a = TestIterator::from([Ok(1), Ok(3), Ok(5)]);
        let b = TestIterator::from([Ok(2), Ok(4), Ok(6)]);
        let c = TestIterator::from([Err(Error::Concurrency)]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));

        // Error after some successful values
        let a = TestIterator::from([Ok(1), Ok(2), Err(Error::Concurrency)]);
        let b = TestIterator::from([Ok(1), Ok(3), Ok(4)]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));
    }

    #[test]
    fn impl_double_ended_iterator_with_errors() {
        // Error at end of first iterator
        let a = TestIterator::from([Ok(1), Ok(2), Err(Error::Concurrency)]);
        let b = TestIterator::from([Ok(1), Ok(2), Ok(3)]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert!(matches!(iter.next_back(), Some(Err(Error::Concurrency))));

        // Error at end of second iterator
        let a = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        let b = TestIterator::from([Ok(4), Ok(5), Err(Error::Concurrency)]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert!(matches!(iter.next_back(), Some(Err(Error::Concurrency))));

        // Error in middle - mixed iteration
        let a = TestIterator::from([Ok(1), Ok(2), Ok(5), Ok(6)]);
        let b = TestIterator::from([Ok(3), Err(Error::Concurrency), Ok(7), Ok(8)]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert_eq!(Some(Ok(8)), iter.next_back());
        assert_eq!(Some(Ok(7)), iter.next_back());
        assert!(matches!(iter.next_back(), Some(Err(Error::Concurrency))));
    }
}
