pub mod and;
pub mod or;

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

// -------------------------------------------------------------------------------------------------

// Caching Iterator(s)

/// The [`CachingIterators`] type implements a collection of [`CachingIterator`]
/// instances, and an optional value of the iterator item type. This is used as
/// common internal state for various iteration utility structures/functions.
#[derive(new, Debug)]
#[new(args(iterators: impl IntoIterator<Item = I>))]
struct CachingIterators<I, T>
where
    I: Iterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[new(default)]
    errored: bool,
    #[new(val(iterators.into_iter().map(CachingIterator::new).collect()))]
    iterators: Vec<CachingIterator<I, T>>,
    #[new(default)]
    value: Option<T>,
}

impl<I, T> CachingIterators<I, T>
where
    I: Iterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[allow(clippy::unnecessary_wraps)]
    fn return_err(&mut self, err: Error) -> Option<Result<T, Error>> {
        self.errored = true;
        self.iterators.clear();
        self.value = None;

        Some(Err(err))
    }

    #[allow(clippy::unnecessary_wraps)]
    fn return_ok_some(&mut self, value: T) -> Option<Result<T, Error>> {
        self.errored = false;
        self.iterators.retain(|iter| iter.value.is_some());
        self.value = Some(value);

        Some(Ok(value))
    }

    fn return_ok_none(&mut self) -> Option<Result<T, Error>> {
        self.errored = false;
        self.iterators.clear();
        self.value = None;

        None
    }
}

impl<S, I, T> From<S> for CachingIterators<I, T>
where
    S: IntoIterator<Item = I>,
    I: Iterator<Item = Result<T, Error>>,
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
    I: Iterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[new(default)]
    errored: bool,
    iterator: I,
    #[new(default)]
    value: Option<T>,
}

impl<I, T> Iterator for CachingIterator<I, T>
where
    I: Iterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.errored {
            return None;
        }

        if let Some(result) = self.iterator.next() {
            match result {
                Ok(value) => {
                    self.value = Some(value);

                    Some(Ok(value))
                }
                Err(err) => {
                    self.errored = true;
                    self.value = None;

                    Some(Err(err))
                }
            }
        } else {
            self.value = None;

            None
        }
    }
}

impl<I, T> IteratorCached for CachingIterator<I, T>
where
    I: Iterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    /// Returns the cached item value if it's not [`None`], otherwise returns
    /// (and caches) the value of [`Self::next`]. Note that if the underlying
    /// iterator returns [`None`], this will return cache and return that value.
    fn next_cached(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.value {
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
        util::iteration::{
            and::SequentialAndIterator,
            or::SequentialOrIterator,
        },
    };

    #[derive(new, Debug)]
    pub enum TestIterator {
        And(SequentialAndIterator<TestIterator, u64>),
        Or(SequentialOrIterator<TestIterator, u64>),
        #[new]
        Vec(#[new(default)] usize, #[new(into)] Vec<u64>),
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
            Self::Vec(0, vec)
        }
    }

    impl Iterator for TestIterator {
        type Item = Result<u64, Error>;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Self::And(iterator) => iterator.next(),
                Self::Or(iterator) => iterator.next(),
                Self::Vec(pos, values) => {
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
