# Research for Enhance Unicode Test Suite

## Research on Modern Unicode Attacks

**Decision**: Incorporate test cases for bidirectional control characters, homoglyphs, zero-width characters, and normalization issues as per Unicode Security Mechanisms (UTS #39) and Trojan Source vulnerabilities.

**Rationale**: These are common vectors for Unicode-based attacks, such as code reordering (Trojan Source CVE-2021-42574), character confusion (CVE-2021-42694), and hiding malicious content. Research from Unicode TR#36 (stabilized), UTS #39, and Trojan Source paper confirms these as critical for source code security.

**Alternatives considered**: Focusing only on bidi controls or homoglyphs; however, a comprehensive approach covering multiple categories ensures broader coverage.

## Best Practices for Unicode Testing in Rust

**Decision**: Use proptest for property-based testing of Unicode invariants, cargo-fuzz for input fuzzing, and custom fixtures for specific attack vectors.

**Rationale**: Aligns with Rust's testing ecosystem and project constitution. Proptest can generate diverse Unicode strings to test detection logic, while fuzzing discovers edge cases like invalid sequences.

**Alternatives considered**: Quickcheck as alternative to proptest; chosen proptest for better integration with Rust's test framework.

## Glassworm Hiding Technique

**Decision**: Treat "glassworm" as a potential typo or reference to "Glastonbury" or similar, but interpret as homoglyph or bidi hiding based on context; include tests for invisible characters and confusable scripts.

**Rationale**: No direct match found for "glassworm" in Unicode attack literature; likely refers to worm-like hiding via zero-width spaces or bidi overrides, similar to Trojan Source examples.

**Alternatives considered**: If it's a specific malware, further clarification needed, but assuming general hiding techniques based on user input.
