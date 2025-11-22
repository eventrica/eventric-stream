use std::{
    cmp::Ordering,
    ops::Bound,
};

use any_range::AnyRange;

use crate::event::version::Version;

// =================================================================================================
// Point
// =================================================================================================

#[derive(Debug, Eq, PartialEq)]
pub enum Point {
    Open(Version),
    Close(Version),
}

impl Point {
    pub fn pair_from_range(range: &AnyRange<Version>) -> [Point; 2] {
        [
            Point::Open(match range.start_bound() {
                Bound::Excluded(version) => *version - 1u8,
                Bound::Included(version) => *version,
                Bound::Unbounded => Version::MIN,
            }),
            Point::Close(match range.end_bound() {
                Bound::Excluded(version) => *version,
                Bound::Included(version) => *version + 1,
                Bound::Unbounded => Version::MAX,
            }),
        ]
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Open(lhs), Self::Close(rhs)) if lhs == rhs => Ordering::Less,
            (Self::Close(lhs), Self::Open(rhs)) if lhs == rhs => Ordering::Greater,
            (Self::Open(lhs) | Self::Close(lhs), Self::Open(rhs) | Self::Close(rhs)) => {
                lhs.cmp(rhs)
            }
        }
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use any_range::AnyRange;

    use crate::{
        event::version::Version,
        stream::query::filter::point::Point,
    };

    // Point Creation

    #[test]
    fn pair_from_range_inclusive_start_exclusive_end() {
        let range = AnyRange::from(Version::new(5)..Version::new(10));
        let [open, close] = Point::pair_from_range(&range);

        assert_eq!(open, Point::Open(Version::new(5)));
        assert_eq!(close, Point::Close(Version::new(10)));
    }

    #[test]
    fn pair_from_range_inclusive_both_ends() {
        let range = AnyRange::from(Version::new(5)..=Version::new(10));
        let [open, close] = Point::pair_from_range(&range);

        assert_eq!(open, Point::Open(Version::new(5)));
        assert_eq!(close, Point::Close(Version::new(11)));
    }

    #[test]
    fn pair_from_range_unbounded_start_inclusive_end() {
        let range = AnyRange::from(..=Version::new(10));
        let [open, close] = Point::pair_from_range(&range);

        assert_eq!(open, Point::Open(Version::MIN));
        assert_eq!(close, Point::Close(Version::new(11)));
    }

    #[test]
    fn pair_from_range_from_unbounded_end() {
        let range = AnyRange::from(Version::new(5)..);
        let [open, close] = Point::pair_from_range(&range);

        assert_eq!(open, Point::Open(Version::new(5)));
        assert_eq!(close, Point::Close(Version::MAX));
    }

    #[test]
    fn pair_from_range_unbounded_start_exclusive_end() {
        let range = AnyRange::from(..Version::new(10));
        let [open, close] = Point::pair_from_range(&range);

        assert_eq!(open, Point::Open(Version::MIN));
        assert_eq!(close, Point::Close(Version::new(10)));
    }

    #[test]
    fn pair_from_range_unbounded_both() {
        let range = AnyRange::<Version>::from(..);
        let [open, close] = Point::pair_from_range(&range);

        assert_eq!(open, Point::Open(Version::MIN));
        assert_eq!(close, Point::Close(Version::MAX));
    }

    #[test]
    fn pair_from_range_single_version_inclusive() {
        let range = AnyRange::from(Version::new(5)..=Version::new(5));
        let [open, close] = Point::pair_from_range(&range);

        assert_eq!(open, Point::Open(Version::new(5)));
        assert_eq!(close, Point::Close(Version::new(6)));
    }

    // Point Ordering

    #[test]
    fn point_ordering_open_vs_close_same_version() {
        let open = Point::Open(Version::new(5));
        let close = Point::Close(Version::new(5));

        assert!(open < close);
        assert!(close > open);
        assert_eq!(open.cmp(&close), Ordering::Less);
        assert_eq!(close.cmp(&open), Ordering::Greater);
    }

    #[test]
    fn point_ordering_open_vs_open_different_versions() {
        let open1 = Point::Open(Version::new(5));
        let open2 = Point::Open(Version::new(10));

        assert!(open1 < open2);
        assert_eq!(open1.cmp(&open2), Ordering::Less);
    }

    #[test]
    fn point_ordering_close_vs_close_different_versions() {
        let close1 = Point::Close(Version::new(5));
        let close2 = Point::Close(Version::new(10));

        assert!(close1 < close2);
        assert_eq!(close1.cmp(&close2), Ordering::Less);
    }

    #[test]
    fn point_ordering_open_vs_close_different_versions() {
        let open = Point::Open(Version::new(5));
        let close = Point::Close(Version::new(10));

        assert!(open < close);
        assert_eq!(open.cmp(&close), Ordering::Less);
    }

    #[test]
    fn point_ordering_close_before_open_different_versions() {
        let close = Point::Close(Version::new(5));
        let open = Point::Open(Version::new(10));

        assert!(close < open);
        assert_eq!(close.cmp(&open), Ordering::Less);
    }

    #[test]
    fn point_ordering_equality() {
        let open1 = Point::Open(Version::new(5));
        let open2 = Point::Open(Version::new(5));
        let close1 = Point::Close(Version::new(5));
        let close2 = Point::Close(Version::new(5));

        assert_eq!(open1.cmp(&open2), Ordering::Equal);
        assert_eq!(close1.cmp(&close2), Ordering::Equal);
    }

    #[test]
    fn point_sorting_mixed() {
        let mut points = vec![
            Point::Close(Version::new(5)),
            Point::Open(Version::new(3)),
            Point::Open(Version::new(5)),
            Point::Close(Version::new(3)),
            Point::Open(Version::new(1)),
        ];

        points.sort();

        assert_eq!(points, vec![
            Point::Open(Version::new(1)),
            Point::Open(Version::new(3)),
            Point::Close(Version::new(3)),
            Point::Open(Version::new(5)),
            Point::Close(Version::new(5)),
        ]);
    }

    #[test]
    fn point_equality() {
        let open1 = Point::Open(Version::new(5));
        let open2 = Point::Open(Version::new(5));
        let close1 = Point::Close(Version::new(5));
        let close2 = Point::Close(Version::new(5));

        assert_eq!(open1, open2);
        assert_eq!(close1, close2);
        assert_ne!(open1, close1);
    }
}
