# Property-Based Testing

This directory contains property-based tests using [proptest](https://github.com/proptest-rs/proptest).

## What is Property-Based Testing?

Instead of writing individual test cases, property-based tests define **properties** that should hold for all inputs. The framework then generates hundreds or thousands of random inputs to verify these properties.

## Test Files

### unicode_categories.rs
Tests invariants about Unicode character categorization:
- Bidi controls are in expected ranges
- Zero-width chars are marked as invisible
- ASCII never flagged as homoglyph risks
- Category functions never panic

### unicode_ranges.rs
Tests Unicode range boundary conditions:
- Range boundaries are correctly included/excluded
- Range intersection is symmetric
- Characters belong to exactly one category
- Surrogate ranges handled correctly

### normalization.rs
Tests Unicode normalization invariants:
- NFC and NFD are idempotent
- Normalization forms are reversible
- Scanner handles both normalized and denormalized text
- Combining marks are preserved

### bidi_properties.rs
Tests bidirectional text properties:
- Bidi controls detected regardless of context
- Multiple bidi chars all detected
- Nested bidi controls handled correctly
- Isolates vs overrides distinguished

### homoglyph_properties.rs
Tests homoglyph detection accuracy:
- ASCII letters never flagged as risks
- Known homoglyphs are detected (Cyrillic, Greek)
- No false positives on pure ASCII
- Mixed scripts trigger warnings

### scanner_stability.rs
Tests that scanner never panics:
- Arbitrary UTF-8 strings handled safely
- Arbitrary byte sequences don't crash
- Very long lines processed without panic
- Many lines don't cause crashes

### scanner_determinism.rs
Tests scanner produces consistent results:
- Same file always gives same violations
- Identical content gives identical results
- Scan order doesn't affect results

### config_properties.rs
Tests configuration parsing robustness:
- Parser never panics on arbitrary TOML
- Valid TOML structures handled safely
- Unicode ranges validated without panic
- Duplicate keys handled gracefully

### encoding_properties.rs
Tests encoding detection consistency:
- Same bytes always detect same encoding
- Valid UTF-8 detected correctly
- BOM detection is consistent
- Binary data handled without panic

## Running Property Tests

### Run all property tests
```bash
cargo test --test proptest
```

### Run specific property test file
```bash
cargo test --test unicode_categories
```

### Run with more cases (default is 256)
```bash
PROPTEST_CASES=10000 cargo test --test proptest
```

### Run with specific seed for reproduction
```bash
PROPTEST_SEED=abc123 cargo test --test proptest
```

## Writing Property Tests

Basic structure:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn my_property(input in 0u32..1000) {
        // Test that some property holds for all inputs
        prop_assert!(my_function(input) >= 0);
    }
}
```

### Common Strategies

```rust
// Any character
c in any::<char>()

// Any string
s in "\\PC*"  // Any Unicode string

// Range
n in 0..100

// Vector
v in prop::collection::vec(any::<u8>(), 0..1000)

// Custom
pub fn small_string() -> impl Strategy<Value = String> {
    "[a-z]{1,20}"
}
```

## Configuration

Property tests can be configured via `proptest!` macro or `ProptestConfig`:

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,  // Number of test cases
        max_shrink_iters: 1000,  // Shrinking iterations
        .. ProptestConfig::default()
    })]
    
    #[test]
    fn my_test(x in 0..100) {
        // test
    }
}
```

## Environment Variables

- `PROPTEST_CASES`: Number of cases to run (default: 256)
- `PROPTEST_MAX_SHRINK_ITERS`: Max shrinking iterations
- `PROPTEST_SEED`: Seed for reproducibility
- `PROPTEST_VERBOSE`: Enable verbose output

## Shrinking

When a property test fails, proptest automatically tries to find the **minimal failing case**:

```
Test failed for input: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
Shrinking to: [1, 2]
Minimal failing case: [1, 2]
```

This helps identify the root cause quickly.

## Regression Testing

Failed cases are automatically saved in `proptest-regressions/`:

```
proptest-regressions/
└── unicode_categories.txt
```

These files are checked into git to prevent regressions.

## Best Practices

1. **Test invariants, not implementation**: Test properties that should always be true
2. **Use generators wisely**: Start with simple generators, add complexity as needed
3. **Shrinking is your friend**: Let proptest find minimal cases
4. **Combine with example-based tests**: Use both property and example tests
5. **Check panics**: Use `std::panic::catch_unwind` to test panic-free guarantees

## Common Patterns

### Never panics
```rust
proptest! {
    #[test]
    fn never_panics(input in any::<String>()) {
        let result = std::panic::catch_unwind(|| {
            my_function(&input)
        });
        prop_assert!(result.is_ok());
    }
}
```

### Idempotence
```rust
proptest! {
    #[test]
    fn idempotent(x in any::<u32>()) {
        let once = normalize(x);
        let twice = normalize(once);
        prop_assert_eq!(once, twice);
    }
}
```

### Round-trip
```rust
proptest! {
    #[test]
    fn round_trip(data in any::<Vec<u8>>()) {
        let encoded = encode(&data);
        let decoded = decode(&encoded)?;
        prop_assert_eq!(data, decoded);
    }
}
```

## Resources

- [Proptest documentation](https://docs.rs/proptest/)
- [Proptest book](https://proptest-rs.github.io/proptest/)
- [Property-based testing](https://hypothesis.works/articles/what-is-property-based-testing/)
