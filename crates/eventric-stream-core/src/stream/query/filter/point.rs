use std::{
    cmp::Ordering,
    ops::Range,
};

use crate::event::version::Version;

// =================================================================================================
// Point
// =================================================================================================

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq)]
pub enum Point {
    Open(Version),
    Close(Version),
}

impl Point {
    #[allow(dead_code)]
    pub fn pair_from_range(range: &Range<Version>) -> [Point; 2] {
        [Point::Open(range.start), Point::Close(range.end)]
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

    use crate::{
        event::version::Version,
        stream::query::filter::point::Point,
    };

    // pair_from_range

    #[test]
    fn pair_from_range_creates_open_and_close_points() {
        let range = Version::new(5)..Version::new(10);

        let [open, close] = Point::pair_from_range(&range);

        assert_eq!(Point::Open(Version::new(5)), open);
        assert_eq!(Point::Close(Version::new(10)), close);
    }

    #[test]
    fn pair_from_range_preserves_range_bounds() {
        let range = Version::new(0)..Version::new(1);

        let [open, close] = Point::pair_from_range(&range);

        assert_eq!(Point::Open(Version::new(0)), open);
        assert_eq!(Point::Close(Version::new(1)), close);
    }

    #[test]
    fn pair_from_range_with_version_max() {
        let range = Version::new(100)..Version::MAX;

        let [open, close] = Point::pair_from_range(&range);

        assert_eq!(Point::Open(Version::new(100)), open);
        assert_eq!(Point::Close(Version::MAX), close);
    }

    // Ord implementation - Equal versions

    #[test]
    fn open_is_less_than_close_at_same_version() {
        let open = Point::Open(Version::new(5));
        let close = Point::Close(Version::new(5));

        assert_eq!(Ordering::Less, open.cmp(&close));
    }

    #[test]
    fn close_is_greater_than_open_at_same_version() {
        let close = Point::Close(Version::new(5));
        let open = Point::Open(Version::new(5));

        assert_eq!(Ordering::Greater, close.cmp(&open));
    }

    #[test]
    fn open_equals_open_at_same_version() {
        let open1 = Point::Open(Version::new(5));
        let open2 = Point::Open(Version::new(5));

        assert_eq!(Ordering::Equal, open1.cmp(&open2));
    }

    #[test]
    fn close_equals_close_at_same_version() {
        let close1 = Point::Close(Version::new(5));
        let close2 = Point::Close(Version::new(5));

        assert_eq!(Ordering::Equal, close1.cmp(&close2));
    }

    // Ord implementation - Different versions

    #[test]
    fn open_points_compared_by_version() {
        let open_lower = Point::Open(Version::new(3));
        let open_higher = Point::Open(Version::new(7));

        assert_eq!(Ordering::Less, open_lower.cmp(&open_higher));
        assert_eq!(Ordering::Greater, open_higher.cmp(&open_lower));
    }

    #[test]
    fn close_points_compared_by_version() {
        let close_lower = Point::Close(Version::new(3));
        let close_higher = Point::Close(Version::new(7));

        assert_eq!(Ordering::Less, close_lower.cmp(&close_higher));
        assert_eq!(Ordering::Greater, close_higher.cmp(&close_lower));
    }

    #[test]
    fn open_and_close_different_versions_compared_by_version() {
        let open_lower = Point::Open(Version::new(3));
        let close_higher = Point::Close(Version::new(7));

        assert_eq!(Ordering::Less, open_lower.cmp(&close_higher));
        assert_eq!(Ordering::Greater, close_higher.cmp(&open_lower));
    }

    #[test]
    fn close_and_open_different_versions_compared_by_version() {
        let close_lower = Point::Close(Version::new(3));
        let open_higher = Point::Open(Version::new(7));

        assert_eq!(Ordering::Less, close_lower.cmp(&open_higher));
        assert_eq!(Ordering::Greater, open_higher.cmp(&close_lower));
    }

    // PartialOrd

    #[test]
    fn partial_cmp_returns_same_as_cmp() {
        let open = Point::Open(Version::new(5));
        let close = Point::Close(Version::new(5));

        assert_eq!(Some(Ordering::Less), open.partial_cmp(&close));
        assert_eq!(open.cmp(&close), open.partial_cmp(&close).unwrap());
    }

    // Eq and PartialEq

    #[test]
    fn open_points_equal_when_versions_equal() {
        let open1 = Point::Open(Version::new(5));
        let open2 = Point::Open(Version::new(5));

        assert_eq!(open1, open2);
    }

    #[test]
    fn close_points_equal_when_versions_equal() {
        let close1 = Point::Close(Version::new(5));
        let close2 = Point::Close(Version::new(5));

        assert_eq!(close1, close2);
    }

    #[test]
    fn open_and_close_not_equal_even_with_same_version() {
        let open = Point::Open(Version::new(5));
        let close = Point::Close(Version::new(5));

        assert_ne!(open, close);
    }

    #[test]
    fn points_not_equal_when_versions_differ() {
        let open1 = Point::Open(Version::new(3));
        let open2 = Point::Open(Version::new(7));

        assert_ne!(open1, open2);
    }

    // Sorting behavior

    #[test]
    fn points_sort_by_version_then_type() {
        let mut points = vec![
            Point::Close(Version::new(5)),
            Point::Open(Version::new(3)),
            Point::Open(Version::new(5)),
            Point::Close(Version::new(3)),
            Point::Open(Version::new(7)),
        ];

        points.sort();

        assert_eq!(
            vec![
                Point::Open(Version::new(3)),
                Point::Close(Version::new(3)),
                Point::Open(Version::new(5)),
                Point::Close(Version::new(5)),
                Point::Open(Version::new(7)),
            ],
            points
        );
    }

    #[test]
    fn adjacent_ranges_sort_correctly() {
        let range1 = Version::new(1)..Version::new(5);
        let range2 = Version::new(5)..Version::new(10);

        let mut points = Vec::new();
        points.extend(Point::pair_from_range(&range1));
        points.extend(Point::pair_from_range(&range2));
        points.sort();

        // Open(5) should come before Close(5) due to sorting rules
        assert_eq!(
            vec![
                Point::Open(Version::new(1)),
                Point::Open(Version::new(5)),
                Point::Close(Version::new(5)),
                Point::Close(Version::new(10)),
            ],
            points
        );
    }
}
