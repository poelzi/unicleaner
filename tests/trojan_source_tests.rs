use std::path::Path;
use test_case::test_case;
use unicleaner::scanner::file_scanner::scan_file;

#[test_case("tests/fixtures/unicode_attacks/trojan_source/rlo_attack.rs", "RLO character detected" ; "RLO attack")]
#[test_case("tests/fixtures/unicode_attacks/trojan_source/lri_hiding.rs", "LRI character detected" ; "LRI hiding")]
#[test_case("tests/fixtures/unicode_attacks/trojan_source/rli_reorder.rs", "RLI character detected" ; "RLI reorder")]
#[test_case("tests/fixtures/unicode_attacks/trojan_source/fsi_attack.rs", "FSI character detected" ; "FSI attack")]
#[test_case("tests/fixtures/unicode_attacks/trojan_source/pdi_attack.rs", "PDI character detected" ; "PDI attack")]
#[test_case("tests/fixtures/unicode_attacks/trojan_source/combined_bidi.rs", "RLO character detected" ; "Combined bidi")]
fn test_trojan_source_attacks(path: &str, message: &str) {
    let violations = scan_file(Path::new(path)).unwrap();
    assert!(!violations.is_empty());
    assert!(violations.iter().any(|v| v.message.contains(message)));
}
