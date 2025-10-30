//! The [`iteration`][iteration] module provides combinatorial iterators over
//! collections of sequential iterators, such that sequential order is
//! maintained while the output represents a boolean logical operation over the
//! input iterators.
//!
//! [iteration]: self

pub(crate) mod and;
pub(crate) mod or;

use derive_more::with_trait::Debug;
use fancy_constructor::new;

use crate::error::Error;

// =================================================================================================
// Iteration
// =================================================================================================

// Iterator Cached

trait IteratorCached: Iterator {
    fn next_cached(&mut self) -> Option<<Self as Iterator>::Item>;
}

trait DoubleEndedIteratorCached: DoubleEndedIterator {
    fn next_back_cached(&mut self) -> Option<<Self as Iterator>::Item>;
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
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[new(default)]
    errored: bool,
    #[new(val(iterators.into_iter().map(CachingIterator::new).collect()))]
    iterators: Vec<CachingIterator<I, T>>,
    #[new(default)]
    next: Option<T>,
    #[new(default)]
    next_back: Option<T>,
}

impl<I, T> CachingIterators<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[allow(clippy::unnecessary_wraps)]
    fn return_err(&mut self, err: Error) -> Option<Result<T, Error>> {
        self.errored = true;
        self.iterators.clear();
        self.next = None;
        self.next_back = None;

        Some(Err(err))
    }

    #[allow(clippy::unnecessary_wraps)]
    fn return_ok_some_next(&mut self, value: T) -> Option<Result<T, Error>> {
        self.errored = false;
        self.iterators.retain(|iter| iter.next.is_some());
        self.next = Some(value);

        Some(Ok(value))
    }

    fn return_ok_none(&mut self) -> Option<Result<T, Error>> {
        self.errored = false;
        self.iterators.clear();
        self.next = None;
        self.next = None;

        None
    }
}

impl<S, I, T> From<S> for CachingIterators<I, T>
where
    S: IntoIterator<Item = I>,
    I: DoubleEndedIterator<Item = Result<T, Error>>,
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
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[new(default)]
    errored: bool,
    iterator: I,
    #[new(default)]
    next: Option<T>,
    #[new(default)]
    next_back: Option<T>,
}

impl<I, T> CachingIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[allow(clippy::unnecessary_wraps)]
    fn return_err(&mut self, err: Error) -> Option<Result<T, Error>> {
        self.errored = true;
        self.next = None;
        self.next_back = None;

        Some(Err(err))
    }

    #[allow(clippy::unnecessary_wraps)]
    fn return_ok_some_next(&mut self, value: T) -> Option<Result<T, Error>> {
        self.next = Some(value);

        Some(Ok(value))
    }

    #[allow(clippy::unnecessary_wraps)]
    fn return_ok_some_next_back(&mut self, value: T) -> Option<Result<T, Error>> {
        self.next_back = Some(value);

        Some(Ok(value))
    }

    fn return_ok_none(&mut self) -> Option<Result<T, Error>> {
        self.next = None;
        self.next_back = None;

        None
    }
}

impl<I, T> DoubleEndedIterator for CachingIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.errored {
            return None;
        }

        if let Some(result) = self.iterator.next_back() {
            match result {
                Ok(value) => self.return_ok_some_next_back(value),
                Err(err) => self.return_err(err),
            }
        } else {
            self.return_ok_none()
        }
    }
}

impl<I, T> DoubleEndedIteratorCached for CachingIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    fn next_back_cached(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.next_back {
            Some(value) => Some(Ok(value)),
            _ => self.next_back(),
        }
    }
}

impl<I, T> Iterator for CachingIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.errored {
            return None;
        }

        if let Some(result) = self.iterator.next() {
            match result {
                Ok(value) => self.return_ok_some_next(value),
                Err(err) => self.return_err(err),
            }
        } else {
            self.return_ok_none()
        }
    }
}

impl<I, T> IteratorCached for CachingIterator<I, T>
where
    I: DoubleEndedIterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    fn next_cached(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.next {
            Some(value) => Some(Ok(value)),
            _ => self.next(),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use derive_more::Debug;
    use fancy_constructor::new;

    use crate::{
        error::Error,
        utils::iteration::{
            and::SequentialAndIterator,
            or::SequentialOrIterator,
        },
    };

    #[derive(new, Debug)]
    pub enum TestIterator {
        And(SequentialAndIterator<TestIterator, u64>),
        Or(SequentialOrIterator<TestIterator, u64>),
        #[new]
        Vec(
            #[new(default)] usize,
            #[new(default)] usize,
            #[new(into)] Vec<u64>,
        ),
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
            Self::Vec(0, vec.len() - 1, vec)
        }
    }

    impl Iterator for TestIterator {
        type Item = Result<u64, Error>;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Self::And(iterator) => iterator.next(),
                Self::Or(iterator) => iterator.next(),
                Self::Vec(pos, _, values) => {
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

    impl DoubleEndedIterator for TestIterator {
        fn next_back(&mut self) -> Option<Self::Item> {
            match self {
                Self::And(iterator) => iterator.next_back(),
                Self::Or(iterator) => iterator.next_back(),
                Self::Vec(next_pos, next_back_pos, values) => {
                    if *next_back_pos <= *next_pos {
                        None
                    } else {
                        *next_back_pos -= 1;

                        Some(Ok(*values.get(*next_back_pos + 1).unwrap()))
                    }
                }
            }
        }
    }

    #[test]
    fn test_iterator_returns_supplied_empty_vec() {
        assert_eq!(
            Vec::<Result<u64, Error>>::new(),
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
}
