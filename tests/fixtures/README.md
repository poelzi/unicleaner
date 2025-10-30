# Test Fixtures

This directory contains test fixtures for Unicode attack detection tests.

## Directory Structure

```
tests/fixtures/unicode_attacks/
├── trojan_source/      # Trojan Source attacks (CVE-2021-42574, CVE-2021-42694)
├── homoglyphs/         # Homoglyph attacks (lookalike characters)
├── zero_width/         # Zero-width and invisible characters
└── normalization/      # Unicode normalization attacks
```

## Trojan Source Fixtures

Located in `trojan_source/`:

- **rlo_attack.rs**: Right-to-Left Override (U+202E) attack
- **lri_hiding.rs**: Left-to-Right Isolate (U+2066) hiding
- **rli_reorder.rs**: Right-to-Left Isolate (U+2067) reordering
- **fsi_attack.rs**: First Strong Isolate (U+2068) attack
- **pdi_attack.rs**: Pop Directional Isolate (U+2069) 
- **combined_bidi.rs**: Multiple bidi controls combined

These fixtures contain bidirectional text control characters that can reorder code display.

## Homoglyph Fixtures

Located in `homoglyphs/`:

- **cyrillic.rs**: Cyrillic letters that look like Latin (а, е, о)
- **greek.rs**: Greek letters that look like Latin (ο, α, ν, ρ)
- **math_alphanumeric.rs**: Mathematical bold/italic variants (𝐚, 𝑎, 𝒂)
- **fullwidth.rs**: Fullwidth Latin characters (ａ, ｂ, ｃ)
- **identifiers.rs**: Confusable identifier patterns
- **mixed_scripts.rs**: Mixed Latin/Cyrillic/Greek in identifiers

Homoglyphs are characters from different scripts that look visually identical or very similar.

## Zero-Width Fixtures

Located in `zero_width/`:

- **zwsp.rs**: Zero-Width Space (U+200B)
- **zwnj.rs**: Zero-Width Non-Joiner (U+200C)
- **zwj.rs**: Zero-Width Joiner (U+200D)
- **bom.rs**: Byte Order Mark / ZWNBSP (U+FEFF) in middle of file
- **combining.rs**: Excessive combining diacritical marks
- **separators.rs**: Invisible separator characters

These characters are invisible but can hide code or create confusion.

## Normalization Fixtures

Located in `normalization/`:

- **nfc_nfd.rs**: NFC vs NFD normalization confusion (café vs café)
- **nfkc_nfkd.rs**: Compatibility normalization (fullwidth, ligatures)

Unicode has multiple normalization forms that can make identically-looking text have different byte representations.

## Usage in Tests

These fixtures are used by integration tests in `tests/integration/`:

```rust
use std::path::Path;
use unicleaner::scanner::file_scanner::scan_file;

#[test]
fn test_trojan_source_detection() {
    let path = Path::new("tests/fixtures/unicode_attacks/trojan_source/rlo_attack.rs");
    let violations = scan_file(path).expect("Failed to scan");
    assert!(!violations.is_empty(), "Should detect RLO attack");
}
```

## Adding New Fixtures

When adding new attack patterns:

1. Create the fixture file in the appropriate subdirectory
2. Include comments explaining the attack
3. Use actual malicious Unicode characters (not descriptions)
4. Add corresponding integration tests
5. Document the fixture in this README

## Security Note

These files contain actual malicious Unicode characters for testing purposes. They should:

- Never be deployed to production
- Be scanned with caution in editors (may display incorrectly)
- Be included in `.gitignore` for sensitive repositories

## References

- [Trojan Source: CVE-2021-42574](https://nvd.nist.gov/vuln/detail/CVE-2021-42574)
- [Unicode Security Mechanisms (UTS #39)](https://www.unicode.org/reports/tr39/)
- [Unicode Technical Report #36](https://www.unicode.org/reports/tr36/)
