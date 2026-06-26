//! The boolean AND combinator: [`Intersection`], a lazy set intersection over
//! sorted child iterators.

use std::{
    cmp::Ordering,
    iter::FusedIterator,
};

use derive_more::with_trait::Debug;
use fancy_constructor::new;

use super::{
    Cursor,
    Seek,
};

// =================================================================================================
// Intersection
// =================================================================================================

/// A boolean AND (set intersection) over several sorted child iterators.
///
/// Each child must yield `Result<T, E>` in ascending order. The intersection is
/// produced lazily and stays sorted: forward iteration ([`Iterator::next`])
/// emits the values present in *every* child in ascending order, and reverse
/// iteration ([`DoubleEndedIterator::next_back`]) is the exact mirror over
/// descending order.
///
/// `next` and `next_back` share one algorithm with the comparisons flipped (see
/// the comments on each); they are written out in full — rather than shared via
/// a macro — so the convergence logic stays easy to read and step through.
#[derive(new, Debug)]
#[new(const_fn, vis())]
pub(crate) struct Intersection<I, T, E>(Vec<Cursor<I>>)
where
    I: DoubleEndedIterator<Item = Result<T, E>>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T, E> Intersection<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + From<Intersection<I, T, E>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    /// Take an iterable value of iterators, and return an iterator of the same
    /// type which will implement the boolean AND operation on the input
    /// iterators.
    pub(crate) fn iter<S>(iters: S) -> I
    where
        S: IntoIterator<Item = I>,
    {
        let iters = iters.into_iter().map(Cursor::new).collect();

        I::from(Intersection::new(iters))
    }
}

impl<I, T, E> Iterator for Intersection<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + Seek<T>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T, E>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Intersection: no guaranteed lower bound, at most the smallest child.
        let upper = self.0.iter().filter_map(|iter| iter.size_hint().1).min();

        (0, upper)
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }

        // Drive a shared candidate up to the largest head value across the
        // children, advancing any child whose head lags behind it, until every
        // head agrees on the candidate (the next value common to all children).
        // Inputs are ascending, so "lagging behind" is `Less` (advance it) and
        // the candidate only ever moves up (`Greater`).
        let mut candidate = None;

        loop {
            let mut converged = true;

            for iter in &mut self.0 {
                match iter.peek() {
                    Some(Ok(value)) => match &mut candidate {
                        Some(current) => match value.cmp(current) {
                            Ordering::Greater => {
                                *current = *value;
                                converged = false;
                            }
                            // Leapfrog: skip the lagging child straight to the
                            // candidate (the largest head so far) rather than
                            // single-stepping it through every position below it.
                            Ordering::Less => {
                                let candidate = *current;
                                iter.seek(candidate);
                                converged = false;
                            }
                            Ordering::Equal => {}
                        },
                        None => candidate = Some(*value),
                    },
                    // A child has ended (`None`) or errored: the intersection
                    // cannot continue, so forward that terminal item as-is.
                    _ => return iter.next(),
                }
            }

            if converged {
                break;
            }
        }

        // Emit the agreed value, consuming it from every child so the next call
        // starts past it.
        candidate.map(Ok).inspect(|item| {
            for iter in &mut self.0 {
                match (item, iter.peek()) {
                    (Ok(lhs), Some(Ok(rhs))) if lhs.eq(rhs) => {
                        iter.next();
                    }
                    _ => {}
                }
            }
        })
    }
}

impl<I, T, E> DoubleEndedIterator for Intersection<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + Seek<T>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }

        // The mirror of `next`, taken from the back of each child (descending):
        // drive the candidate *down* to the smallest back value (`Less`),
        // advancing from the back any child whose head is still ahead of it
        // (`Greater`).
        let mut candidate = None;

        loop {
            let mut converged = true;

            for iter in &mut self.0 {
                match iter.peek_back() {
                    Some(Ok(value)) => match &mut candidate {
                        Some(current) => match value.cmp(current) {
                            Ordering::Less => {
                                *current = *value;
                                converged = false;
                            }
                            // Leapfrog (reverse): skip the lagging child straight
                            // down to the candidate (the smallest back so far)
                            // rather than single-stepping it down from above.
                            Ordering::Greater => {
                                let candidate = *current;
                                iter.seek_back(candidate);
                                converged = false;
                            }
                            Ordering::Equal => {}
                        },
                        None => candidate = Some(*value),
                    },
                    _ => return iter.next_back(),
                }
            }

            if converged {
                break;
            }
        }

        candidate.map(Ok).inspect(|item| {
            for iter in &mut self.0 {
                match (item, iter.peek_back()) {
                    (Ok(lhs), Some(Ok(rhs))) if lhs.eq(rhs) => {
                        iter.next_back();
                    }
                    _ => {}
                }
            }
        })
    }
}

impl<I, T, E> FusedIterator for Intersection<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + FusedIterator + Seek<T>,
    T: Copy + Debug + Ord + PartialOrd,
{
}

impl<I, T, E> Seek<T> for Intersection<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + Seek<T>,
    T: Copy + Debug + Ord + PartialOrd,
{
    // Seeking an intersection seeks every child: nothing below `target` can be in
    // the intersection from here on.
    fn seek(&mut self, target: T) {
        for iter in &mut self.0 {
            iter.seek(target);
        }
    }

    fn seek_back(&mut self, target: T) {
        for iter in &mut self.0 {
            iter.seek_back(target);
        }
    }
}

// =================================================================================================
// Tests
// =================================================================================================

#[cfg(test)]
mod tests {
    use error_stack::Report;

    use crate::{
        error::Error,
        iter::test_util::*,
    };

    #[test]
    fn intersects_forward() {
        let iter = and([leaf([1, 2, 3, 4, 5]), leaf([2, 3, 5, 7]), leaf([2, 5, 9])]);

        assert_eq!(forward(iter), vec![2, 5]);
    }

    #[test]
    fn intersects_backward() {
        let iter = and([leaf([1, 2, 3, 4, 5]), leaf([2, 3, 5, 7]), leaf([2, 5, 9])]);

        assert_eq!(backward(iter), vec![5, 2]);
    }

    #[test]
    fn is_empty_with_no_common_values() {
        let iter = and([leaf([1, 2, 3]), leaf([4, 5, 6])]);

        assert_eq!(forward(iter), Vec::<u64>::new());
    }

    #[test]
    fn single_child_is_identity() {
        assert_eq!(forward(and([leaf([1, 2, 3])])), vec![1, 2, 3]);
    }

    #[test]
    fn propagates_error_after_common_prefix() {
        let mut iter = and([
            leaf_results(vec![Ok(1), Err(Report::new(Error)), Ok(3)]),
            leaf([1, 2, 3]),
        ]);

        assert_eq!(iter.next().map(Result::unwrap), Some(1));
        assert!(iter.next().is_some_and(|value| value.is_err()));
    }

    #[test]
    fn is_double_ended_from_both_ends() {
        let mut iter = and([leaf([1, 2, 3, 4, 5]), leaf([1, 2, 3, 4, 5])]);

        assert_eq!(iter.next().map(Result::unwrap), Some(1));
        assert_eq!(iter.next_back().map(Result::unwrap), Some(5));
        assert_eq!(iter.next().map(Result::unwrap), Some(2));
        assert_eq!(iter.next_back().map(Result::unwrap), Some(4));
        assert_eq!(iter.next().map(Result::unwrap), Some(3));
        assert_eq!(iter.next().map(Result::unwrap), None);
    }

    #[test]
    fn with_initially_empty_child_is_empty() {
        // An empty child terminates the intersection via the `_ => return ...` arm.
        let iter = and([leaf(Vec::<u64>::new()), leaf([1, 2])]);

        assert_eq!(forward(iter), Vec::<u64>::new());
    }

    #[test]
    fn with_no_children_is_empty() {
        assert_eq!(forward(and(Vec::<TestIter>::new())), Vec::<u64>::new());
    }

    // Intersecting a long, dense child with a short, sparse one yields exactly the
    // sparse positions — the leapfrog `seek` must skip the dense gaps without
    // dropping or inventing a match.
    #[test]
    fn dense_with_sparse_intersects_via_seek() {
        let iter = and([leaf(0..1_000), leaf([7, 250, 251, 999])]);

        assert_eq!(forward(iter), vec![7, 250, 251, 999]);
    }

    // The reverse leapfrog must skip the dense gaps downward, equally.
    #[test]
    fn dense_with_sparse_intersects_via_seek_back() {
        let iter = and([leaf(0..1_000), leaf([7, 250, 251, 999])]);

        assert_eq!(backward(iter), vec![999, 251, 250, 7]);
    }

    // Seeking while also driven from the back (mixed direction) stays correct via
    // the single-step fallback.
    #[test]
    fn mixed_direction_with_seek_is_correct() {
        let mut iter = and([leaf(0..100), leaf([10, 20, 30, 40, 50])]);

        assert_eq!(iter.next().map(Result::unwrap), Some(10));
        assert_eq!(iter.next_back().map(Result::unwrap), Some(50));
        assert_eq!(iter.next().map(Result::unwrap), Some(20));
        assert_eq!(iter.next_back().map(Result::unwrap), Some(40));
        assert_eq!(iter.next().map(Result::unwrap), Some(30));
        assert_eq!(iter.next().map(Result::unwrap), None);
    }

    // Per the `Seek` contract: leapfrogging skips entries strictly below the target
    // without reading them, so a read error on a skipped entry is *not* surfaced
    // (it could not be in the intersection). If the error leaked, `forward`'s
    // `unwrap` would panic.
    #[test]
    fn seek_does_not_surface_errors_in_the_skipped_region() {
        let dense = leaf_results(vec![Ok(0), Err(Report::new(Error)), Ok(100)]);
        let sparse = leaf([100]);

        let iter = and([dense, sparse]);

        assert_eq!(forward(iter), vec![100]);
    }

    // The reverse mirror: leapfrogging *down* skips entries strictly above the
    // target (here above the candidate 0), so a read error there is not surfaced.
    #[test]
    fn seek_back_does_not_surface_errors_in_the_skipped_region() {
        let dense = leaf_results(vec![Ok(0), Err(Report::new(Error)), Ok(100)]);
        let sparse = leaf([0]);

        let iter = and([dense, sparse]);

        assert_eq!(backward(iter), vec![0]);
    }
}
