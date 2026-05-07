#![no_main]

use libfuzzer_sys::fuzz_target;
use unicleaner::cleaner::{CleanPolicy, clean};

fuzz_target!(|data: &[u8]| {
    // Only feed valid UTF-8 — `clean` operates on `&str`.
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };

    // Strict policy: covers the strip path.
    let r = clean(text, &CleanPolicy::strict());
    assert!(r.output.len() <= text.len());
    if !r.modified {
        assert_eq!(
            r.output.as_ref(),
            text,
            "unmodified output must equal input"
        );
        assert!(
            r.violations.is_empty(),
            "unmodified output must have no violations"
        );
    }

    // Lossy policy: replacement char (3 bytes) may grow the output.
    let r = clean(text, &CleanPolicy::lossy());
    if !r.modified {
        assert_eq!(r.output.as_ref(), text);
    }

    // Report-only: must never mutate.
    let r = clean(text, &CleanPolicy::report_only());
    assert!(!r.modified, "report_only must not mutate");
    assert_eq!(r.output.as_ref(), text);

    // Strict + NFC: post-NFC output must itself be in NFC form.
    let r = clean(text, &CleanPolicy::strict().with_nfc(true));
    use unicode_normalization::is_nfc;
    assert!(
        is_nfc(r.output.as_ref()),
        "NFC pass must produce NFC output"
    );
});
