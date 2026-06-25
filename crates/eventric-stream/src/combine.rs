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
use fancy_constructor::new;
use smallvec::SmallVec;

// =================================================================================================
// Seek
// =================================================================================================

/// A sorted iterator that can skip forward to the first item `>= target` in one
/// step, rather than being advanced one element at a time. Implemented by the
/// index iterators (by re-seeking the underlying scan) — it is what lets
/// [`AndIter`] leapfrog a lagging child over a long run of non-matching
/// positions instead of single-stepping through every one of them.
pub(crate) trait Seek<T> {
    /// Advance so the next item yielded is the first one `>= target` (a no-op
    /// if already at or past `target`).
    ///
    /// Items strictly below `target` are skipped *without being read*, so a
    /// read error among them is not surfaced. This never changes a result:
    /// a child is only sought up to the intersection's shared candidate,
    /// and nothing below it can be in the intersection — so the skipped
    /// (and unread) region is exactly the part that cannot affect the
    /// answer.
    fn seek(&mut self, target: T);

    /// The reverse mirror of [`seek`](Self::seek): advance *from the back* so
    /// the next item yielded by `next_back` is the first one `<= target` (a
    /// no-op if already at or before `target`). Items strictly above
    /// `target` are skipped without being read, with the same
    /// (result-preserving) error-skipping consequence — it is what lets a
    /// reverse intersection leapfrog symmetrically with the forward one.
    fn seek_back(&mut self, target: T);
}

// -------------------------------------------------------------------------------------------------

// Cursor

/// A double-ended peeking wrapper (one-element front and back lookahead) that
/// also supports forward [`Seek`] — replacing a plain peekable so the
/// combinators can both peek *and* skip a child forward.
///
/// Each direction's seek re-seeks the backend only while the cursor has not
/// been driven from the *other* end (`from_back` guards forward `seek`,
/// `from_front` guards `seek_back`). Once the opposite end has been observed,
/// re-seeking would discard that end's progress, so it falls back to
/// single-stepping — always correct, just unoptimised (the index query is
/// consumed in a single direction in practice; the fallback only guards the
/// mixed-direction case the unit tests exercise).
#[derive(Debug)]
struct Cursor<I>
where
    I: DoubleEndedIterator,
{
    #[debug("..")]
    inner: I,
    #[debug("..")]
    front: Option<I::Item>,
    #[debug("..")]
    back: Option<I::Item>,
    from_front: bool,
    from_back: bool,
}

impl<I> Cursor<I>
where
    I: DoubleEndedIterator,
{
    fn new(inner: I) -> Self {
        Self {
            inner,
            front: None,
            back: None,
            from_front: false,
            from_back: false,
        }
    }

    fn peek(&mut self) -> Option<&I::Item> {
        self.from_front = true;

        if self.front.is_none() {
            self.front = self.inner.next().or_else(|| self.back.take());
        }

        self.front.as_ref()
    }

    fn peek_back(&mut self) -> Option<&I::Item> {
        self.from_back = true;

        if self.back.is_none() {
            self.back = self.inner.next_back().or_else(|| self.front.take());
        }

        self.back.as_ref()
    }
}

impl<I> Iterator for Cursor<I>
where
    I: DoubleEndedIterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.peek();
        self.front.take()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.inner.size_hint();
        let cached = usize::from(self.front.is_some()) + usize::from(self.back.is_some());

        (lower + cached, upper.map(|upper| upper + cached))
    }
}

impl<I> DoubleEndedIterator for Cursor<I>
where
    I: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.peek_back();
        self.back.take()
    }
}

impl<I, T, E> Cursor<I>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + Seek<T>,
    T: Copy + Ord,
{
    // Skip forward so the next peek/next yields the first item `>= target`.
    fn seek(&mut self, target: T) {
        // A cached front already at/after target stays put; a stale `Ok` front
        // below target is dropped so the backend (or the single-step fallback) can
        // advance past it; an `Err` front is left in place for `next` to surface.
        let stale = match self.front.as_ref() {
            Some(Ok(value)) => *value < target,
            Some(Err(_)) => return,
            None => false,
        };

        if stale {
            self.front = None;
        } else if self.front.is_some() {
            return;
        }

        if self.from_back {
            loop {
                let lagging = match self.peek() {
                    Some(Ok(value)) => *value < target,
                    _ => false,
                };

                if lagging {
                    self.next();
                } else {
                    break;
                }
            }
        } else {
            self.inner.seek(target);
        }
    }

    // The reverse mirror of `seek`: skip back so the next next_back yields the
    // first item `<= target`.
    fn seek_back(&mut self, target: T) {
        // Mirror of `seek` with the comparison flipped: keep a cached back at/below
        // target, drop a stale Ok back above it, keep an Err back for next_back.
        let stale = match self.back.as_ref() {
            Some(Ok(value)) => *value > target,
            Some(Err(_)) => return,
            None => false,
        };

        if stale {
            self.back = None;
        } else if self.back.is_some() {
            return;
        }

        if self.from_front {
            loop {
                let lagging = match self.peek_back() {
                    Some(Ok(value)) => *value > target,
                    _ => false,
                };

                if lagging {
                    self.next_back();
                } else {
                    break;
                }
            }
        } else {
            self.inner.seek_back(target);
        }
    }
}

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
pub(crate) struct AndIter<I, T, E>(Vec<Cursor<I>>)
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
        let iters = iters.into_iter().map(Cursor::new).collect();

        I::from(AndIter::new(iters))
    }
}

impl<I, T, E> Iterator for AndIter<I, T, E>
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

impl<I, T, E> DoubleEndedIterator for AndIter<I, T, E>
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

impl<I, T, E> FusedIterator for AndIter<I, T, E>
where
    I: DoubleEndedIterator<Item = Result<T, E>> + FusedIterator + Seek<T>,
    T: Copy + Debug + Ord + PartialOrd,
{
}

impl<I, T, E> Seek<T> for AndIter<I, T, E>
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
pub(crate) struct OrIter<I, T, E>(Vec<Cursor<I>>)
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
        let iters = iters.into_iter().map(Cursor::new).collect();

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

impl<I, T, E> Seek<T> for OrIter<I, T, E>
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

    use super::{
        AndIter,
        OrIter,
        Seek,
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

    impl Seek<u64> for TestIter {
        fn seek(&mut self, target: u64) {
            match self {
                Self::And(iter) => iter.seek(target),
                Self::Or(iter) => iter.seek(target),
                // A leaf is a sorted `Vec`: drop everything before the first item
                // `>= target` — including any `Err` in that skipped run — so this
                // stand-in matches the index layer's re-seek, which jumps past the
                // unread region wholesale rather than stopping at an error in it.
                Self::Leaf(iter) => {
                    let slice = iter.as_slice();
                    let skip = slice
                        .iter()
                        .position(|value| matches!(value, Ok(value) if *value >= target))
                        .unwrap_or(slice.len());

                    if skip > 0 {
                        iter.nth(skip - 1);
                    }
                }
            }
        }

        fn seek_back(&mut self, target: u64) {
            match self {
                Self::And(iter) => iter.seek_back(target),
                Self::Or(iter) => iter.seek_back(target),
                // The reverse mirror: keep up to the last item `<= target` and drop
                // the trailing run above it (including any trailing `Err`).
                Self::Leaf(iter) => {
                    let slice = iter.as_slice();
                    let keep = slice
                        .iter()
                        .rposition(|value| matches!(value, Ok(value) if *value <= target))
                        .map_or(0, |index| index + 1);
                    let drop = slice.len() - keep;

                    if drop > 0 {
                        iter.nth_back(drop - 1);
                    }
                }
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

    // Seek
    // -----------------------------------------------------------------------------

    #[test]
    fn seek_positions_leaf_at_first_at_or_after_target() {
        let mut iter = leaf([1, 5, 9, 12]);

        iter.seek(6);

        assert_eq!(forward(iter), vec![9, 12]);
    }

    #[test]
    fn seek_past_the_end_exhausts() {
        let mut iter = leaf([1, 2, 3]);

        iter.seek(100);

        assert_eq!(forward(iter), Vec::<u64>::new());
    }

    #[test]
    fn seek_below_the_head_is_a_no_op() {
        let mut iter = leaf([5, 6, 7]);

        iter.seek(2);

        assert_eq!(forward(iter), vec![5, 6, 7]);
    }

    // Intersecting a long, dense child with a short, sparse one yields exactly the
    // sparse positions — the leapfrog `seek` must skip the dense gaps without
    // dropping or inventing a match.
    #[test]
    fn and_dense_with_sparse_intersects_via_seek() {
        let iter = and([leaf(0..1_000), leaf([7, 250, 251, 999])]);

        assert_eq!(forward(iter), vec![7, 250, 251, 999]);
    }

    // The seek path must thread through a nested OR: `(dense_a OR dense_b) AND
    // sparse` is the index query's exact shape (type union AND tag).
    #[test]
    fn and_of_or_with_sparse_intersects_via_seek() {
        let evens = (0..1_000).filter(|n| n % 2 == 0);
        let odds = (0..1_000).filter(|n| n % 2 == 1);
        let iter = and([or([leaf(evens), leaf(odds)]), leaf([7, 250, 999])]);

        assert_eq!(forward(iter), vec![7, 250, 999]);
    }

    // Seeking while the cursor is also driven from the back (mixed direction) must
    // stay correct via the single-step fallback.
    #[test]
    fn and_mixed_direction_with_seek_is_correct() {
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
    // (it could not be in the intersection). Here the dense child's `Err` sits
    // between its head and the candidate (100); the intersection is the clean
    // `[100]`. If the error leaked through, `forward`'s `unwrap` would panic.
    #[test]
    fn seek_does_not_surface_errors_in_the_skipped_region() {
        let dense = leaf_results(vec![Ok(0), Err(Report::new(Error)), Ok(100)]);
        let sparse = leaf([100]);

        let iter = and([dense, sparse]);

        assert_eq!(forward(iter), vec![100]);
    }

    // Seek (reverse) — the mirror of the forward cases above.
    // -----------------------------------------------------------------------------

    #[test]
    fn seek_back_positions_leaf_at_last_at_or_before_target() {
        let mut iter = leaf([1, 5, 9, 12]);

        iter.seek_back(8);

        // `backward` consumes via `next_back`; after seek_back(8) the largest item
        // `<= 8` is 5, then 1.
        assert_eq!(backward(iter), vec![5, 1]);
    }

    #[test]
    fn seek_back_above_the_head_is_a_no_op() {
        let mut iter = leaf([5, 6, 7]);

        iter.seek_back(100);

        assert_eq!(backward(iter), vec![7, 6, 5]);
    }

    // Reverse intersection of a long dense child with a short sparse one: the
    // reverse leapfrog must skip the dense gaps downward without dropping a match.
    #[test]
    fn and_dense_with_sparse_intersects_via_seek_back() {
        let iter = and([leaf(0..1_000), leaf([7, 250, 251, 999])]);

        assert_eq!(backward(iter), vec![999, 251, 250, 7]);
    }

    // The reverse seek must thread through a nested OR, same as the forward one.
    #[test]
    fn and_of_or_with_sparse_intersects_via_seek_back() {
        let evens = (0..1_000).filter(|n| n % 2 == 0);
        let odds = (0..1_000).filter(|n| n % 2 == 1);
        let iter = and([or([leaf(evens), leaf(odds)]), leaf([7, 250, 999])]);

        assert_eq!(backward(iter), vec![999, 250, 7]);
    }

    // The reverse mirror of the error-skip contract: leapfrogging *down* skips
    // entries strictly above the target without reading them, so a read error there
    // (here above the candidate 0) is not surfaced.
    #[test]
    fn seek_back_does_not_surface_errors_in_the_skipped_region() {
        let dense = leaf_results(vec![Ok(0), Err(Report::new(Error)), Ok(100)]);
        let sparse = leaf([0]);

        let iter = and([dense, sparse]);

        assert_eq!(backward(iter), vec![0]);
    }
}
