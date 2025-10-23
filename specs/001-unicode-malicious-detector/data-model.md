# Data Model: Unicode Malicious Character Detector

**Feature**: Unicode Malicious Character Detector  
**Date**: 2025-10-23  
**Purpose**: Define core data structures and their relationships

---

## Core Entities

### 1. UnicodeRange

Represents a contiguous range of Unicode code points.

**Fields**:
- `start: u32` - Starting code point (inclusive)
- `end: u32` - Ending code point (inclusive)
- `description: Option<String>` - Human-readable description (e.g., "Greek Letters")

**Validation Rules**:
- `start <= end`
- Both `start` and `end` must be valid Unicode code points (<= 0x10FFFF)
- Ranges should not overlap within a single allowlist (warn if they do)

**Operations**:
- `contains(code_point: u32) -> bool` - Check if code point is in range
- `intersects(other: &UnicodeRange) -> bool` - Check for overlap with another range
- `merge(other: &UnicodeRange) -> Option<UnicodeRange>` - Merge adjacent/overlapping ranges

**Example**:
```rust
UnicodeRange {
    start: 0x0370,  // Greek capital letter Heta
    end: 0x03FF,    // Greek lowercase letter omega with ypogegrammeni
    description: Some("Greek and Coptic".to_string())
}
```

---

### 2. LanguagePreset

Named collection of Unicode ranges representing a natural language or script.

**Fields**:
- `name: String` - Preset identifier (e.g., "greek", "cyrillic", "chinese")
- `display_name: String` - Human-readable name (e.g., "Greek Letters")
- `ranges: Vec<UnicodeRange>` - Unicode ranges included in this preset
- `description: Option<String>` - Additional context

**Validation Rules**:
- `name` must be lowercase, alphanumeric + hyphens only
- `ranges` must not be empty
- Preset names must be unique within the built-in set

**Built-in Presets** (minimum viable set):
- `ascii`: U+0020-U+007E (printable ASCII)
- `latin-extended`: ASCII + U+0080-U+024F (Latin-1, Latin Extended A/B)
- `greek`: U+0370-U+03FF
- `cyrillic`: U+0400-U+04FF
- `chinese`: U+4E00-U+9FFF (CJK Unified Ideographs - common)
- `japanese-hiragana`: U+3040-U+309F
- `japanese-katakana`: U+30A0-U+30FF
- `korean-hangul`: U+AC00-U+D7AF
- `arabic`: U+0600-U+06FF
- `hebrew`: U+0590-U+05FF
- `emoji`: U+1F300-U+1F9FF (subset)

**Example**:
```rust
LanguagePreset {
    name: "greek".to_string(),
    display_name: "Greek Letters".to_string(),
    ranges: vec![
        UnicodeRange { start: 0x0370, end: 0x03FF, description: Some("Greek and Coptic".to_string()) }
    ],
    description: Some("Allows modern and ancient Greek characters".to_string())
}
```

---

### 3. MaliciousPattern

Defines a specific Unicode pattern considered malicious or suspicious.

**Fields**:
- `name: String` - Pattern identifier (e.g., "zero-width-space", "bidi-override")
- `category: MaliciousCategory` - Classification (enum: ZeroWidth, BidiOverride, Homoglyph, ControlChar)
- `code_points: Vec<u32>` - Specific code points OR
- `range: Option<UnicodeRange>` - Range of malicious code points
- `severity: Severity` - Impact level (enum: Error, Warning, Info)
- `description: String` - Explanation of why this is malicious
- `references: Vec<String>` - URLs to security advisories/research

**MaliciousCategory Enum**:
- `ZeroWidth` - Zero-width or invisible characters
- `BidiOverride` - Bidirectional text override chars (Trojan Source attack)
- `Homoglyph` - Characters that look similar to others (confusables)
- `ControlChar` - Non-printable control characters (excluding newline/tab)
- `NonStandard` - Deprecated or non-standard characters

**Severity Enum**:
- `Error` - Definite security risk (e.g., bidi override)
- `Warning` - Suspicious, needs review (e.g., homoglyphs in identifiers)
- `Info` - Low risk but unusual (e.g., rare Unicode)

**Example**:
```rust
MaliciousPattern {
    name: "zero-width-space".to_string(),
    category: MaliciousCategory::ZeroWidth,
    code_points: vec![0x200B],
    range: None,
    severity: Severity::Error,
    description: "Zero-width space can hide malicious code or alter identifiers invisibly".to_string(),
    references: vec!["https://trojansource.codes/".to_string()]
}
```

---

### 4. FileRule

Configuration rule specifying which Unicode characters are allowed for files matching a pattern.

**Fields**:
- `pattern: String` - Glob pattern (e.g., "*.rs", "src/**/*.py")
- `allowed_ranges: Vec<UnicodeRange>` - Explicitly allowed ranges
- `allowed_presets: Vec<String>` - Preset names to include
- `deny_overrides: Vec<u32>` - Specific code points to deny even if in allowed ranges
- `priority: i32` - Rule precedence (higher = more specific, applied last)

**Validation Rules**:
- `pattern` must be valid glob syntax
- At least one of `allowed_ranges` or `allowed_presets` must be non-empty
- `deny_overrides` takes precedence over allowlists

**Matching Logic**:
1. Find all rules where pattern matches the file path
2. Sort by priority (ascending)
3. Merge allowed ranges from all matching rules
4. Apply deny overrides
5. If no rules match, use global default (deny-by-default)

**Example**:
```rust
FileRule {
    pattern: "src/**/*.rs".to_string(),
    allowed_ranges: vec![],
    allowed_presets: vec!["ascii".to_string(), "latin-extended".to_string()],
    deny_overrides: vec![0x200B], // Still deny zero-width even if somehow in preset
    priority: 10
}
```

---

### 5. Configuration

Top-level configuration loaded from TOML file.

**Fields**:
- `default_allow: Vec<String>` - Preset names allowed globally (default: ["ascii"])
- `default_deny: Vec<u32>` - Always-denied code points regardless of rules
- `file_rules: Vec<FileRule>` - Per-file-pattern rules
- `ignore_patterns: Vec<String>` - Glob patterns for files to skip (e.g., "*.min.js", "vendor/**")
- `severity_threshold: Severity` - Only report violations >= this level (default: Warning)

**Validation Rules**:
- Must have at least default_allow OR file_rules (empty config is invalid)
- Ignore patterns should not conflict with scan targets
- Severity threshold must be valid enum value

**Default Configuration** (if no file provided):
```toml
[default]
allow = ["ascii"]
deny = [
    0x200B, 0x200C, 0x200D, 0xFEFF,  # Zero-width
    0x202A, 0x202B, 0x202C, 0x202D, 0x202E,  # Bidi overrides
]

[ignore]
patterns = [".git/**", "node_modules/**", "target/**", "*.lock"]

[reporting]
severity_threshold = "warning"
```

**Example (Rust config)**:
```rust
Configuration {
    default_allow: vec!["ascii".to_string()],
    default_deny: vec![0x200B, 0x200C, 0x200D, 0xFEFF, 0x202A, 0x202B, 0x202C, 0x202D, 0x202E],
    file_rules: vec![
        FileRule {
            pattern: "docs/**/*.md".to_string(),
            allowed_presets: vec!["ascii".to_string(), "emoji".to_string()],
            allowed_ranges: vec![],
            deny_overrides: vec![],
            priority: 5
        }
    ],
    ignore_patterns: vec![".git/**".to_string(), "target/**".to_string()],
    severity_threshold: Severity::Warning
}
```

---

### 6. Violation

Represents a detected malicious Unicode character in a file.

**Fields**:
- `file_path: PathBuf` - Absolute or relative path to file
- `line: usize` - Line number (1-indexed)
- `column: usize` - Column number (1-indexed, byte offset or grapheme cluster)
- `code_point: u32` - The offending Unicode code point
- `character: char` - Rust char representation
- `context: String` - Surrounding text (e.g., 20 chars before/after)
- `pattern: MaliciousPattern` - Which pattern was violated
- `severity: Severity` - Severity level (from pattern)

**Validation Rules**:
- `line` and `column` must be > 0
- `code_point` must be valid Unicode
- `file_path` must exist (or be reported as scan error)

**Display Format** (human-readable):
```
error: Zero-width space detected
  --> src/main.rs:42:15
   |
42 | let user​name = "admin";  // <- invisible U+200B here
   |            ^
   = note: Zero-width space can hide malicious code
```

**JSON Format**:
```json
{
  "file": "src/main.rs",
  "line": 42,
  "column": 15,
  "code_point": "U+200B",
  "character": "​",
  "severity": "error",
  "pattern": "zero-width-space",
  "message": "Zero-width space can hide malicious code",
  "context": "let user​name = \"admin\";"
}
```

**Example (Rust struct)**:
```rust
Violation {
    file_path: PathBuf::from("src/main.rs"),
    line: 42,
    column: 15,
    code_point: 0x200B,
    character: '\u{200B}',
    context: "let user​name = \"admin\";".to_string(),
    pattern: MaliciousPattern { /* ... */ },
    severity: Severity::Error
}
```

---

### 7. ScanResult

Aggregate result of scanning one or more files.

**Fields**:
- `violations: Vec<Violation>` - All detected violations
- `files_scanned: usize` - Total files processed
- `files_clean: usize` - Files with no violations
- `files_with_violations: usize` - Files with at least one violation
- `errors: Vec<ScanError>` - Files that couldn't be scanned
- `duration: Duration` - Time taken to scan
- `config_used: PathBuf` - Path to config file (or "<default>")

**Derived Fields** (computed):
- `total_violations: usize` = violations.len()
- `passed: bool` = violations.is_empty() && errors.is_empty()
- `exit_code: i32` = 0 if passed, 1 if violations, 2 if errors

**Operations**:
- `by_severity() -> HashMap<Severity, Vec<Violation>>` - Group violations by severity
- `by_file() -> HashMap<PathBuf, Vec<Violation>>` - Group violations by file
- `summary_string() -> String` - Human-readable summary

**Example**:
```rust
ScanResult {
    violations: vec![/* ... */],
    files_scanned: 150,
    files_clean: 147,
    files_with_violations: 3,
    errors: vec![],
    duration: Duration::from_secs(2),
    config_used: PathBuf::from("unicleaner.toml")
}
```

---

### 8. ScanError

Represents a file that failed to scan.

**Fields**:
- `file_path: PathBuf` - File that caused the error
- `error_type: ErrorType` - Classification (enum: IoError, EncodingError, ParseError)
- `message: String` - Detailed error message

**ErrorType Enum**:
- `IoError` - File not readable (permissions, doesn't exist, etc.)
- `EncodingError` - Could not detect or decode encoding
- `ParseError` - File is binary or otherwise unparsable as text

**Example**:
```rust
ScanError {
    file_path: PathBuf::from("vendor/binary.dat"),
    error_type: ErrorType::ParseError,
    message: "Binary file detected (null bytes in first 8KB)".to_string()
}
```

---

## Entity Relationships

```
Configuration (1) ──┬──> (0..N) FileRule
                    │
                    └──> (0..N) LanguagePreset ──> (1..N) UnicodeRange

MaliciousPattern (N) <──┐
                        │
Violation (N) ──────────┴──> (1) MaliciousPattern
              └──> (1) File

ScanResult (1) ──┬──> (0..N) Violation
                 │
                 └──> (0..N) ScanError
```

---

## State Transitions

### Configuration Loading
```
Start → Parse TOML → Validate → Resolve Presets → Merge Rules → Ready
                         │                                        │
                         └── Error ──────────────────────────────┘
```

### File Scanning
```
File → Detect Encoding → Decode to UTF-8 → Iterate Graphemes → Check Against Rules
  │                                              │                      │
  │                                              v                      v
  │                                         Violation              Clean
  │                                              │                      │
  └── Scan Error ────────────────────────────────┴──────────────────────┴─→ Result
```

---

## Invariants

1. **Code Point Validity**: All `u32` values representing code points MUST be <= 0x10FFFF
2. **Range Consistency**: In `UnicodeRange`, `start <= end` always
3. **Severity Ordering**: Error > Warning > Info (for filtering)
4. **File Rule Priority**: Higher priority rules override lower priority when patterns overlap
5. **Deny Overrides**: `deny_overrides` in FileRule ALWAYS take precedence over `allowed_ranges`
6. **Default Deny**: If no rules match a file, only `default_allow` presets are permitted
7. **Exit Code Contract**: 0 = clean, 1 = violations found, 2 = scan errors (e.g., IO failures)

---

## Performance Considerations

1. **Unicode Range Lookup**: Use interval trees or sorted vectors for O(log n) contains() checks
2. **Grapheme Iteration**: Use `unicode-segmentation` to handle multi-codepoint characters correctly
3. **Parallel Scanning**: ScanResult aggregates results from parallel file scans (thread-safe)
4. **Memory Mapping**: For files >10MB, use memory-mapped IO instead of loading into RAM
5. **Preset Expansion**: Presets are expanded once at config load time, not per-file

---

## Example Data Flow

1. **User runs**: `unicleaner src/ --config unicleaner.toml`
2. Load & validate `unicleaner.toml` → **Configuration**
3. Walk directory `src/` → List of files
4. Apply `ignore_patterns` → Filter files
5. For each file in parallel:
   - Detect encoding → Decode to UTF-8
   - Match file path against `FileRule` patterns → Determine allowed ranges
   - Iterate grapheme clusters
   - For each code point: Check if in allowed ranges OR in malicious patterns
   - If malicious → Create **Violation**
6. Aggregate all **Violations** → **ScanResult**
7. Format output (human or JSON)
8. Exit with appropriate code

---

## JSON Schema (for API contract)

Violation output schema:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["file", "line", "column", "code_point", "severity", "pattern", "message"],
  "properties": {
    "file": { "type": "string" },
    "line": { "type": "integer", "minimum": 1 },
    "column": { "type": "integer", "minimum": 1 },
    "code_point": { "type": "string", "pattern": "^U\\+[0-9A-F]{4,6}$" },
    "character": { "type": "string" },
    "severity": { "enum": ["error", "warning", "info"] },
    "pattern": { "type": "string" },
    "message": { "type": "string" },
    "context": { "type": "string" }
  }
}
```

ScanResult output schema:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["violations", "summary"],
  "properties": {
    "violations": {
      "type": "array",
      "items": { "$ref": "#/definitions/Violation" }
    },
    "summary": {
      "type": "object",
      "required": ["files_scanned", "files_clean", "files_with_violations", "total_violations"],
      "properties": {
        "files_scanned": { "type": "integer" },
        "files_clean": { "type": "integer" },
        "files_with_violations": { "type": "integer" },
        "total_violations": { "type": "integer" },
        "duration_ms": { "type": "integer" }
      }
    }
  }
}
```
