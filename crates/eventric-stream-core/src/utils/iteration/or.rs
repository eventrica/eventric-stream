//! The [`or`][or] module provides an iterator which provides the boolean OR
//! operation over a collection of sequential iterators, such that an item will
//! appear in the output if it occurs in any of the input iterators.
//!
//! [or]: self]

use derive_more::with_trait::Debug;
use double_ended_peekable::{
    DoubleEndedPeekable,
    DoubleEndedPeekableExt,
};
use fancy_constructor::new;

use crate::error::Error;

// =================================================================================================
// Or
// =================================================================================================

/// The [`SequentialOrIterator`] type represents an iterator over the combined
/// values of a set of sequential iterators. The resulting iterator is
/// equivalent to an ordered union (∪) of the underlying iterators (i.e. values
/// appear only once, and are totally ordered).
///
/// See local unit tests for simple examples.
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
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut current = None;

        for iter in &mut self.0 {
            match iter.peek_back() {
                Some(Ok(next)) => match &mut current {
                    Some(current) => *current = *next.max(current),
                    None => current = Some(*next),
                },
                Some(Err(_)) => return iter.next_back(),
                None => {}
            }
        }

        current.map(Ok).inspect(|item| {
            for iter in &mut self.0 {
                iter.next_back_if_eq(item);
            }
        })
    }
}

impl<I, T> Iterator for SequentialOrIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current = None;

        for iter in &mut self.0 {
            match iter.peek() {
                Some(Ok(next)) => match &mut current {
                    Some(current) => *current = *next.min(current),
                    None => current = Some(*next),
                },
                Some(Err(_)) => return iter.next(),
                None => {}
            }
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
