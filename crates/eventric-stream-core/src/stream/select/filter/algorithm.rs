use std::ops::Range;

use crate::{
    event::version::Version,
    stream::select::filter::point::Point,
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
#[allow(dead_code)]
pub fn normalize_version_ranges(ranges: &[Range<Version>]) -> Vec<Range<Version>> {
    if ranges.is_empty() {
        return Vec::new();
    }

    let mut points = Vec::with_capacity(ranges.len() * 2);

    points.extend(ranges.iter().flat_map(Point::pair_from_range));
    points.sort();

    let mut depth = 0usize;
    let mut range_start = None;
    let mut normalized = Vec::with_capacity(ranges.len());

    for point in points {
        match point {
            Point::Open(version) => {
                depth += 1;

                if depth == 1 {
                    range_start = Some(version);
                }
            }
            Point::Close(version) => {
                depth -= 1;

                if depth == 0 {
                    normalized.push(range_start.take().unwrap()..version);
                }
            }
        }
    }

    normalized
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use crate::{
        event::version::Version,
        stream::select::filter::algorithm::normalize_version_ranges,
    };

    // Empty and single ranges

    #[test]
    fn empty_input_returns_empty_output() {
        let ranges = [];

        let result = normalize_version_ranges(&ranges);

        assert!(result.is_empty());
    }

    #[test]
    fn single_range_returns_same_range() {
        let ranges = [Version::new(5)..Version::new(10)];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(1, result.len());
        assert_eq!(Version::new(5), result[0].start);
        assert_eq!(Version::new(10), result[0].end);
    }

    #[test]
    fn single_range_at_boundaries() {
        let ranges = [Version::MIN..Version::MAX];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(1, result.len());
        assert_eq!(Version::MIN, result[0].start);
        assert_eq!(Version::MAX, result[0].end);
    }

    // Non-overlapping ranges

    #[test]
    fn non_overlapping_ranges_remain_separate() {
        let ranges = [
            Version::new(1)..Version::new(5),
            Version::new(10)..Version::new(15),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(2, result.len());
        assert_eq!(Version::new(1), result[0].start);
        assert_eq!(Version::new(5), result[0].end);
        assert_eq!(Version::new(10), result[1].start);
        assert_eq!(Version::new(15), result[1].end);
    }

    #[test]
    fn three_non_overlapping_ranges_remain_separate() {
        let ranges = [
            Version::new(1)..Version::new(3),
            Version::new(10)..Version::new(15),
            Version::new(20)..Version::new(25),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(3, result.len());
        assert_eq!(Version::new(1)..Version::new(3), result[0]);
        assert_eq!(Version::new(10)..Version::new(15), result[1]);
        assert_eq!(Version::new(20)..Version::new(25), result[2]);
    }

    // Adjacent ranges (should merge)

    #[test]
    fn adjacent_ranges_merge() {
        let ranges = [
            Version::new(1)..Version::new(5),
            Version::new(5)..Version::new(10),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(1, result.len());
        assert_eq!(Version::new(1), result[0].start);
        assert_eq!(Version::new(10), result[0].end);
    }

    #[test]
    fn three_adjacent_ranges_merge() {
        let ranges = [
            Version::new(1)..Version::new(5),
            Version::new(5)..Version::new(10),
            Version::new(10)..Version::new(15),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(1, result.len());
        assert_eq!(Version::new(1)..Version::new(15), result[0]);
    }

    // Overlapping ranges (should merge)

    #[test]
    fn overlapping_ranges_merge() {
        let ranges = [
            Version::new(1)..Version::new(10),
            Version::new(5)..Version::new(15),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(1, result.len());
        assert_eq!(Version::new(1), result[0].start);
        assert_eq!(Version::new(15), result[0].end);
    }

    #[test]
    fn multiple_overlapping_ranges_merge() {
        let ranges = [
            Version::new(1)..Version::new(5),
            Version::new(3)..Version::new(7),
            Version::new(6)..Version::new(10),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(1, result.len());
        assert_eq!(Version::new(1)..Version::new(10), result[0]);
    }

    // Nested ranges (inner subsumed by outer)

    #[test]
    fn nested_range_subsumed_by_outer() {
        let ranges = [
            Version::new(1)..Version::new(20),
            Version::new(5)..Version::new(10),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(1, result.len());
        assert_eq!(Version::new(1), result[0].start);
        assert_eq!(Version::new(20), result[0].end);
    }

    #[test]
    fn multiple_nested_ranges_subsumed() {
        let ranges = [
            Version::new(1)..Version::new(100),
            Version::new(10)..Version::new(20),
            Version::new(30)..Version::new(40),
            Version::new(50)..Version::new(60),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(1, result.len());
        assert_eq!(Version::new(1)..Version::new(100), result[0]);
    }

    // Complex scenarios

    #[test]
    fn mix_of_overlapping_and_separate_ranges() {
        let ranges = [
            Version::new(1)..Version::new(5),
            Version::new(3)..Version::new(8),
            Version::new(15)..Version::new(20),
            Version::new(18)..Version::new(25),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(2, result.len());
        assert_eq!(Version::new(1)..Version::new(8), result[0]);
        assert_eq!(Version::new(15)..Version::new(25), result[1]);
    }

    #[test]
    fn complex_with_adjacent_overlapping_and_nested() {
        let ranges = [
            Version::new(1)..Version::new(5),
            Version::new(5)..Version::new(10), // Adjacent to first
            Version::new(7)..Version::new(12), // Overlaps second
            Version::new(20)..Version::new(30),
            Version::new(22)..Version::new(25), // Nested in fourth
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(2, result.len());
        assert_eq!(Version::new(1)..Version::new(12), result[0]);
        assert_eq!(Version::new(20)..Version::new(30), result[1]);
    }

    // Unsorted input

    #[test]
    fn unsorted_ranges_are_normalized() {
        let ranges = [
            Version::new(20)..Version::new(25),
            Version::new(1)..Version::new(5),
            Version::new(10)..Version::new(15),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(3, result.len());
        assert_eq!(Version::new(1)..Version::new(5), result[0]);
        assert_eq!(Version::new(10)..Version::new(15), result[1]);
        assert_eq!(Version::new(20)..Version::new(25), result[2]);
    }

    #[test]
    fn unsorted_overlapping_ranges_merge_correctly() {
        let ranges = [
            Version::new(10)..Version::new(20),
            Version::new(1)..Version::new(15),
            Version::new(18)..Version::new(25),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(1, result.len());
        assert_eq!(Version::new(1)..Version::new(25), result[0]);
    }

    // Edge cases

    #[test]
    fn identical_ranges_merge_to_one() {
        let ranges = [
            Version::new(5)..Version::new(10),
            Version::new(5)..Version::new(10),
            Version::new(5)..Version::new(10),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(1, result.len());
        assert_eq!(Version::new(5)..Version::new(10), result[0]);
    }

    #[test]
    fn ranges_with_version_min() {
        let ranges = [
            Version::MIN..Version::new(10),
            Version::new(5)..Version::new(15),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(1, result.len());
        assert_eq!(Version::MIN, result[0].start);
        assert_eq!(Version::new(15), result[0].end);
    }

    #[test]
    fn ranges_with_version_max() {
        let ranges = [
            Version::new(10)..Version::new(20),
            Version::new(15)..Version::MAX,
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(1, result.len());
        assert_eq!(Version::new(10), result[0].start);
        assert_eq!(Version::MAX, result[0].end);
    }

    #[test]
    fn single_version_ranges() {
        let ranges = [
            Version::new(1)..Version::new(2),
            Version::new(3)..Version::new(4),
            Version::new(5)..Version::new(6),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(3, result.len());
        assert_eq!(Version::new(1)..Version::new(2), result[0]);
        assert_eq!(Version::new(3)..Version::new(4), result[1]);
        assert_eq!(Version::new(5)..Version::new(6), result[2]);
    }

    // Already normalized input

    #[test]
    fn already_normalized_ranges_unchanged() {
        let ranges = [
            Version::new(1)..Version::new(5),
            Version::new(10)..Version::new(15),
            Version::new(20)..Version::new(25),
        ];

        let result = normalize_version_ranges(&ranges);

        assert_eq!(ranges.len(), result.len());
        for (original, normalized) in ranges.iter().zip(result.iter()) {
            assert_eq!(original.start, normalized.start);
            assert_eq!(original.end, normalized.end);
        }
    }
}
