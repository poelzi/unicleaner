# Feature Specification: Named Unicode Range Support

**Feature Branch**: `003-named-unicode-ranges`
**Created**: 2026-02-04
**Status**: Draft
**Input**: User description: "Support official Unicode block names in range configuration. Allow multiple ranges like 'range = latin-1, hebrew'. Use core unicode or other well-known unicode crate."

## Clarifications

### Session 2026-02-04

- Q: How should the config handle mixed types (named blocks + numeric ranges) given TOML's homogeneous array constraint? → A: Use separate fields: keep `allowed_ranges` for numeric pairs, add a new `allowed_blocks` field for named ranges.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Configure Allowed Blocks by Name (Priority: P1)

A developer configuring unicleaner for a multilingual project wants to specify allowed Unicode ranges using human-readable names instead of raw hex code point pairs. They edit their `unicleaner.toml` and write `allowed_blocks = ["Basic Latin", "Latin-1 Supplement", "Greek and Coptic"]` instead of memorizing and typing `allowed_ranges = [[0x0000, 0x007F], [0x0080, 0x00FF], [0x0370, 0x03FF]]`.

**Why this priority**: This is the core value proposition. Without named range support, users must look up hex code points manually, which is error-prone and unreadable. Named ranges make the configuration self-documenting and accessible.

**Independent Test**: Can be fully tested by creating a config file with named blocks, scanning a file containing characters from those ranges, and verifying they are accepted.

**Acceptance Scenarios**:

1. **Given** a config file with `allowed_blocks = ["Basic Latin"]`, **When** a file containing only ASCII characters is scanned, **Then** no violations are reported.
2. **Given** a config file with `allowed_blocks = ["Basic Latin"]`, **When** a file containing Greek characters (U+0370-U+03FF) is scanned, **Then** violations are reported for the Greek characters.
3. **Given** a config file with `allowed_blocks = ["Greek and Coptic"]`, **When** a file containing Greek characters is scanned, **Then** no violations are reported for those characters.
4. **Given** a config file with an unrecognized block name like `"Nonexistent Block"`, **When** the config is loaded, **Then** a clear error message is shown listing the invalid name and suggesting similar valid names.

---

### User Story 2 - Use Multiple Named Blocks Together (Priority: P1)

A developer working on a project that serves Hebrew and Latin audiences wants to allow both scripts. They configure `allowed_blocks = ["Basic Latin", "Latin-1 Supplement", "Hebrew"]` to permit characters from all three Unicode blocks in a single rule.

**Why this priority**: Real-world projects need multiple scripts. Supporting list-based named blocks is essential for practical use.

**Independent Test**: Can be tested by scanning a file with mixed Latin and Hebrew text against a config allowing both, verifying no false violations.

**Acceptance Scenarios**:

1. **Given** a config with `allowed_blocks = ["Basic Latin", "Hebrew"]`, **When** a file containing both ASCII and Hebrew characters is scanned, **Then** no violations are reported.
2. **Given** a config with `allowed_blocks = ["Basic Latin", "Hebrew"]`, **When** a file containing Cyrillic characters is scanned, **Then** violations are reported for the Cyrillic characters.
3. **Given** a config with `allowed_blocks = ["Basic Latin"]` and `allowed_ranges = [[0x0400, 0x04FF]]`, **When** a file is scanned, **Then** both the named blocks and numeric ranges are applied correctly (union of both).

---

### User Story 3 - Use Short Aliases for Common Blocks (Priority: P2)

A developer wants to use convenient short aliases like `"latin-1"` or `"ascii"` instead of typing the full official Unicode block name `"Latin-1 Supplement"` or `"Basic Latin"`. The system accepts both the official name and well-known aliases in the `allowed_blocks` field.

**Why this priority**: Reduces friction for common cases. Most users will work with Latin, ASCII, and a few other common scripts. Short aliases improve the daily experience.

**Independent Test**: Can be tested by using alias names in config and verifying they resolve to the correct Unicode block ranges.

**Acceptance Scenarios**:

1. **Given** a config with `allowed_blocks = ["ascii"]`, **When** parsed, **Then** it resolves to the Basic Latin block (U+0000-U+007F).
2. **Given** a config with `allowed_blocks = ["latin-1"]`, **When** parsed, **Then** it resolves to the Latin-1 Supplement block (U+0080-U+00FF).
3. **Given** a config with `allowed_blocks = ["hebrew"]`, **When** parsed, **Then** it resolves to the Hebrew block (U+0590-U+05FF).
4. **Given** a config with `allowed_blocks = ["Latin-1 Supplement"]` (official name), **When** parsed, **Then** it resolves to the same range as `"latin-1"`.

---

### User Story 4 - Discover Available Block Names (Priority: P2)

A developer wants to know what named blocks are available. They run a command or check documentation to see a list of all supported Unicode block names and their aliases.

**Why this priority**: Discoverability is important for usability. Without a way to list available blocks, users must consult external Unicode documentation.

**Independent Test**: Can be tested by invoking the listing functionality and verifying all official Unicode blocks are shown with their code point ranges.

**Acceptance Scenarios**:

1. **Given** the user runs a list command (e.g., `unicleaner list-blocks`), **When** the output is displayed, **Then** all supported Unicode block names are shown with their code point ranges.
2. **Given** the user searches for a block (e.g., `unicleaner list-blocks hebrew`), **When** the output is displayed, **Then** only matching blocks are shown.

---

### Edge Cases

- What happens when a block name is misspelled? The system should suggest the closest matching name(s).
- What happens when the same range is specified both via `allowed_blocks` and `allowed_ranges`? The ranges should be merged without duplication (union semantics).
- What happens when a named block and a numeric range overlap partially? Both should be applied; the union of all ranges determines what is allowed.
- How does the system handle case sensitivity? Block names should be case-insensitive (`"basic latin"`, `"Basic Latin"`, and `"BASIC LATIN"` all resolve to the same block).
- What happens with Unicode blocks that have changed between Unicode versions? The system should use a fixed, well-known Unicode version and document which version is in use.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST accept official Unicode block names (as defined by the Unicode Standard) as values in a new `allowed_blocks` configuration field.
- **FR-002**: The system MUST support specifying multiple named blocks in a single `allowed_blocks` list (e.g., `["Basic Latin", "Hebrew", "Arabic"]`).
- **FR-003**: The system MUST support using both `allowed_blocks` (named) and `allowed_ranges` (numeric) in the same rule, applying union semantics to determine allowed characters.
- **FR-004**: The system MUST provide short aliases for commonly used Unicode blocks (at minimum: `ascii`, `latin-1`, `latin-extended-a`, `latin-extended-b`, `greek`, `cyrillic`, `hebrew`, `arabic`, `cjk`, `hangul`, `hiragana`, `katakana`, `emoji`). The `emoji` alias maps to the Emoticons block (U+1F600-U+1F64F) only; other emoji-related blocks can be specified by official name.
- **FR-005**: Named block lookup MUST be case-insensitive.
- **FR-006**: When an unrecognized block name is provided, the system MUST produce a clear error message that includes the invalid name and suggests similar valid names.
- **FR-007**: The system MUST provide a way to list all available named blocks with their code point boundaries (via a CLI flag).
- **FR-008**: The system MUST use a well-maintained Unicode data source (a standard Rust crate that provides official Unicode block definitions) rather than hardcoding block ranges.
- **FR-009**: The existing numeric range format (`allowed_ranges = [[0xSTART, 0xEND]]`) MUST continue to work unchanged for backward compatibility.
- **FR-010**: The system MUST resolve named blocks to their correct Unicode code point boundaries at configuration load time, not at scan time.

### Key Entities

- **Unicode Block**: An official Unicode block with a canonical name, optional aliases, and a contiguous code point range (start, end). Examples: "Basic Latin" (U+0000-U+007F), "Hebrew" (U+0590-U+05FF).
- **Block Specification**: A string value in the `allowed_blocks` config field that references a Unicode block by official name or alias.
- **Range Specification**: A numeric pair `[start, end]` in the `allowed_ranges` config field that defines an allowed code point range.
- **Alias Mapping**: A lookup table mapping short convenience names (e.g., "ascii", "latin-1") to their corresponding official Unicode block names.

## Assumptions

- The Unicode block definitions will come from a well-maintained Rust crate (such as `unicode-blocks`, `icu`, or `unic-ucd-block`) rather than being manually maintained. The specific crate will be chosen during the planning phase based on maintenance status, dependency weight, and completeness.
- Aliases are additive convenience shortcuts; they do not replace or conflict with official names.
- The `cjk` alias maps to "CJK Unified Ideographs" (U+4E00-U+9FFF), the most commonly needed CJK block. Users who need additional CJK blocks (Extension A, B, etc.) can specify them by official name.
- Language presets (e.g., `preset = "rust"`) continue to work alongside named blocks. Named blocks provide finer-grained control.
- Both `allowed_blocks` and `allowed_ranges` are optional fields. A rule can use either, both, or neither.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can configure allowed Unicode ranges using human-readable block names instead of hex code points, reducing configuration errors.
- **SC-002**: All official Unicode blocks (as defined by the Unicode Standard) are available as named blocks.
- **SC-003**: Configuration files using named blocks produce identical scan results to equivalent numeric range configurations.
- **SC-004**: Invalid block names produce actionable error messages with suggestions, enabling users to self-correct without consulting external documentation.
- **SC-005**: Existing configurations using only numeric ranges continue to work without modification.
