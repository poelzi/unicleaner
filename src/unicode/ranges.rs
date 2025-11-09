//! Unicode range definitions and operations

/// Represents a contiguous range of Unicode code points
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicodeRange {
    pub start: u32,
    pub end: u32,
    pub description: Option<String>,
}

impl UnicodeRange {
    /// Create a new Unicode range
    pub fn new(start: u32, end: u32) -> Self {
        assert!(start <= end, "start must be <= end");
        assert!(start <= 0x10FFFF, "start must be valid Unicode");
        assert!(end <= 0x10FFFF, "end must be valid Unicode");

        Self {
            start,
            end,
            description: None,
        }
    }

    /// Create a range with a description
    pub fn with_description(start: u32, end: u32, description: String) -> Self {
        let mut range = Self::new(start, end);
        range.description = Some(description);
        range
    }

    /// Check if a code point is within this range
    pub fn contains(&self, code_point: impl Into<u32>) -> bool {
        let code_point = code_point.into();
        code_point >= self.start && code_point <= self.end
    }

    /// Check if this range intersects with another
    pub fn intersects(&self, other: &UnicodeRange) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    /// Merge two adjacent or overlapping ranges
    pub fn merge(&self, other: &UnicodeRange) -> Option<UnicodeRange> {
        if self.intersects(other) || self.end + 1 == other.start || other.end + 1 == self.start {
            Some(UnicodeRange::new(
                self.start.min(other.start),
                self.end.max(other.end),
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_range_contains() {
        let range = UnicodeRange::new(0x0370, 0x03FF); // Greek range

        assert!(range.contains(0x0370u32)); // Start boundary
        assert!(range.contains(0x03FFu32)); // End boundary
        assert!(range.contains(0x0385u32)); // Middle
        assert!(!range.contains(0x036Fu32)); // Before range
        assert!(!range.contains(0x0400u32)); // After range
    }

    #[test]
    fn test_unicode_range_intersects() {
        let range1 = UnicodeRange::new(0x0370, 0x03FF);
        let range2 = UnicodeRange::new(0x03E0, 0x0420); // Overlaps
        let range3 = UnicodeRange::new(0x0400, 0x04FF); // Adjacent
        let range4 = UnicodeRange::new(0x0500, 0x05FF); // Separate

        assert!(range1.intersects(&range2));
        assert!(!range1.intersects(&range3));
        assert!(!range1.intersects(&range4));
    }

    #[test]
    fn test_unicode_range_merge() {
        let range1 = UnicodeRange::new(0x0370, 0x03FF);
        let range2 = UnicodeRange::new(0x03E0, 0x0420);

        let merged = range1.merge(&range2).unwrap();
        assert_eq!(merged.start, 0x0370);
        assert_eq!(merged.end, 0x0420);
    }

    #[test]
    #[should_panic(expected = "start must be <= end")]
    fn test_invalid_range_panics() {
        UnicodeRange::new(0x0400, 0x0370); // Start > end
    }

    #[test]
    fn test_with_description() {
        let range = UnicodeRange::with_description(0x0370, 0x03FF, "Greek and Coptic".to_string());

        assert_eq!(range.start, 0x0370);
        assert_eq!(range.end, 0x03FF);
        assert_eq!(range.description, Some("Greek and Coptic".to_string()));
    }

    #[test]
    fn test_merge_adjacent_ranges() {
        let range1 = UnicodeRange::new(0x0370, 0x03FF);
        let range2 = UnicodeRange::new(0x0400, 0x04FF); // Adjacent (start = end + 1)

        let merged = range1.merge(&range2);
        assert!(merged.is_some(), "Adjacent ranges should merge");

        if let Some(m) = merged {
            assert_eq!(m.start, 0x0370);
            assert_eq!(m.end, 0x04FF);
        }
    }

    #[test]
    fn test_merge_overlapping_ranges() {
        let range1 = UnicodeRange::new(0x0370, 0x0400);
        let range2 = UnicodeRange::new(0x03E0, 0x0420);

        let merged = range1.merge(&range2);
        assert!(merged.is_some());

        if let Some(m) = merged {
            assert_eq!(m.start, 0x0370);
            assert_eq!(m.end, 0x0420);
        }
    }

    #[test]
    fn test_merge_separate_ranges() {
        let range1 = UnicodeRange::new(0x0370, 0x03FF);
        let range2 = UnicodeRange::new(0x0500, 0x05FF); // Gap between them

        let merged = range1.merge(&range2);
        assert!(merged.is_none(), "Separate ranges should not merge");
    }

    #[test]
    fn test_intersects_self() {
        let range = UnicodeRange::new(0x0370, 0x03FF);
        assert!(
            range.intersects(&range),
            "Range should intersect with itself"
        );
    }

    #[test]
    fn test_intersects_reversed() {
        let range1 = UnicodeRange::new(0x0370, 0x03FF);
        let range2 = UnicodeRange::new(0x03E0, 0x0420);

        assert!(range1.intersects(&range2));
        assert!(
            range2.intersects(&range1),
            "Intersection should be symmetric"
        );
    }

    #[test]
    fn test_boundary_values() {
        let range = UnicodeRange::new(0x0000, 0x10FFFF); // Full Unicode range

        assert!(range.contains(0x0000u32));
        assert!(range.contains(0x10FFFFu32));
        assert!(range.contains(0x0080u32));
    }

    #[test]
    #[should_panic(expected = "start must be valid Unicode")]
    fn test_invalid_start_panics() {
        UnicodeRange::new(0x110000, 0x110000); // Beyond Unicode max
    }

    #[test]
    #[should_panic(expected = "end must be valid Unicode")]
    fn test_invalid_end_panics() {
        UnicodeRange::new(0x0000, 0x110000); // End beyond Unicode max
    }
}
