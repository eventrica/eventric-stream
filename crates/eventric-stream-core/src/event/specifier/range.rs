use std::ops;

use crate::event::version::Version;

// =================================================================================================
// Range
// =================================================================================================

/// .
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Range {
    /// .
    Range(ops::Range<Version>),
    /// .
    RangeFrom(ops::RangeFrom<Version>),
    /// .
    RangeFull,
    /// .
    RangeTo(ops::RangeTo<Version>),
}

// Into Range

impl From<ops::Range<Version>> for Range {
    fn from(value: ops::Range<Version>) -> Self {
        Self::Range(value)
    }
}

impl From<ops::RangeFrom<Version>> for Range {
    fn from(value: ops::RangeFrom<Version>) -> Self {
        Self::RangeFrom(value)
    }
}

impl From<ops::RangeFull> for Range {
    fn from(_: ops::RangeFull) -> Self {
        Self::RangeFull
    }
}

impl From<ops::RangeTo<Version>> for Range {
    fn from(value: ops::RangeTo<Version>) -> Self {
        Self::RangeTo(value)
    }
}

// From Range

impl From<Range> for ops::Range<Version> {
    fn from(value: Range) -> Self {
        match value {
            Range::Range(range) => range,
            Range::RangeFrom(range) => range.start..Version::MAX,
            Range::RangeFull => Version::MIN..Version::MAX,
            Range::RangeTo(range) => Version::MIN..range.end,
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Tests

#[cfg(test)]
mod tests {
    use std::ops;

    use crate::event::{
        specifier::range::Range,
        version::Version,
    };

    // From<ops::Range<Version>> for Range

    #[test]
    fn from_range_to_range_enum() {
        let range = Version::new(1)..Version::new(5);

        let result: Range = range.into();

        assert!(matches!(result, Range::Range(_)));
    }

    #[test]
    fn from_range_preserves_bounds() {
        let range = Version::new(2)..Version::new(8);

        let result: Range = range.into();

        match result {
            Range::Range(r) => {
                assert_eq!(Version::new(2), r.start);
                assert_eq!(Version::new(8), r.end);
            }
            _ => panic!("Expected Range::Range variant"),
        }
    }

    // From<ops::RangeFrom<Version>> for Range

    #[test]
    fn from_range_from_to_range_enum() {
        let range = Version::new(3)..;

        let result: Range = range.into();

        assert!(matches!(result, Range::RangeFrom(_)));
    }

    #[test]
    fn from_range_from_preserves_start() {
        let range = Version::new(5)..;

        let result: Range = range.into();

        match result {
            Range::RangeFrom(r) => {
                assert_eq!(Version::new(5), r.start);
            }
            _ => panic!("Expected Range::RangeFrom variant"),
        }
    }

    // From<ops::RangeFull> for Range

    #[test]
    fn from_range_full_to_range_enum() {
        let range = ..;

        let result: Range = range.into();

        assert!(matches!(result, Range::RangeFull));
    }

    // From<ops::RangeTo<Version>> for Range

    #[test]
    fn from_range_to_to_range_enum() {
        let range = ..Version::new(5);

        let result: Range = range.into();

        assert!(matches!(result, Range::RangeTo(_)));
    }

    #[test]
    fn from_range_to_preserves_end() {
        let range = ..Version::new(10);

        let result: Range = range.into();

        match result {
            Range::RangeTo(r) => {
                assert_eq!(Version::new(10), r.end);
            }
            _ => panic!("Expected Range::RangeTo variant"),
        }
    }

    // From<Range> for ops::Range<Version>

    #[test]
    fn from_range_enum_to_ops_range_variant() {
        let range = Range::Range(Version::new(1)..Version::new(5));

        let result: ops::Range<Version> = range.into();

        assert_eq!(Version::new(1), result.start);
        assert_eq!(Version::new(5), result.end);
    }

    #[test]
    fn from_range_from_enum_to_ops_range() {
        let range = Range::RangeFrom(Version::new(3)..);

        let result: ops::Range<Version> = range.into();

        assert_eq!(Version::new(3), result.start);
        assert_eq!(Version::MAX, result.end);
    }

    #[test]
    fn from_range_full_enum_to_ops_range() {
        let range = Range::RangeFull;

        let result: ops::Range<Version> = range.into();

        assert_eq!(Version::MIN, result.start);
        assert_eq!(Version::MAX, result.end);
    }

    #[test]
    fn from_range_to_enum_to_ops_range() {
        let range = Range::RangeTo(..Version::new(5));

        let result: ops::Range<Version> = range.into();

        assert_eq!(Version::MIN, result.start);
        assert_eq!(Version::new(5), result.end);
    }

    // Round-trip conversions

    #[test]
    fn round_trip_exclusive_range() {
        let original = Version::new(3)..Version::new(9);

        let as_enum: Range = original.clone().into();
        let back_to_ops: ops::Range<Version> = as_enum.into();

        assert_eq!(original.start, back_to_ops.start);
        assert_eq!(original.end, back_to_ops.end);
    }

    #[test]
    fn round_trip_range_from() {
        let original_start = Version::new(5);

        let as_enum: Range = (original_start..).into();
        let back_to_ops: ops::Range<Version> = as_enum.into();

        assert_eq!(original_start, back_to_ops.start);
        assert_eq!(Version::MAX, back_to_ops.end);
    }

    #[test]
    fn round_trip_range_full() {
        let as_enum: Range = (..).into();
        let back_to_ops: ops::Range<Version> = as_enum.into();

        assert_eq!(Version::MIN, back_to_ops.start);
        assert_eq!(Version::MAX, back_to_ops.end);
    }

    #[test]
    fn round_trip_range_to() {
        let original_end = Version::new(7);

        let as_enum: Range = (..original_end).into();
        let back_to_ops: ops::Range<Version> = as_enum.into();

        assert_eq!(Version::MIN, back_to_ops.start);
        assert_eq!(original_end, back_to_ops.end);
    }

    // Edge cases

    #[test]
    fn range_with_version_min() {
        let range = Version::MIN..Version::new(5);

        let as_enum: Range = range.into();
        let back_to_ops: ops::Range<Version> = as_enum.into();

        assert_eq!(Version::MIN, back_to_ops.start);
        assert_eq!(Version::new(5), back_to_ops.end);
    }

    #[test]
    fn range_with_version_max() {
        let range = Version::new(10)..Version::MAX;

        let as_enum: Range = range.into();
        let back_to_ops: ops::Range<Version> = as_enum.into();

        assert_eq!(Version::new(10), back_to_ops.start);
        assert_eq!(Version::MAX, back_to_ops.end);
    }

    #[test]
    fn range_zero_to_one() {
        let range = Version::new(0)..Version::new(1);

        let as_enum: Range = range.into();
        let back_to_ops: ops::Range<Version> = as_enum.into();

        assert_eq!(Version::new(0), back_to_ops.start);
        assert_eq!(Version::new(1), back_to_ops.end);
    }
}
