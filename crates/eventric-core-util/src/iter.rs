pub mod and;
pub mod or;

use derive_more::Debug;
use fancy_constructor::new;

// =================================================================================================
// Iter
// =================================================================================================

/// A convenience trait alias for items compatible with the various iterator
/// utilities, which expect items to be comparable, and copy-friendly.
pub trait Item = Copy + PartialOrd;

// -------------------------------------------------------------------------------------------------

// Iterators

/// The [`CachingIterators`] type implements a collection of [`CachingIterator`]
/// instances, and an optional value of the iterator item type. This is used as
/// common internal state for various iteration utility structures/functions.
#[derive(new, Debug)]
#[new(name(inner), vis())]
struct CachingIterators<I, T>
where
    I: Iterator<Item = T>,
    T: Item,
{
    iterators: Vec<CachingIterator<I, T>>,
    #[new(default)]
    value: Option<T>,
}

impl<I, T> CachingIterators<I, T>
where
    I: Iterator<Item = T>,
    T: Item,
{
    pub fn new<S>(iterators: S) -> Self
    where
        S: IntoIterator<Item = I>,
    {
        Self::inner(iterators.into_iter().map(CachingIterator::new).collect())
    }
}

impl<S, I, T> From<S> for CachingIterators<I, T>
where
    S: IntoIterator<Item = I>,
    I: Iterator<Item = T>,
    T: Item,
{
    fn from(value: S) -> Self {
        CachingIterators::new(value)
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
    I: Iterator<Item = T>,
    T: Item,
{
    iterator: I,
    #[new(default)]
    value: Option<T>,
}

impl<I, T> CachingIterator<I, T>
where
    I: Iterator<Item = T>,
    T: Item,
{
    /// Returns the cached item value if it's not [`None`], otherwise returns
    /// (and caches) the value of [`Self::next`]. Note that if the underlying
    /// iterator returns [`None`], this will return cache and return that value.
    fn next_cached(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.value {
            Some(value) => Some(value),
            None => self.next(),
        }
    }

    /// Calls [`Self::next`] before returning a mutable reference to Self. A
    /// convenience method to advance the iterator while passing it to a
    /// function, etc.
    fn next_self_ref(&mut self) -> &mut Self {
        self.next();
        self
    }
}

impl<I, T> Iterator for CachingIterator<I, T>
where
    I: Iterator<Item = T>,
    T: Item,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.value = self.iterator.next();
        self.value
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod test {
    use derive_more::Debug;
    use fancy_constructor::new;

    use crate::iter::{
        and::SequentialAnd,
        or::SequentialOr,
    };

    // Test Iterator

    #[derive(new, Debug)]
    pub enum TestIterator {
        And(SequentialAnd<TestIterator, u64>),
        Or(SequentialOr<TestIterator, u64>),
        #[new]
        Vec(#[new(default)] usize, #[new(into)] Vec<u64>),
    }

    impl From<SequentialAnd<TestIterator, u64>> for TestIterator {
        fn from(value: SequentialAnd<TestIterator, u64>) -> Self {
            Self::And(value)
        }
    }

    impl From<SequentialOr<TestIterator, u64>> for TestIterator {
        fn from(value: SequentialOr<TestIterator, u64>) -> Self {
            Self::Or(value)
        }
    }

    impl From<Vec<u64>> for TestIterator {
        fn from(value: Vec<u64>) -> Self {
            Self::Vec(0, value)
        }
    }

    impl Iterator for TestIterator {
        type Item = u64;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Self::And(iterator) => iterator.next(),
                Self::Or(iterator) => iterator.next(),
                Self::Vec(pos, values) => {
                    if *pos >= values.len() {
                        None
                    } else {
                        *pos += 1;

                        Some(*values.get(*pos - 1).unwrap())
                    }
                }
            }
        }
    }

    // Tests

    #[test]
    fn test_iterator_returns_supplied_empty_vec() {
        let empty: Vec<u64> = vec![];

        assert_eq!(empty, TestIterator::new([]).collect::<Vec<_>>());
    }

    #[test]
    fn test_iterator_returns_supplied_vec() {
        let seq: Vec<u64> = vec![0, 1, 2, 3, 4, 5];

        assert_eq!(seq.clone(), TestIterator::new(seq).collect::<Vec<_>>());
    }
}
