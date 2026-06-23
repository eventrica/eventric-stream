//! Generic boolean set-algebra over sorted, fallible iterators: [`AndIter`]
//! (intersection) and [`OrIter`] (union). These combinators are independent of
//! the stream — they work over any `DoubleEndedIterator<Item = Result<T, E>>`
//! whose `Ok` values are `Copy + Ord` and ascending — and back the index-driven
//! query in `stream::store`.

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

// =================================================================================================
// Combine
// =================================================================================================

// Boolean And Iterator

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
pub(crate) struct AndIter<I, T, E>(Vec<DoubleEndedPeekable<I>>)
where
    I: DoubleEndedIterator<Item = Result<T, E>>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T, E> AndIter<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + From<AndIter<I, T, E>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    /// Take an iterable value of iterators, and return an iterator of the same
    /// type which will implement the boolean AND operation on the input
    /// iterators.
    pub(crate) fn iter<S>(iters: S) -> I
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

impl<I, T, E> Iterator for AndIter<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>>,
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
                            Ordering::Less => {
                                iter.next();
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

impl<I, T, E> DoubleEndedIterator for AndIter<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>>,
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
                            Ordering::Greater => {
                                iter.next_back();
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

impl<I, T, E> FusedIterator for AndIter<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + FusedIterator,
    T: Copy + Debug + Ord + PartialOrd,
{
}

// -------------------------------------------------------------------------------------------------

// Boolean Or Iterator

/// A boolean OR (set union) over several sorted child iterators.
///
/// Each child must yield `Result<T, E>` in ascending order. The union is
/// produced lazily and stays sorted with duplicates collapsed: forward
/// iteration ([`Iterator::next`]) emits the smallest head value across the
/// children in ascending order, and reverse iteration
/// ([`DoubleEndedIterator::next_back`]) is the mirror, emitting the largest in
/// descending order.
///
/// As with [`AndIter`], `next`/`next_back` are written out in full (with the
/// comparison flipped) rather than macro-shared, for readability.
#[derive(new, Debug)]
#[new(const_fn, vis())]
pub(crate) struct OrIter<I, T, E>(Vec<DoubleEndedPeekable<I>>)
where
    I: DoubleEndedIterator<Item = Result<T, E>>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T, E> OrIter<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + From<OrIter<I, T, E>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    /// Take an iterable value of iterators, and return an iterator of the same
    /// type which will implement the boolean OR operation on the input
    /// iterators.
    pub(crate) fn iter<S>(iters: S) -> I
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

impl<I, T, E> Iterator for OrIter<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    type Item = Result<T, E>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Union: at least the largest child, at most the sum of all children.
        let lower = self
            .0
            .iter()
            .map(|iter| iter.size_hint().0)
            .max()
            .unwrap_or(0);
        let upper = self.0.iter().try_fold(0usize, |sum, iter| {
            iter.size_hint().1.and_then(|upper| sum.checked_add(upper))
        });

        (lower, upper)
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }

        // Find the smallest head value across all children and remember every
        // child that currently holds it. Inputs are ascending, so a smaller
        // head (`Less`) becomes the new candidate and resets the holder set; an
        // equal head joins the holder set.
        let mut candidate = None;
        let mut indices: SmallVec<[usize; 8]> = SmallVec::new();

        for (index, iter) in self.0.iter_mut().enumerate() {
            match iter.peek() {
                Some(Ok(value)) => {
                    if let Some(current) = &mut candidate {
                        match value.cmp(current) {
                            Ordering::Less => {
                                *current = *value;
                                indices.clear();
                                indices.push(index);
                            }
                            Ordering::Equal => {
                                indices.push(index);
                            }
                            Ordering::Greater => {}
                        }
                    } else {
                        candidate = Some(*value);
                        indices.push(index);
                    }
                }
                // An errored child short-circuits; an ended child is skipped.
                Some(Err(_)) => return iter.next(),
                None => {}
            }
        }

        // Emit the chosen value, consuming it from every child that held it so
        // duplicates collapse to a single output.
        candidate.map(Ok).inspect(|item| {
            for &index in &indices {
                match (item, self.0[index].peek()) {
                    (Ok(lhs), Some(Ok(rhs))) if lhs.eq(rhs) => {
                        self.0[index].next();
                    }
                    _ => {}
                }
            }
        })
    }
}

impl<I, T, E> DoubleEndedIterator for OrIter<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }

        // The mirror of `next`, taken from the back of each child (descending):
        // pick the *largest* back value (`Greater` becomes the new candidate).
        let mut candidate = None;
        let mut indices: SmallVec<[usize; 8]> = SmallVec::new();

        for (index, iter) in self.0.iter_mut().enumerate() {
            match iter.peek_back() {
                Some(Ok(value)) => {
                    if let Some(current) = &mut candidate {
                        match value.cmp(current) {
                            Ordering::Greater => {
                                *current = *value;
                                indices.clear();
                                indices.push(index);
                            }
                            Ordering::Equal => {
                                indices.push(index);
                            }
                            Ordering::Less => {}
                        }
                    } else {
                        candidate = Some(*value);
                        indices.push(index);
                    }
                }
                Some(Err(_)) => return iter.next_back(),
                None => {}
            }
        }

        candidate.map(Ok).inspect(|item| {
            for &index in &indices {
                match (item, self.0[index].peek_back()) {
                    (Ok(lhs), Some(Ok(rhs))) if lhs.eq(rhs) => {
                        self.0[index].next_back();
                    }
                    _ => {}
                }
            }
        })
    }
}

impl<I, T, E> FusedIterator for OrIter<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + FusedIterator,
    T: Copy + Debug + Ord + PartialOrd,
{
}

// =================================================================================================
// Tests
// =================================================================================================

#[cfg(test)]
mod tests {
    use error_stack::Report;

    use super::{
        AndIter,
        OrIter,
    };
    use crate::error::{
        Error,
        Result,
    };

    // A concrete iterator type satisfying the self-referential
    // `From<AndIter<Self, _>>` / `From<OrIter<Self, _>>` bound, so AND/OR trees
    // can be composed and exercised in isolation. `Leaf` wraps a fixed sequence.
    #[derive(Debug)]
    enum TestIter {
        And(AndIter<TestIter, u64, Report<Error>>),
        Or(OrIter<TestIter, u64, Report<Error>>),
        Leaf(std::vec::IntoIter<Result<u64>>),
    }

    impl From<AndIter<TestIter, u64, Report<Error>>> for TestIter {
        fn from(iter: AndIter<TestIter, u64, Report<Error>>) -> Self {
            Self::And(iter)
        }
    }

    impl From<OrIter<TestIter, u64, Report<Error>>> for TestIter {
        fn from(iter: OrIter<TestIter, u64, Report<Error>>) -> Self {
            Self::Or(iter)
        }
    }

    impl Iterator for TestIter {
        type Item = Result<u64>;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Self::And(iter) => iter.next(),
                Self::Or(iter) => iter.next(),
                Self::Leaf(iter) => iter.next(),
            }
        }
    }

    impl DoubleEndedIterator for TestIter {
        fn next_back(&mut self) -> Option<Self::Item> {
            match self {
                Self::And(iter) => iter.next_back(),
                Self::Or(iter) => iter.next_back(),
                Self::Leaf(iter) => iter.next_back(),
            }
        }
    }

    fn leaf<I>(values: I) -> TestIter
    where
        I: IntoIterator<Item = u64>,
    {
        let values = values.into_iter().map(Ok).collect::<Vec<Result<u64>>>();

        TestIter::Leaf(values.into_iter())
    }

    fn leaf_results(values: Vec<Result<u64>>) -> TestIter {
        TestIter::Leaf(values.into_iter())
    }

    fn and<I>(iters: I) -> TestIter
    where
        I: IntoIterator<Item = TestIter>,
    {
        AndIter::<TestIter, u64, Report<Error>>::iter(iters)
    }

    fn or<I>(iters: I) -> TestIter
    where
        I: IntoIterator<Item = TestIter>,
    {
        OrIter::<TestIter, u64, Report<Error>>::iter(iters)
    }

    fn forward(iter: TestIter) -> Vec<u64> {
        iter.map(Result::unwrap).collect()
    }

    fn backward(iter: TestIter) -> Vec<u64> {
        iter.rev().map(Result::unwrap).collect()
    }

    // And -------------------------------------------------------------------------------------

    #[test]
    fn and_intersects_forward() {
        let iter = and([leaf([1, 2, 3, 4, 5]), leaf([2, 3, 5, 7]), leaf([2, 5, 9])]);

        assert_eq!(forward(iter), vec![2, 5]);
    }

    #[test]
    fn and_intersects_backward() {
        let iter = and([leaf([1, 2, 3, 4, 5]), leaf([2, 3, 5, 7]), leaf([2, 5, 9])]);

        assert_eq!(backward(iter), vec![5, 2]);
    }

    #[test]
    fn and_is_empty_with_no_common_values() {
        let iter = and([leaf([1, 2, 3]), leaf([4, 5, 6])]);

        assert_eq!(forward(iter), Vec::<u64>::new());
    }

    #[test]
    fn and_of_single_child_is_identity() {
        assert_eq!(forward(and([leaf([1, 2, 3])])), vec![1, 2, 3]);
    }

    #[test]
    fn and_propagates_error_after_common_prefix() {
        let mut iter = and([
            leaf_results(vec![Ok(1), Err(Report::new(Error)), Ok(3)]),
            leaf([1, 2, 3]),
        ]);

        assert_eq!(iter.next().map(Result::unwrap), Some(1));
        assert!(iter.next().is_some_and(|value| value.is_err()));
    }

    #[test]
    fn and_is_double_ended_from_both_ends() {
        let mut iter = and([leaf([1, 2, 3, 4, 5]), leaf([1, 2, 3, 4, 5])]);

        assert_eq!(iter.next().map(Result::unwrap), Some(1));
        assert_eq!(iter.next_back().map(Result::unwrap), Some(5));
        assert_eq!(iter.next().map(Result::unwrap), Some(2));
        assert_eq!(iter.next_back().map(Result::unwrap), Some(4));
        assert_eq!(iter.next().map(Result::unwrap), Some(3));
        assert_eq!(iter.next().map(Result::unwrap), None);
    }

    // Or --------------------------------------------------------------------------------------

    #[test]
    fn or_unions_and_dedupes_forward() {
        let iter = or([leaf([1, 3, 5]), leaf([2, 3, 6]), leaf([3, 7])]);

        assert_eq!(forward(iter), vec![1, 2, 3, 5, 6, 7]);
    }

    #[test]
    fn or_unions_and_dedupes_backward() {
        let iter = or([leaf([1, 3, 5]), leaf([2, 3, 6]), leaf([3, 7])]);

        assert_eq!(backward(iter), vec![7, 6, 5, 3, 2, 1]);
    }

    #[test]
    fn or_of_single_child_is_identity() {
        assert_eq!(forward(or([leaf([1, 2, 3])])), vec![1, 2, 3]);
    }

    #[test]
    fn or_propagates_error() {
        let mut iter = or([leaf_results(vec![Err(Report::new(Error))]), leaf([1, 2])]);

        assert!(iter.next().is_some_and(|value| value.is_err()));
    }

    // Composition
    // -----------------------------------------------------------------------------

    #[test]
    fn and_of_ors_composes() {
        // (a OR b) AND c
        let iter = and([or([leaf([1, 2]), leaf([2, 3, 4])]), leaf([2, 4, 6])]);

        assert_eq!(forward(iter), vec![2, 4]);
    }

    // Edge cases
    // -----------------------------------------------------------------------------

    #[test]
    fn or_skips_initially_empty_child() {
        // An empty child hits the `None => {}` skip arm; non-empty siblings
        // still produce the full union.
        let iter = or([leaf(Vec::<u64>::new()), leaf([1, 2])]);

        assert_eq!(forward(iter), vec![1, 2]);
    }

    #[test]
    fn and_with_initially_empty_child_is_empty() {
        // An empty child terminates the intersection via the `_ => return ...`
        // arm.
        let iter = and([leaf(Vec::<u64>::new()), leaf([1, 2])]);

        assert_eq!(forward(iter), Vec::<u64>::new());
    }

    #[test]
    fn and_with_no_children_is_empty() {
        assert_eq!(forward(and(Vec::<TestIter>::new())), Vec::<u64>::new());
    }

    #[test]
    fn or_with_no_children_is_empty() {
        assert_eq!(forward(or(Vec::<TestIter>::new())), Vec::<u64>::new());
        assert_eq!(backward(or(Vec::<TestIter>::new())), Vec::<u64>::new());
    }
}
