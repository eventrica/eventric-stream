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
    use crate::utils::iteration::{
        or::SequentialOrIterator,
        tests::TestIterator,
    };

    #[test]
    fn impl_iterator() {
        // Empty

        let a = TestIterator::from([]);
        let b = TestIterator::from([]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert_eq!(None, iter.next());

        // Matched Lengths

        let a = TestIterator::from([0, 4]);
        let b = TestIterator::from([1, 5]);
        let c = TestIterator::from([2, 3]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next());
        assert_eq!(None, iter.next());

        // Variable Lengths

        let a = TestIterator::from([0, 3, 4]);
        let b = TestIterator::from([1, 2, 3]);
        let c = TestIterator::from([0, 1, 4, 5]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn impl_double_ended_iterator() {
        // Empty

        let a = TestIterator::from([]);
        let b = TestIterator::from([]);

        let mut iter = SequentialOrIterator::combine([a, b]);

        assert_eq!(None, iter.next_back());

        // Matched Lengths

        let a = TestIterator::from([0, 4]);
        let b = TestIterator::from([1, 5]);
        let c = TestIterator::from([2, 3]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(Some(Ok(0)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Variable Lengths

        let a = TestIterator::from([0, 3, 4]);
        let b = TestIterator::from([1, 2, 3]);
        let c = TestIterator::from([0, 1, 4, 5]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

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
        // Matched Lengths

        let a = TestIterator::from([0, 4]);
        let b = TestIterator::from([1, 5]);
        let c = TestIterator::from([2, 3]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(None, iter.next());

        // Variable Lengths

        let a = TestIterator::from([0, 3, 4]);
        let b = TestIterator::from([1, 2, 3]);
        let c = TestIterator::from([0, 1, 4, 5]);

        let mut iter = SequentialOrIterator::combine([a, b, c]);

        assert_eq!(Some(Ok(5)), iter.next_back());
        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(None, iter.next());
    }
}
