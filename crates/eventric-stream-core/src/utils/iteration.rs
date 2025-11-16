//! The [`iteration`][iteration] module provides combinatorial iterators over
//! collections of sequential iterators, such that sequential order is
//! maintained while the output represents a boolean logical operation over the
//! input iterators.
//!
//! [iteration]: self

pub(crate) mod and;
pub(crate) mod or;

// =================================================================================================
// Iteration
// =================================================================================================

// Tests

#[cfg(test)]
mod tests {
    use derive_more::Debug;

    use crate::{
        error::Error,
        utils::iteration::{
            and::SequentialAndIterator,
            or::SequentialOrIterator,
        },
    };

    #[derive(Debug)]
    pub enum TestIterator {
        And(SequentialAndIterator<TestIterator, u64>),
        Or(SequentialOrIterator<TestIterator, u64>),
        Boxed(#[debug("Boxed")] Box<dyn DoubleEndedIterator<Item = Result<u64, Error>>>),
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

    impl<T> From<T> for TestIterator
    where
        T: Into<Vec<u64>>,
    {
        fn from(vec: T) -> Self {
            Self::Boxed(Box::new(vec.into().into_iter().map(Ok)))
        }
    }

    impl Iterator for TestIterator {
        type Item = Result<u64, Error>;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Self::And(iterator) => iterator.next(),
                Self::Or(iterator) => iterator.next(),
                Self::Boxed(iter) => iter.next(),
            }
        }
    }

    impl DoubleEndedIterator for TestIterator {
        fn next_back(&mut self) -> Option<Self::Item> {
            match self {
                Self::And(iterator) => iterator.next_back(),
                Self::Or(iterator) => iterator.next_back(),
                Self::Boxed(iter) => iter.next_back(),
            }
        }
    }

    #[test]
    fn impl_iterator() {
        let mut iter = TestIterator::from([0, 1, 2, 3]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn impl_double_ended_iterator() {
        let mut iter = TestIterator::from([0, 1, 2, 3]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(Some(Ok(0)), iter.next_back());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn impl_both_iterator_and_double_ended_iterator() {
        let mut iter = TestIterator::from([0, 1, 2, 3]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(None, iter.next());
    }
}
