# unicleaner Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-23

## Active Technologies

- Rust (stable channel, edition 2024, MSRV 1.85+) (001-unicode-malicious-detector)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust (stable channel, edition 2024, MSRV 1.85+): Follow standard conventions
AVOID openssl
use rust only libraries everywhere

## Recent Changes
- 001-unicode-malicious-detector: Added Rust (stable channel, edition 2024, MSRV 1.85+)
- 002-enhance-unicode-tests: Added [if applicable, e.g., PostgreSQL, CoreData, files or N/A]
- 003-named-unicode-ranges: Added Rust 1.85+ (edition 2024) + `unicode-blocks` (block definitions), `strsim` (fuzzy matching), `clap` (CLI), `serde`/`toml` (config)
- 003-named-unicode-ranges: Added [if applicable, e.g., PostgreSQL, CoreData, files or N/A]

## Active Technologies
- Rust 1.85+ (edition 2024) + `unicode-blocks` (block definitions), `strsim` (fuzzy matching), `clap` (CLI), `serde`/`toml` (config) (003-named-unicode-ranges)
- TOML configuration files (003-named-unicode-ranges)

<!-- MANUAL ADDITIONS START -->

## Config Sync Rules

When changing config fields, parsing, or adding new config options (e.g. in `src/config/`), always update `examples/unicleaner.toml` to reflect the changes. The example config is validated by the integration test `test_example_config_loads_successfully` in `tests/integration/config_tests.rs` and must remain loadable by the scanner.

<!-- MANUAL ADDITIONS END -->
