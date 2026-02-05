# Implementation Plan: Named Unicode Range Support

**Branch**: `003-named-unicode-ranges` | **Date**: 2026-02-04 | **Spec**: `specs/003-named-unicode-ranges/spec.md`
**Input**: Feature specification from `/specs/003-named-unicode-ranges/spec.md`

## Summary

Add support for official Unicode block names (e.g., "Basic Latin", "Hebrew") in the `unicleaner.toml` configuration via a new `allowed_blocks` field. Uses the `unicode-blocks` crate for block definitions and `strsim` for typo suggestions. Named blocks are resolved to `UnicodeRange` at config load time and merged with existing numeric `allowed_ranges` using union semantics.

## Technical Context

**Language/Version**: Rust 1.85+ (MSRV, edition 2024)
**Primary Dependencies**: `unicode-blocks` (block definitions), `strsim` (fuzzy matching), `once_cell` (lazy initialization), `clap` (CLI), `serde`/`toml` (config)
**Storage**: TOML configuration files
**Testing**: `cargo test --all-features`, `cargo-fuzz` for fuzz testing
**Target Platform**: Linux, macOS, Windows (cross-platform CLI)
**Project Type**: Single Rust project
**Performance Goals**: Block name resolution at config load time only; no runtime overhead during scanning
**Constraints**: Must not break existing `allowed_ranges` configs (backward compatible).
**Scale/Scope**: ~330 Unicode blocks, 13 short aliases

## Constitution Compliance

Per constitution principles:

- **III. Test-First (NON-NEGOTIABLE)**: All implementation steps follow TDD — tests are written and verified to fail before implementation code is written.
- **IV. Comprehensive Testing**: Unit tests, integration tests, and fuzz testing are all included.
- **VII. Code Quality**: `cargo clippy` and `cargo fmt --check` gates are applied after each phase.

## Project Structure

### Documentation (this feature)

```text
specs/003-named-unicode-ranges/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: crate evaluation, design decisions
├── data-model.md        # Phase 1: entity definitions, relationships
├── quickstart.md        # Phase 1: usage guide
├── checklists/
│   └── requirements.md  # Quality checklist
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── unicode/
│   ├── blocks.rs        # NEW: BlockRegistry, resolve(), suggest(), list()
│   ├── ranges.rs        # EXISTING: UnicodeRange (unchanged)
│   └── mod.rs           # MODIFY: add blocks module
├── config/
│   ├── parser.rs        # MODIFY: add allowed_blocks to RuleConfig, resolve in load_config
│   ├── validation.rs    # EXISTING: unchanged (block names validated in parser.rs at load time)
│   ├── rules.rs         # EXISTING: FileRule (unchanged)
│   ├── presets.rs        # EXISTING: unchanged
│   └── mod.rs           # EXISTING: unchanged
├── cli/
│   └── args.rs          # MODIFY: add ListBlocks command with optional filter

tests/
├── integration/
│   └── block_config_tests.rs  # NEW: integration tests for named block configs

fuzz/
└── fuzz_targets/
    └── fuzz_block_resolve.rs  # NEW: fuzz target for BlockRegistry::resolve()
```

**Structure Decision**: Single Rust project. New code goes in `src/unicode/blocks.rs`. Config parsing extended in `src/config/parser.rs`. CLI extended in `src/cli/args.rs`. Fuzz target added for block name resolution.

## Implementation Steps (TDD)

Each step follows the Red-Green-Refactor cycle per constitution Principle III.

### Step 1: Add dependencies

Add `unicode-blocks`, `strsim`, and `once_cell` to `Cargo.toml`. Register `pub mod blocks;` in `src/unicode/mod.rs`.

### Step 2: Create `src/unicode/blocks.rs` - BlockRegistry (TDD)

**Red**: Write unit tests first for resolve, case-insensitive lookup, and unknown name suggestions. Create minimal stubs so tests compile but fail.

**Green**: Then implement:
- Define `BlockEntry` struct (name, start, end)
- Build static `HashMap<String, BlockEntry>` using `once_cell::sync::Lazy`
- Populate from `unicode_blocks` crate constants (all ~330 blocks), keyed by lowercased official name
- Implement `resolve(name: &str) -> Result<UnicodeRange, BlockError>` — lowercase input, lookup in map, return `UnicodeRange` on hit or `BlockError` with suggestions on miss
- Implement `suggest(name: &str) -> Vec<String>` — use `strsim::jaro_winkler` to find top 3 similar names
- Implement `list_blocks(filter: Option<&str>) -> Vec<BlockInfo>` — return all blocks sorted by start code point, with optional case-insensitive substring filter

**Refactor**: Run `cargo clippy` and `cargo fmt --check`. Verify all tests pass.

### Step 3: Extend config parser (TDD)

**Red**: Write integration tests first — config with `allowed_blocks` accepts correct characters, rejects others; invalid block name produces error with suggestions.

**Green**: Then implement:
- Add `allowed_blocks: Vec<String>` to `RuleConfig` in `src/config/parser.rs`
- In `load_config`, resolve each block name via `BlockRegistry::resolve()`, add resulting `UnicodeRange` to `FileRule.allowed_ranges`
- Propagate errors with suggestions on failure

**Refactor**: Run `cargo clippy` and `cargo fmt --check`. Verify all tests pass.

### Step 4: Add short aliases (TDD)

**Red**: Write tests for alias resolution ("ascii" → Basic Latin, "latin-1" → Latin-1 Supplement, etc.). Verify they fail.

**Green**: Add 13 alias entries to the BlockRegistry HashMap.

**Refactor**: Run `cargo clippy` and `cargo fmt --check`. Verify all tests pass.

### Step 5: Add `ListBlocks` CLI command (TDD)

**Red**: Write unit tests for `list_blocks` with and without filter. Verify they fail.

**Green**: Implement:
- Add `ListBlocks` variant to `Command` enum in `src/cli/args.rs` with optional filter argument
- Handle in `main.rs`: call `BlockRegistry::list_blocks()`, format and print as table

**Refactor**: Run `cargo clippy` and `cargo fmt --check`. Verify all tests pass.

### Step 6: Fuzz testing

Add fuzz target for `BlockRegistry::resolve()` in `fuzz/fuzz_targets/fuzz_block_resolve.rs`. Run for at least 60 seconds. Fix any panics discovered.

### Step 7: Final validation

- Run full quality gate: `cargo test --all-features`, `cargo clippy -- -D warnings`, `cargo fmt --check`
- Verify backward compatibility: configs with only `allowed_ranges` still work
- Validate quickstart.md examples work as documented

## Complexity Tracking

No constitution violations. This feature adds one new source file, extends two existing files, adds three new dependencies (`unicode-blocks`, `strsim`, `once_cell`), and one fuzz target. No new abstractions or patterns beyond what already exists in the codebase.
