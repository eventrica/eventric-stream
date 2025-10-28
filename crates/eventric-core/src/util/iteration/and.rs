use std::cmp::Ordering;

use derive_more::with_trait::Debug;
use fancy_constructor::new;

use crate::{
    error::Error,
    util::iteration::{
        CachingIterators,
        IteratorCached as _,
    },
};

// =================================================================================================
// And
// =================================================================================================

/// The [`SequentialAnd`] type represents an iterator over the combined values
/// of a set of sequential iterators (such as the
/// [`SequentialIterator`][seq_int] type found in the [`index`][index] module).
/// The resulting iterator is equivalent to an ordered intersection (∩) of the
/// underlying iterators (i.e. values appear only once, and are totally
/// ordered).
///
/// See local unit tests for simple examples.
///
/// [index]: crate::persistence::index
/// [seq_int]: crate::persistence::index::SequentialIterator
#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct SequentialAndIterator<I, T>(CachingIterators<I, T>)
where
    I: Iterator<Item = Result<T, Error>>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T> SequentialAndIterator<I, T>
where
    I: Iterator<Item = Result<T, Error>> + From<SequentialAndIterator<I, T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    pub fn combine<S>(iterators: S) -> I
    where
        S: IntoIterator<Item = I>,
    {
        let iterators = iterators.into();
        let iterator = SequentialAndIterator::new(iterators);

        I::from(iterator)
    }
}

impl<I, T> Iterator for SequentialAndIterator<I, T>
where
    I: Iterator<Item = Result<T, Error>>,
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

                'a: loop {
                    'b: for iter in iters.iter_mut() {
                        match (iter.next_cached()?, current) {
                            (Ok(iter_val), Some(current_val)) => {
                                let iter_val = match self.0.value {
                                    Some(previous_val) if iter_val == previous_val => {
                                        iter.next()?
                                    }
                                    _ => Ok(iter_val),
                                };

                                match iter_val {
                                    Ok(iter_val) => match iter_val.cmp(&current_val) {
                                        Ordering::Less => loop {
                                            match iter.next()? {
                                                Ok(iter_val) => match iter_val.cmp(&current_val) {
                                                    Ordering::Less => {}
                                                    Ordering::Equal => continue 'b,
                                                    Ordering::Greater => {
                                                        current = Some(iter_val);
                                                        continue 'a;
                                                    }
                                                },
                                                Err(err) => return self.0.return_err(err),
                                            }
                                        },
                                        Ordering::Equal => {}
                                        Ordering::Greater => {
                                            current = Some(iter_val);
                                            continue 'a;
                                        }
                                    },
                                    Err(err) => return self.0.return_err(err),
                                }
                            }
                            (Ok(iter_val), _) => {
                                current = Some(match self.0.value {
                                    Some(previous_val) if iter_val == previous_val => {
                                        match iter.next()? {
                                            Ok(iter_val) => iter_val,
                                            Err(err) => return self.0.return_err(err),
                                        }
                                    }
                                    _ => iter_val,
                                });
                            }
                            (Err(err), _) => return self.0.return_err(err),
                        }
                    }

                    break 'a;
                }

                match current {
                    Some(value) => self.0.return_ok_some(value),
                    None => self.0.return_ok_none(),
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use crate::{
        error::Error,
        util::iteration::{
            and::SequentialAndIterator,
            tests::TestIterator,
        },
    };

    #[test]
    fn sequential_and_empty_iterators_combine_as_empty() {
        let empty: Vec<Result<u64, Error>> = Vec::from_iter([]);

        let a = TestIterator::new([]);
        let b = TestIterator::new([]);

        assert_eq!(
            empty,
            SequentialAndIterator::combine([a, b]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn sequential_and_iterators_combine_sequentially_same_lengths() {
        let combined = Vec::from_iter([Ok(2), Ok(3)]);

        let a = TestIterator::new([0, 1, 2, 3]);
        let b = TestIterator::new([1, 2, 3, 4]);
        let c = TestIterator::new([2, 3, 4, 5]);

        assert_eq!(
            combined,
            SequentialAndIterator::combine([a, b, c]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn sequential_and_iterators_combine_single_iterator() {
        let combined = Vec::from_iter([Ok(0), Ok(1), Ok(2), Ok(3)]);

        let a = TestIterator::new([0, 1, 2, 3]);

        assert_eq!(
            combined,
            SequentialAndIterator::combine([a]).collect::<Vec<_>>()
        );
    }

    #[test]
    fn sequential_and_iterators_combine_sequentially_differing_lengths() {
        let combined = Vec::from_iter([Ok(2), Ok(3)]);

        let a = TestIterator::new([2, 3]);
        let b = TestIterator::new([1, 2, 3, 4]);
        let c = TestIterator::new([2, 3, 4]);

        assert_eq!(
            combined,
            SequentialAndIterator::combine([a, b, c]).collect::<Vec<_>>()
        );

        let a = TestIterator::new([1, 2, 3, 4]);
        let b = TestIterator::new([2, 3]);
        let c = TestIterator::new([2, 3, 4]);

        assert_eq!(
            combined,
            SequentialAndIterator::combine([a, b, c]).collect::<Vec<_>>()
        );

        let a = TestIterator::new([2, 3, 4]);
        let b = TestIterator::new([2, 3]);
        let c = TestIterator::new([1, 2, 3, 4]);

        assert_eq!(
            combined,
            SequentialAndIterator::combine([a, b, c]).collect::<Vec<_>>()
        );
    }
}
