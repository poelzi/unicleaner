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
    pub fn contains(&self, code_point: u32) -> bool {
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

        assert!(range.contains(0x0370)); // Start boundary
        assert!(range.contains(0x03FF)); // End boundary
        assert!(range.contains(0x0385)); // Middle
        assert!(!range.contains(0x036F)); // Before range
        assert!(!range.contains(0x0400)); // After range
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
}
