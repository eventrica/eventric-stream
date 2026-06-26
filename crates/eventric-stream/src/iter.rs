//! Generic boolean set-algebra over sorted, fallible iterators, mirroring
//! `std::iter`: the [`Intersection`](intersection::Intersection) (AND) and
//! [`Union`](union::Union) (OR) combinators each live in their own submodule
//! over the shared [`Cursor`] + [`Seek`] machinery defined here. They work over
//! any `DoubleEndedIterator<Item = Result<T, E>>` whose `Ok` values are `Copy +
//! Ord` and ascending, and back the index-driven query in `stream::store`.

use derive_more::with_trait::Debug;

pub(crate) mod intersection;
pub(crate) mod union;

// =================================================================================================
// Seek
// =================================================================================================

/// A sorted iterator that can skip forward to the first item `>= target` in one
/// step, rather than being advanced one element at a time. Implemented by the
/// index iterators (by re-seeking the underlying scan) — it is what lets
/// [`Intersection`](intersection::Intersection) leapfrog a lagging child over a
/// long run of non-matching positions instead of single-stepping through every
/// one of them.
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
// Test support
// =================================================================================================

// Shared test scaffolding for both combinators (and their composition): the
// self-referential `TestIter` (`From<Intersection<Self, _>>` /
// `From<Union<Self, _>>`) plus `Leaf`, and the
// `and`/`or`/`leaf`/`forward`/`backward` builders. Lives here in the `iter`
// root so the `union`/`intersection` submodule tests can share
// it (`crate::iter::test_util`).
#[cfg(test)]
pub(crate) mod test_util {
    use error_stack::Report;

    use super::{
        Seek,
        intersection::Intersection,
        union::Union,
    };
    use crate::error::{
        Error,
        Result,
    };

    #[derive(Debug)]
    pub(crate) enum TestIter {
        Intersection(Intersection<TestIter, u64, Report<Error>>),
        Union(Union<TestIter, u64, Report<Error>>),
        Leaf(std::vec::IntoIter<Result<u64>>),
    }

    impl From<Intersection<TestIter, u64, Report<Error>>> for TestIter {
        fn from(iter: Intersection<TestIter, u64, Report<Error>>) -> Self {
            Self::Intersection(iter)
        }
    }

    impl From<Union<TestIter, u64, Report<Error>>> for TestIter {
        fn from(iter: Union<TestIter, u64, Report<Error>>) -> Self {
            Self::Union(iter)
        }
    }

    impl Iterator for TestIter {
        type Item = Result<u64>;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Self::Intersection(iter) => iter.next(),
                Self::Union(iter) => iter.next(),
                Self::Leaf(iter) => iter.next(),
            }
        }
    }

    impl DoubleEndedIterator for TestIter {
        fn next_back(&mut self) -> Option<Self::Item> {
            match self {
                Self::Intersection(iter) => iter.next_back(),
                Self::Union(iter) => iter.next_back(),
                Self::Leaf(iter) => iter.next_back(),
            }
        }
    }

    impl Seek<u64> for TestIter {
        fn seek(&mut self, target: u64) {
            match self {
                Self::Intersection(iter) => iter.seek(target),
                Self::Union(iter) => iter.seek(target),
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
                Self::Intersection(iter) => iter.seek_back(target),
                Self::Union(iter) => iter.seek_back(target),
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

    pub(crate) fn leaf<I>(values: I) -> TestIter
    where
        I: IntoIterator<Item = u64>,
    {
        let values = values.into_iter().map(Ok).collect::<Vec<Result<u64>>>();

        TestIter::Leaf(values.into_iter())
    }

    pub(crate) fn leaf_results(values: Vec<Result<u64>>) -> TestIter {
        TestIter::Leaf(values.into_iter())
    }

    pub(crate) fn and<I>(iters: I) -> TestIter
    where
        I: IntoIterator<Item = TestIter>,
    {
        Intersection::<TestIter, u64, Report<Error>>::iter(iters)
    }

    pub(crate) fn or<I>(iters: I) -> TestIter
    where
        I: IntoIterator<Item = TestIter>,
    {
        Union::<TestIter, u64, Report<Error>>::iter(iters)
    }

    pub(crate) fn forward(iter: TestIter) -> Vec<u64> {
        iter.map(Result::unwrap).collect()
    }

    pub(crate) fn backward(iter: TestIter) -> Vec<u64> {
        iter.rev().map(Result::unwrap).collect()
    }
}

// =================================================================================================
// Tests
// =================================================================================================

// Cross-cutting tests (composition of both combinators) and the `Cursor`/`Seek`
// leaf-level behaviour. Combinator-specific tests live with their type, in the
// `intersection` / `union` submodules.
#[cfg(test)]
mod tests {
    use super::{
        Seek,
        test_util::{
            and,
            backward,
            forward,
            leaf,
            or,
        },
    };

    #[test]
    fn and_of_ors_composes() {
        // (a OR b) AND c
        let iter = and([or([leaf([1, 2]), leaf([2, 3, 4])]), leaf([2, 4, 6])]);

        assert_eq!(forward(iter), vec![2, 4]);
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

    // The reverse seek must thread through a nested OR, same as the forward one.
    #[test]
    fn and_of_or_with_sparse_intersects_via_seek_back() {
        let evens = (0..1_000).filter(|n| n % 2 == 0);
        let odds = (0..1_000).filter(|n| n % 2 == 1);
        let iter = and([or([leaf(evens), leaf(odds)]), leaf([7, 250, 999])]);

        assert_eq!(backward(iter), vec![999, 250, 7]);
    }

    // Cursor/Seek leaf-level behaviour (not specific to either combinator).

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

    #[test]
    fn seek_back_positions_leaf_at_last_at_or_before_target() {
        let mut iter = leaf([1, 5, 9, 12]);

        iter.seek_back(8);

        assert_eq!(backward(iter), vec![5, 1]);
    }

    #[test]
    fn seek_back_above_the_head_is_a_no_op() {
        let mut iter = leaf([5, 6, 7]);

        iter.seek_back(100);

        assert_eq!(backward(iter), vec![7, 6, 5]);
    }
}
