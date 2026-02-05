# Quickstart: Named Unicode Range Support

## Configuration

Add named Unicode blocks to any rule in `unicleaner.toml`:

```toml
[[rules]]
pattern = "**/*.rs"
allowed_blocks = ["Basic Latin", "Latin-1 Supplement"]

[[rules]]
pattern = "docs/**/*.md"
allowed_blocks = ["ascii", "greek", "cyrillic", "hebrew"]
allowed_ranges = [[0x2000, 0x206F]]  # General Punctuation - can combine both
```

Both `allowed_blocks` and `allowed_ranges` can be used together (union semantics).

## Listing Available Blocks

```bash
# List all available Unicode blocks
unicleaner list-blocks

# Filter by name
unicleaner list-blocks hebrew
```

## Short Aliases

Common blocks have short aliases: `ascii`, `latin-1`, `latin-extended-a`, `latin-extended-b`, `greek`, `cyrillic`, `hebrew`, `arabic`, `cjk`, `hangul`, `hiragana`, `katakana`, `emoji`.

## Error Handling

Misspelled block names produce helpful suggestions:

```
Error: Unknown Unicode block "Hewbrew"
  Did you mean: "Hebrew"?
```

## Running Tests

```bash
cargo test --all-features
```

## Key Files

- `src/unicode/blocks.rs` - BlockRegistry, name resolution, alias table
- `src/config/parser.rs` - RuleConfig with `allowed_blocks` field
- `src/cli/args.rs` - `ListBlocks` command with optional filter
- `tests/integration/` - Integration tests for named block configs
