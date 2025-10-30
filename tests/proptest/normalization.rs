// Property-based tests for character normalization invariants (T043)

use proptest::prelude::*;
use unicode_normalization::UnicodeNormalization;

// Property: NFC(NFC(s)) = NFC(s) - idempotent
proptest! {
    #[test]
    fn nfc_is_idempotent(s in "\\PC{0,100}") {
        let nfc1: String = s.nfc().collect();
        let nfc2: String = nfc1.nfc().collect();

        prop_assert_eq!(nfc1, nfc2, "NFC should be idempotent");
    }
}

// Property: NFD(NFD(s)) = NFD(s) - idempotent
proptest! {
    #[test]
    fn nfd_is_idempotent(s in "\\PC{0,100}") {
        let nfd1: String = s.nfd().collect();
        let nfd2: String = nfd1.nfd().collect();

        prop_assert_eq!(nfd1, nfd2, "NFD should be idempotent");
    }
}

// Property: NFC and NFD should be reversible
proptest! {
    #[test]
    fn nfc_nfd_roundtrip(s in "\\PC{0,100}") {
        let nfc: String = s.nfc().collect();
        let nfd: String = s.nfd().collect();

        // NFC of NFD should equal NFC of original
        let nfc_of_nfd: String = nfd.nfc().collect();
        prop_assert_eq!(nfc, nfc_of_nfd, "NFC(NFD(s)) should equal NFC(s)");

        // NFD of NFC should equal NFD of original
        let nfd_of_nfc: String = nfc.nfd().collect();
        prop_assert_eq!(nfd, nfd_of_nfc, "NFD(NFC(s)) should equal NFD(s)");
    }
}

// Property: Normalization preserves string length or makes it longer (NFD)
proptest! {
    #[test]
    fn nfd_may_lengthen(s in "\\PC{0,50}") {
        let nfd: String = s.nfd().collect();

        // NFD can make strings longer (decomposition) but not shorter
        prop_assert!(
            nfd.chars().count() >= s.chars().count(),
            "NFD should not shorten string: {} chars -> {} chars",
            s.chars().count(),
            nfd.chars().count()
        );
    }
}

// Property: Scanner should handle both normalized and denormalized forms
proptest! {
    #[test]
    fn scanner_handles_normalization_forms(s in "\\PC{0,50}") {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let nfc: String = s.nfc().collect();
        let nfd: String = s.nfd().collect();

        // Create temp files with different normalizations
        let mut temp_nfc = NamedTempFile::new().unwrap();
        let mut temp_nfd = NamedTempFile::new().unwrap();

        write!(temp_nfc, "{}", nfc).unwrap();
        write!(temp_nfd, "{}", nfd).unwrap();

        temp_nfc.flush().unwrap();
        temp_nfd.flush().unwrap();

        // Both should scan without panicking
        let result_nfc = std::panic::catch_unwind(|| {
            unicleaner::scanner::file_scanner::scan_file(temp_nfc.path())
        });

        let result_nfd = std::panic::catch_unwind(|| {
            unicleaner::scanner::file_scanner::scan_file(temp_nfd.path())
        });

        prop_assert!(result_nfc.is_ok(), "Scanner panicked on NFC input");
        prop_assert!(result_nfd.is_ok(), "Scanner panicked on NFD input");
    }
}

// Property: Combining marks are preserved under normalization
proptest! {
    #[test]
    fn combining_marks_preserved(base in 'a'..='z', combining in '\u{0300}'..='\u{036F}') {
        let s = format!("{}{}", base, combining);
        let nfd: String = s.nfd().collect();

        // NFD should still contain the combining mark
        prop_assert!(
            nfd.contains(combining),
            "NFD should preserve combining mark U+{:04X}", combining as u32
        );
    }
}
