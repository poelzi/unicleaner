# Data Model: Named Unicode Range Support

## New Entities

### BlockRegistry

A singleton registry that maps block names and aliases to Unicode block definitions. Built once at startup using `once_cell::sync::Lazy`.

- **Fields**:
  - `blocks: HashMap<String, BlockEntry>` - lowercase name/alias → block entry
- **Construction**: Iterates all constants from `unicode-blocks` crate, inserts each by lowercased official name. Then inserts alias entries pointing to the same data.
- **Lookup**: `resolve(name: &str) -> Result<UnicodeRange, BlockError>` - lowercases input, looks up in map, returns `UnicodeRange` or error with suggestions.
- **Listing**: `all_blocks() -> Vec<BlockInfo>` - returns all blocks sorted by start code point for `--list-blocks`.
- **Suggestions**: `suggest(name: &str) -> Vec<String>` - uses `strsim::jaro_winkler` to find similar names when lookup fails.

### BlockEntry

Internal struct holding resolved block data.

- **Fields**:
  - `name: String` - official Unicode block name
  - `start: u32` - first code point
  - `end: u32` - last code point
  - `aliases: Vec<String>` - convenience aliases (e.g., "ascii", "latin-1")

### BlockError

Error type for block resolution failures.

- **Variants**:
  - `UnknownBlock { name: String, suggestions: Vec<String> }` - block name not found, includes similar names

## Modified Entities

### RuleConfig (in `src/config/parser.rs`)

Add a new optional field for named blocks.

```rust
struct RuleConfig {
    pattern: String,
    #[serde(default)]
    allowed_ranges: Vec<[u32; 2]>,      // existing - unchanged
    #[serde(default)]
    allowed_blocks: Vec<String>,         // NEW - named Unicode block references
    #[serde(default)]
    denied_characters: Vec<u32>,
}
```

### Config Loading Pipeline (in `src/config/parser.rs`)

The `load_config` function gains a new step after parsing TOML:

```
For each RuleConfig:
  1. Create FileRule with glob pattern (existing)
  2. Add each allowed_ranges[i] as UnicodeRange (existing)
  3. NEW: For each allowed_blocks[j]:
     a. Resolve via BlockRegistry::resolve(name)
     b. On success: add resulting UnicodeRange to FileRule
     c. On failure: return error with suggestions
  4. Add each denied_characters[k] as denied code point (existing)
```

Resolution happens at config load time (FR-010). After loading, named blocks are indistinguishable from numeric ranges - they're all `UnicodeRange` values in `FileRule.allowed_ranges`.

### CLI (new flag)

- `--list-blocks [filter]` - lists available block names, code point ranges, and aliases. Optional filter does case-insensitive substring match.

## Alias Table

| Alias | Official Unicode Block Name | Range |
|-------|---------------------------|-------|
| `ascii` | Basic Latin | U+0000-U+007F |
| `latin-1` | Latin-1 Supplement | U+0080-U+00FF |
| `latin-extended-a` | Latin Extended-A | U+0100-U+017F |
| `latin-extended-b` | Latin Extended-B | U+0180-U+024F |
| `greek` | Greek and Coptic | U+0370-U+03FF |
| `cyrillic` | Cyrillic | U+0400-U+04FF |
| `hebrew` | Hebrew | U+0590-U+05FF |
| `arabic` | Arabic | U+0600-U+06FF |
| `cjk` | CJK Unified Ideographs | U+4E00-U+9FFF |
| `hangul` | Hangul Syllables | U+AC00-U+D7AF |
| `hiragana` | Hiragana | U+3040-U+309F |
| `katakana` | Katakana | U+30A0-U+30FF |
| `emoji` | Emoticons | U+1F600-U+1F64F |

## Entity Relationships

```
BlockRegistry (singleton, LazyLock)
  └── HashMap<String, BlockEntry>  (lowercase name/alias → block data)

Config Loading:
  RuleConfig.allowed_blocks: Vec<String>
    → BlockRegistry::resolve(name)
    → UnicodeRange { start, end, description }
    → FileRule.allowed_ranges (merged with numeric ranges)

At runtime, FileRule.allowed_ranges contains both:
  - Ranges from allowed_ranges (numeric config)
  - Ranges from allowed_blocks (resolved named blocks)
No distinction is made after loading.
```

## Validation Rules

- Block names must resolve to a known block or alias (case-insensitive)
- On failure, error message includes the invalid name and up to 3 suggestions
- Duplicate ranges (from overlapping blocks/ranges) are harmless - union semantics
- Empty `allowed_blocks` is valid (field is optional, defaults to `[]`)

## New Dependencies

- `unicode-blocks` - Unicode block definitions (Unicode 15.1.0)
- `strsim` - String similarity for typo suggestions
