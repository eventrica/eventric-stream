//! The [`iteration`][self] module provides combinatorial iterators over
//! collections of sequential iterators, such that sequential order is
//! maintained while the output represents a boolean logical operation over the
//! input iterators.

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
            and::AndIter,
            or::OrIter,
        },
    };

    #[derive(Debug)]
    pub enum TestIterator {
        And(AndIter<TestIterator, u64>),
        Or(OrIter<TestIterator, u64>),
        Boxed(#[debug("Boxed")] Box<dyn DoubleEndedIterator<Item = Result<u64, Error>>>),
    }

    impl From<AndIter<TestIterator, u64>> for TestIterator {
        fn from(iter: AndIter<TestIterator, u64>) -> Self {
            Self::And(iter)
        }
    }

    impl From<OrIter<TestIterator, u64>> for TestIterator {
        fn from(iter: OrIter<TestIterator, u64>) -> Self {
            Self::Or(iter)
        }
    }

    impl<T> From<T> for TestIterator
    where
        T: Into<Vec<Result<u64, Error>>>,
    {
        fn from(vec: T) -> Self {
            Self::Boxed(Box::new(vec.into().into_iter()))
        }
    }

    impl Iterator for TestIterator {
        type Item = Result<u64, Error>;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Self::And(iter) => iter.next(),
                Self::Or(iter) => iter.next(),
                Self::Boxed(iter) => iter.next(),
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            match self {
                Self::And(iter) => iter.size_hint(),
                Self::Or(iter) => iter.size_hint(),
                Self::Boxed(iter) => iter.size_hint(),
            }
        }
    }

    impl DoubleEndedIterator for TestIterator {
        fn next_back(&mut self) -> Option<Self::Item> {
            match self {
                Self::And(iter) => iter.next_back(),
                Self::Or(iter) => iter.next_back(),
                Self::Boxed(iter) => iter.next_back(),
            }
        }
    }

    #[test]
    fn impl_iterator() {
        // Empty iterator
        let mut iter = TestIterator::from([]);

        assert_eq!(None, iter.next());

        // Single element
        let mut iter = TestIterator::from([Ok(42)]);

        assert_eq!(Some(Ok(42)), iter.next());
        assert_eq!(None, iter.next());

        // Multiple elements
        let mut iter = TestIterator::from([Ok(0), Ok(1), Ok(2), Ok(3)]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());

        // Multiple calls to next after exhaustion
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn impl_double_ended_iterator() {
        // Empty iterator
        let mut iter = TestIterator::from([]);

        assert_eq!(None, iter.next_back());

        // Single element
        let mut iter = TestIterator::from([Ok(42)]);

        assert_eq!(Some(Ok(42)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Multiple elements
        let mut iter = TestIterator::from([Ok(0), Ok(1), Ok(2), Ok(3)]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(Some(Ok(0)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Multiple calls to next_back after exhaustion
        assert_eq!(None, iter.next_back());
        assert_eq!(None, iter.next_back());
    }

    #[test]
    fn impl_both_iterator_and_double_ended_iterator() {
        // Empty iterator
        let mut iter = TestIterator::from([]);

        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next_back());

        // Single element - forward then backward
        let mut iter = TestIterator::from([Ok(42)]);

        assert_eq!(Some(Ok(42)), iter.next());
        assert_eq!(None, iter.next_back());

        // Single element - backward then forward
        let mut iter = TestIterator::from([Ok(42)]);

        assert_eq!(Some(Ok(42)), iter.next_back());
        assert_eq!(None, iter.next());

        // Multiple elements - alternating
        let mut iter = TestIterator::from([Ok(0), Ok(1), Ok(2), Ok(3)]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next_back());

        // Multiple elements - forward heavy
        let mut iter = TestIterator::from([Ok(0), Ok(1), Ok(2), Ok(3), Ok(4)]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next());
        assert_eq!(None, iter.next());

        // Multiple elements - backward heavy
        let mut iter = TestIterator::from([Ok(0), Ok(1), Ok(2), Ok(3), Ok(4)]);

        assert_eq!(Some(Ok(4)), iter.next_back());
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(None, iter.next_back());
    }

    #[test]
    fn impl_iterator_with_errors() {
        // Error at start
        let mut iter = TestIterator::from([Err(Error::Concurrency), Ok(1), Ok(2)]);

        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));

        // Error in middle
        let mut iter = TestIterator::from([Ok(0), Err(Error::Concurrency), Ok(2)]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));

        // Error at end
        let mut iter = TestIterator::from([Ok(0), Ok(1), Err(Error::Concurrency)]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next());
        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));

        // Multiple errors
        let mut iter = TestIterator::from([
            Ok(0),
            Err(Error::Concurrency),
            Ok(2),
            Err(Error::data("test")),
        ]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));
        assert_eq!(Some(Ok(2)), iter.next());
        assert!(matches!(iter.next(), Some(Err(Error::Data(_)))));

        // Only errors
        let mut iter = TestIterator::from([Err(Error::Concurrency), Err(Error::data("test"))]);

        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));
        assert!(matches!(iter.next(), Some(Err(Error::Data(_)))));
        assert_eq!(None, iter.next());
    }

    #[test]
    fn impl_double_ended_iterator_with_errors() {
        // Error at end
        let mut iter = TestIterator::from([Ok(0), Ok(1), Err(Error::Concurrency)]);

        assert!(matches!(iter.next_back(), Some(Err(Error::Concurrency))));

        // Error in middle
        let mut iter = TestIterator::from([Ok(0), Err(Error::Concurrency), Ok(2)]);

        assert_eq!(Some(Ok(2)), iter.next_back());
        assert!(matches!(iter.next_back(), Some(Err(Error::Concurrency))));

        // Error at start
        let mut iter = TestIterator::from([Err(Error::Concurrency), Ok(1), Ok(2)]);

        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert!(matches!(iter.next_back(), Some(Err(Error::Concurrency))));

        // Multiple errors backward
        let mut iter = TestIterator::from([
            Err(Error::Concurrency),
            Ok(1),
            Err(Error::data("test")),
            Ok(3),
        ]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert!(matches!(iter.next_back(), Some(Err(Error::Data(_)))));
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert!(matches!(iter.next_back(), Some(Err(Error::Concurrency))));
    }

    #[test]
    fn impl_both_iterator_and_double_ended_iterator_with_errors() {
        // Error encountered from forward direction
        let mut iter = TestIterator::from([Ok(0), Err(Error::Concurrency), Ok(2), Ok(3)]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));
        assert_eq!(Some(Ok(3)), iter.next_back());
        assert_eq!(Some(Ok(2)), iter.next());
        assert_eq!(None, iter.next());

        // Error encountered from backward direction
        let mut iter = TestIterator::from([Ok(0), Ok(1), Err(Error::Concurrency), Ok(3)]);

        assert_eq!(Some(Ok(3)), iter.next_back());
        assert!(matches!(iter.next_back(), Some(Err(Error::Concurrency))));
        assert_eq!(Some(Ok(0)), iter.next());
        assert_eq!(Some(Ok(1)), iter.next_back());
        assert_eq!(None, iter.next_back());

        // Errors on both ends
        let mut iter = TestIterator::from([
            Err(Error::Concurrency),
            Ok(1),
            Ok(2),
            Err(Error::data("test")),
        ]);

        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));
        assert!(matches!(iter.next_back(), Some(Err(Error::Data(_)))));
        assert_eq!(Some(Ok(1)), iter.next());
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert_eq!(None, iter.next());

        // Alternating values and errors
        let mut iter = TestIterator::from([
            Ok(0),
            Err(Error::Concurrency),
            Ok(2),
            Err(Error::data("test")),
        ]);

        assert_eq!(Some(Ok(0)), iter.next());
        assert!(matches!(iter.next_back(), Some(Err(Error::Data(_)))));
        assert_eq!(Some(Ok(2)), iter.next_back());
        assert!(matches!(iter.next(), Some(Err(Error::Concurrency))));
        assert_eq!(None, iter.next());
    }

    #[test]
    fn from_conversion() {
        // From empty vec
        let iter = TestIterator::from(vec![]);
        assert!(matches!(iter, TestIterator::Boxed(_)));

        // From vec with values
        let iter = TestIterator::from(vec![Ok(1), Ok(2), Ok(3)]);
        assert!(matches!(iter, TestIterator::Boxed(_)));

        // From array
        let iter = TestIterator::from([Ok(1), Ok(2), Ok(3)]);
        assert!(matches!(iter, TestIterator::Boxed(_)));

        // Verify it actually iterates correctly
        let mut iter = TestIterator::from([Ok(10), Ok(20), Ok(30)]);
        assert_eq!(Some(Ok(10)), iter.next());
        assert_eq!(Some(Ok(20)), iter.next());
        assert_eq!(Some(Ok(30)), iter.next());
        assert_eq!(None, iter.next());
    }
}
