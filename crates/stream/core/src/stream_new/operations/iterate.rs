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

use crate::stream_new::Result;

// =================================================================================================
// Iterate
// =================================================================================================

// Boolean And Iterator

macro_rules! and_next {
    ($peek:ident, $next:ident, $update:path, $advance:path) => {
        #[inline]
        fn $next(&mut self) -> Option<Self::Item> {
            if self.0.is_empty() {
                return None;
            }

            let mut candidate = None;

            loop {
                let mut converged = true;

                for iter in &mut self.0 {
                    match iter.$peek() {
                        Some(Ok(next)) => match &mut candidate {
                            Some(current_candidate) => match next.cmp(current_candidate) {
                                $update => {
                                    *current_candidate = *next;
                                    converged = false;
                                }
                                $advance => {
                                    iter.$next();
                                    converged = false;
                                }
                                Ordering::Equal => {}
                            },
                            None => candidate = Some(*next),
                        },
                        _ => return iter.$next(),
                    }
                }

                if converged {
                    break;
                }
            }

            candidate.map(Ok).inspect(|item| {
                for iter in &mut self.0 {
                    match (item, iter.$peek()) {
                        (Ok(lhs), Some(Ok(rhs))) if lhs.eq(rhs) => {
                            iter.$next();
                        }
                        _ => {}
                    }
                }
            })
        }
    };
}

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct AndIter<I, T>(Vec<DoubleEndedPeekable<I>>)
where
    I: DoubleEndedIterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T> AndIter<I, T>
where
    I: DoubleEndedIterator<Item = Result<T>> + From<AndIter<I, T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    /// Take a an iterable value of iterators, and return an iterator of the
    /// same type which will implement the boolean AND operation on the input
    /// iterators.
    pub fn iter<S>(iters: S) -> I
    where
        S: IntoIterator<Item = I>,
    {
        let iters = iters
            .into_iter()
            .map(DoubleEndedPeekableExt::double_ended_peekable)
            .collect();

        I::from(AndIter::new(iters))
    }
}

impl<I, T> DoubleEndedIterator for AndIter<I, T>
where
    I: DoubleEndedIterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[rustfmt::skip]
    and_next!(peek_back, next_back, Ordering::Less, Ordering::Greater);
}

impl<I, T> FusedIterator for AndIter<I, T>
where
    I: DoubleEndedIterator<Item = Result<T>> + FusedIterator,
    T: Copy + Debug + Ord + PartialOrd,
{
}

impl<I, T> Iterator for AndIter<I, T>
where
    I: DoubleEndedIterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T>;

    #[rustfmt::skip]
    and_next!(peek, next, Ordering::Greater, Ordering::Less);

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.0.is_empty() {
            return (0, Some(0));
        }

        // Lower bound is 0 (might be no intersection)
        // Upper bound is the minimum of all iterator upper bounds
        let upper = self.0.iter().filter_map(|iter| iter.size_hint().1).min();

        (0, upper)
    }
}

// -------------------------------------------------------------------------------------------------

// Boolean Or Iterator

macro_rules! or_next {
    ($peek:ident, $next:ident, $update:path) => {
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
                                $update => {
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
                    match (item, self.0[index].$peek()) {
                        (Ok(lhs), Some(Ok(rhs))) if lhs.eq(rhs) => {
                            self.0[index].$next();
                        }
                        _ => {}
                    }
                }
            })
        }
    };
}

#[derive(new, Debug)]
#[new(const_fn, vis())]
pub struct OrIter<I, T>(Vec<DoubleEndedPeekable<I>>)
where
    I: DoubleEndedIterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T> OrIter<I, T>
where
    I: DoubleEndedIterator<Item = Result<T>> + From<OrIter<I, T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    /// Take a an iterable value of iterators, and return an iterator of the
    /// same type which will implement the boolean OR operation on the input
    /// iterators.
    pub fn iter<S>(iters: S) -> I
    where
        S: IntoIterator<Item = I>,
    {
        let iters = iters
            .into_iter()
            .map(DoubleEndedPeekableExt::double_ended_peekable)
            .collect();

        I::from(OrIter::new(iters))
    }
}

impl<I, T> DoubleEndedIterator for OrIter<I, T>
where
    I: DoubleEndedIterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    or_next!(peek_back, next_back, Ordering::Greater);
}

impl<I, T> FusedIterator for OrIter<I, T>
where
    I: DoubleEndedIterator<Item = Result<T>> + FusedIterator,
    T: Copy + Debug + Ord + PartialOrd,
{
}

impl<I, T> Iterator for OrIter<I, T>
where
    I: DoubleEndedIterator<Item = Result<T>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T>;

    or_next!(peek, next, Ordering::Less);

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
