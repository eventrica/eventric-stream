//! The [`or`][or] module provides an iterator which provides the boolean OR
//! operation over a collection of sequential iterators, such that an item will
//! appear in the output if it occurs in any of the input iterators.
//!
//! [or]: self]

use std::cmp::Ordering;

use derive_more::with_trait::Debug;
use fancy_constructor::new;

use crate::{
    error::Error,
    utils::iteration::{
        CachingIterators,
        DoubleEndedIteratorCached as _,
        IteratorCached as _,
    },
};

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
pub struct SequentialOrIterator<I, T>(CachingIterators<I, T>)
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
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T, Error>;

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
                            let iter_val = match self.0.next {
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
                        (Some(Ok(iter_val)), None) => match self.0.next {
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
                    Some(value) => self.0.return_ok_some_next(value),
                    None => self.0.return_ok_none(),
                }
            }
        }
    }
}

impl<I, T> DoubleEndedIterator for SequentialOrIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        todo!()
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
    fn empty_iterators_combine_as_empty() {
        let empty: Vec<Result<u64, Error>> = Vec::new();

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
