use any_range::AnyRange;

use crate::{
    event::version::Version,
    stream::query::filter::point::Point,
};

// =================================================================================================
// Algorithm
// =================================================================================================

// Normalize Ranges

/// The [`normalize_version_ranges`] function applies a simple
/// sweeping-line-like algorithm to a vector of version ranges to produce an
/// ordered and normalized vector of version ranges (where normalized means that
/// adjacent/overlapping version ranges are merged, and nested version ranges
/// are subsumed).
pub fn normalize_version_ranges(ranges: &[AnyRange<Version>]) -> Vec<AnyRange<Version>> {
    let mut points = ranges
        .iter()
        .flat_map(Point::pair_from_range)
        .collect::<Vec<_>>();

    points.sort();

    let mut depth = 0u8;
    let mut open = None;
    let mut ranges = Vec::new();

    for point in points {
        match point {
            Point::Open(version) => {
                depth += 1;
                open.get_or_insert(version);
            }
            Point::Close(version) => {
                depth -= 1;

                if depth == 0 {
                    ranges.push((open.take().expect("opening bound")..version).into());
                }
            }
        }
    }

    ranges
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use std::ops::Bound;

    use any_range::AnyRange;

    use crate::{
        event::Version,
        stream::query::filter::algorithm::normalize_version_ranges,
    };

    // Normalize Version Ranges

    #[test]
    fn normalize_version_ranges_single_range() {
        let ranges = vec![AnyRange::from(Version::new(5)..Version::new(10))];
        let result = normalize_version_ranges(&ranges);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::new(5)));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::new(10)));
    }

    #[test]
    fn normalize_version_ranges_non_overlapping_ranges() {
        let ranges = vec![
            AnyRange::from(Version::new(1)..Version::new(3)),
            AnyRange::from(Version::new(5)..Version::new(7)),
            AnyRange::from(Version::new(10)..Version::new(15)),
        ];
        let result = normalize_version_ranges(&ranges);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::new(1)));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::new(3)));
        assert_eq!(result[1].start_bound(), Bound::Included(&Version::new(5)));
        assert_eq!(result[1].end_bound(), Bound::Excluded(&Version::new(7)));
        assert_eq!(result[2].start_bound(), Bound::Included(&Version::new(10)));
        assert_eq!(result[2].end_bound(), Bound::Excluded(&Version::new(15)));
    }

    #[test]
    fn normalize_version_ranges_overlapping_ranges() {
        let ranges = vec![
            AnyRange::from(Version::new(1)..Version::new(5)),
            AnyRange::from(Version::new(3)..Version::new(7)),
        ];
        let result = normalize_version_ranges(&ranges);

        // Should merge into one range [1, 7)
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::new(1)));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::new(7)));
    }

    #[test]
    fn normalize_version_ranges_adjacent_ranges_exclusive() {
        let ranges = vec![
            AnyRange::from(Version::new(1)..Version::new(5)),
            AnyRange::from(Version::new(5)..Version::new(10)),
        ];
        let result = normalize_version_ranges(&ranges);

        // Adjacent ranges with exclusive bounds should merge
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::new(1)));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::new(10)));
    }

    #[test]
    fn normalize_version_ranges_adjacent_ranges_inclusive() {
        let ranges = vec![
            AnyRange::from(Version::new(1)..=Version::new(4)),
            AnyRange::from(Version::new(5)..=Version::new(10)),
        ];
        let result = normalize_version_ranges(&ranges);

        // [1, 4] becomes [1, 5) and [5, 10] becomes [5, 11)
        // These should merge into [1, 11)
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::new(1)));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::new(11)));
    }

    #[test]
    fn normalize_version_ranges_nested_ranges() {
        let ranges = vec![
            AnyRange::from(Version::new(1)..Version::new(10)),
            AnyRange::from(Version::new(3)..Version::new(7)),
        ];
        let result = normalize_version_ranges(&ranges);

        // Nested range should be absorbed
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::new(1)));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::new(10)));
    }

    #[test]
    fn normalize_version_ranges_multiple_overlapping_ranges() {
        let ranges = vec![
            AnyRange::from(Version::new(1)..Version::new(5)),
            AnyRange::from(Version::new(3)..Version::new(8)),
            AnyRange::from(Version::new(6)..Version::new(10)),
            AnyRange::from(Version::new(15)..Version::new(20)),
        ];
        let result = normalize_version_ranges(&ranges);

        // First three should merge into [1, 10), last remains [15, 20)
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::new(1)));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::new(10)));
        assert_eq!(result[1].start_bound(), Bound::Included(&Version::new(15)));
        assert_eq!(result[1].end_bound(), Bound::Excluded(&Version::new(20)));
    }

    #[test]
    fn normalize_version_ranges_empty_input() {
        let ranges: Vec<AnyRange<Version>> = vec![];
        let result = normalize_version_ranges(&ranges);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn normalize_version_ranges_identical_ranges() {
        let ranges = vec![
            AnyRange::from(Version::new(5)..Version::new(10)),
            AnyRange::from(Version::new(5)..Version::new(10)),
            AnyRange::from(Version::new(5)..Version::new(10)),
        ];
        let result = normalize_version_ranges(&ranges);

        // All identical ranges should collapse to one
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::new(5)));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::new(10)));
    }

    #[test]
    fn normalize_version_ranges_complex_overlaps() {
        let ranges = vec![
            AnyRange::from(Version::new(1)..Version::new(3)),
            AnyRange::from(Version::new(2)..Version::new(5)),
            AnyRange::from(Version::new(4)..Version::new(6)),
            AnyRange::from(Version::new(8)..Version::new(10)),
            AnyRange::from(Version::new(9)..Version::new(12)),
        ];
        let result = normalize_version_ranges(&ranges);

        // Should produce [1, 6) and [8, 12)
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::new(1)));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::new(6)));
        assert_eq!(result[1].start_bound(), Bound::Included(&Version::new(8)));
        assert_eq!(result[1].end_bound(), Bound::Excluded(&Version::new(12)));
    }

    #[test]
    fn normalize_version_ranges_gap_between_ranges() {
        let ranges = vec![
            AnyRange::from(Version::new(1)..Version::new(3)),
            AnyRange::from(Version::new(5)..Version::new(8)),
        ];
        let result = normalize_version_ranges(&ranges);

        // Gap at 3-5 should keep them separate
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::new(1)));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::new(3)));
        assert_eq!(result[1].start_bound(), Bound::Included(&Version::new(5)));
        assert_eq!(result[1].end_bound(), Bound::Excluded(&Version::new(8)));
    }

    #[test]
    fn normalize_version_ranges_full_range_unbounded() {
        let ranges = vec![AnyRange::<Version>::from(..)];
        let result = normalize_version_ranges(&ranges);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::MIN));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::MAX));
    }

    #[test]
    fn normalize_version_ranges_mixed_bound_types() {
        let ranges = vec![
            AnyRange::from(Version::new(1)..Version::new(5)),
            AnyRange::from(Version::new(4)..=Version::new(8)),
            AnyRange::from(Version::new(7)..Version::new(12)),
        ];
        let result = normalize_version_ranges(&ranges);

        // All should merge into one continuous range
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::new(1)));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::new(12)));
    }

    #[test]
    fn normalize_version_ranges_single_point_range() {
        let ranges = vec![AnyRange::from(Version::new(5)..=Version::new(5))];
        let result = normalize_version_ranges(&ranges);

        // Single point [5, 5] becomes [5, 6)
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::new(5)));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::new(6)));
    }

    #[test]
    fn normalize_version_ranges_touching_exclusive_inclusive() {
        let ranges = vec![
            AnyRange::from(Version::new(1)..Version::new(5)),
            AnyRange::from(Version::new(5)..=Version::new(10)),
        ];
        let result = normalize_version_ranges(&ranges);

        // [1, 5) and [5, 10] = [5, 11) should merge
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_bound(), Bound::Included(&Version::new(1)));
        assert_eq!(result[0].end_bound(), Bound::Excluded(&Version::new(11)));
    }
}
