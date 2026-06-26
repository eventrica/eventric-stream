//! The boolean OR combinator: [`Union`], a lazy, duplicate-collapsing set union
//! over sorted child iterators.

use std::{
    cmp::Ordering,
    iter::FusedIterator,
};

use derive_more::with_trait::Debug;
use fancy_constructor::new;
use smallvec::SmallVec;

use super::{
    Cursor,
    Seek,
};

// =================================================================================================
// Union
// =================================================================================================

/// A boolean OR (set union) over several sorted child iterators.
///
/// Each child must yield `Result<T, E>` in ascending order. The union is
/// produced lazily and stays sorted with duplicates collapsed: forward
/// iteration ([`Iterator::next`]) emits the smallest head value across the
/// children in ascending order, and reverse iteration
/// ([`DoubleEndedIterator::next_back`]) is the mirror, emitting the largest in
/// descending order.
///
/// As with [`Intersection`](super::intersection::Intersection),
/// `next`/`next_back` are written out in full (with the comparison flipped)
/// rather than macro-shared, for readability.
#[derive(new, Debug)]
#[new(const_fn, vis())]
pub(crate) struct Union<I, T, E>(Vec<Cursor<I>>)
where
    I: DoubleEndedIterator<Item = Result<T, E>>,
    T: Copy + Debug + Ord + PartialOrd;

impl<I, T, E> Union<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + From<Union<I, T, E>>,
    T: Copy + Debug + Ord + PartialOrd,
{
    /// Take an iterable value of iterators, and return an iterator of the same
    /// type which will implement the boolean OR operation on the input
    /// iterators.
    pub(crate) fn iter<S>(iters: S) -> I
    where
        S: IntoIterator<Item = I>,
    {
        let iters = iters.into_iter().map(Cursor::new).collect();

        I::from(Union::new(iters))
    }
}

impl<I, T, E> Iterator for Union<I, T, E>
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

impl<I, T, E> DoubleEndedIterator for Union<I, T, E>
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

impl<I, T, E> FusedIterator for Union<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + FusedIterator,
    T: Copy + Debug + Ord + PartialOrd,
{
}

impl<I, T, E> Seek<T> for Union<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + Seek<T>,
    T: Copy + Debug + Ord + PartialOrd,
{
    // Seeking a union seeks every child: the union from `target` onward is the
    // union of each child from `target` onward.
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
    fn unions_and_dedupes_forward() {
        let iter = or([leaf([1, 3, 5]), leaf([2, 3, 6]), leaf([3, 7])]);

        assert_eq!(forward(iter), vec![1, 2, 3, 5, 6, 7]);
    }

    #[test]
    fn unions_and_dedupes_backward() {
        let iter = or([leaf([1, 3, 5]), leaf([2, 3, 6]), leaf([3, 7])]);

        assert_eq!(backward(iter), vec![7, 6, 5, 3, 2, 1]);
    }

    #[test]
    fn single_child_is_identity() {
        assert_eq!(forward(or([leaf([1, 2, 3])])), vec![1, 2, 3]);
    }

    #[test]
    fn propagates_error() {
        let mut iter = or([leaf_results(vec![Err(Report::new(Error))]), leaf([1, 2])]);

        assert!(iter.next().is_some_and(|value| value.is_err()));
    }

    #[test]
    fn skips_initially_empty_child() {
        // An empty child hits the `None => {}` skip arm; non-empty siblings still
        // produce the full union.
        let iter = or([leaf(Vec::<u64>::new()), leaf([1, 2])]);

        assert_eq!(forward(iter), vec![1, 2]);
    }

    #[test]
    fn with_no_children_is_empty() {
        assert_eq!(forward(or(Vec::<TestIter>::new())), Vec::<u64>::new());
        assert_eq!(backward(or(Vec::<TestIter>::new())), Vec::<u64>::new());
    }
}
