# Research: Named Unicode Range Support

## Decision 1: Unicode Block Data Source Crate

**Decision**: Use the `unicode-blocks` crate (by magiclen)

**Rationale**:
- Provides `UnicodeBlock` struct with `name()`, `start()`, `end()`, `contains()` methods
- Contains 300+ constants for all official Unicode blocks (Unicode 15.1.0)
- Lightweight, focused crate with no transitive dependencies
- Actively maintained
- Constants contain the official block name as a `&'static str`
- `find_unicode_block(char)` function for reverse lookups

**Alternatives considered**:
- `unic-ucd-block`: Part of UNIC project, provides similar functionality but last published ~7 years ago. Less actively maintained.
- `icu4x`: Comprehensive Unicode library but does NOT include Unicode block support. Focuses on Script, GeneralCategory, Emoji properties instead. Heavy dependency.
- Manual hardcoding: Current approach in presets.rs. Fragile, error-prone, hard to keep up with Unicode releases.
- Core `std::char`: No block information available in Rust's standard library.

## Decision 2: Name-to-Block Lookup Strategy

**Decision**: Build a static `HashMap<String, &'static UnicodeBlock>` at initialization from the `unicode-blocks` crate constants, keyed by lowercased block name. Add alias entries to the same map.

**Rationale**:
- The `unicode-blocks` crate does not provide a built-in name-search function or an ALL_BLOCKS array
- A static HashMap provides O(1) lookup by name
- Lowercasing keys at build time implements case-insensitive matching
- Aliases are simply additional entries pointing to the same UnicodeBlock constant
- `once_cell::sync::Lazy` (or `std::sync::LazyLock` on Rust 1.80+) avoids manual lazy initialization

**Alternatives considered**:
- Linear search through constants: O(n) per lookup, but with ~330 blocks this is negligible. Simpler but doesn't support alias lookup without additional mapping.
- Code generation at build time: More complex build setup for marginal benefit.

## Decision 3: Fuzzy/Similar Name Suggestions

**Decision**: Use `strsim` crate for Levenshtein/Jaro-Winkler distance to suggest similar block names on typos.

**Rationale**:
- `strsim` is a well-maintained, lightweight crate providing multiple string similarity algorithms
- Jaro-Winkler distance works well for name similarity (favors common prefixes)
- Already a common pattern in CLI tools (e.g., `cargo` suggests similar commands on typos)

**Alternatives considered**:
- Manual substring matching: Less accurate for typos like "Hewbrew" vs "Hebrew"
- No suggestions: Poor user experience when block name is misspelled

## Decision 4: Config Field Design

**Decision**: Add `allowed_blocks: Option<Vec<String>>` to `RuleConfig` alongside existing `allowed_ranges: Vec<[u32; 2]>`. Both fields are optional with union semantics.

**Rationale**:
- TOML requires homogeneous arrays, so mixing strings and number pairs in one field is impossible
- Separate fields keep backward compatibility (existing configs unchanged)
- Union semantics: a character is allowed if it falls in ANY range from either field
- Resolution happens at config load time: block names are resolved to UnicodeRange objects and merged with numeric ranges

**Alternatives considered**:
- Single string-based field: Would require parsing "0x0000-0x007F" as strings, breaking backward compat
- Wrapper enum in TOML: Not supported by serde/toml without custom deserializer complexity
