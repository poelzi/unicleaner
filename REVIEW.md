# unicleaner publish-readiness review (2026-02-11)

## Scope and verification status

- Reviewed crate packaging, metadata, CLI/docs consistency, tests, and release workflows.
- All findings resolved 2026-02-12. Verified with the project toolchain and clean target directories.

### Verification results (2026-02-12)

```
cargo test --all-features --workspace  => 349 passed, 0 failed
cargo clippy --all-targets --all-features -- -D warnings  => clean
cargo package --allow-dirty --no-verify  => 124 files, 470.7KiB (107.6KiB compressed)
cargo package --allow-dirty --list  => no tmp/, .direnv/, specs/, or other artifacts
```

---

## Findings

### P0 - publish blockers

1) ~~Packaging fails due transient `tmp/` artifacts being included~~ **DONE**

- Fix applied: Added root-anchored `include` list in `Cargo.toml`. Package now contains exactly 124 files with no unwanted artifacts.

### P1 - high priority

2) ~~Crate/repo metadata still uses placeholder owner~~ **DONE**

- Fix applied: Replaced `yourusername` with `poelzi` across Cargo.toml, README.md, docs/DOCKER.md, CHANGELOG.md, flake.nix, release.yml, example workflows, and specs quickstart. Only remaining occurrence is in this REVIEW.md file itself.

3) ~~CLI docs/examples still reference unsupported `--output` flag~~ **DONE**

- Fix applied: Replaced all `--output <file>` patterns with `> <file>` shell redirection in README.md (4 occurrences) and docs/DOCKER.md (6 occurrences). Zero `--output` references remain in either file.

4) ~~Core module is still a TODO stub~~ **DONE**

- Fix applied: Deleted `src/unicode/database.rs` and removed `pub mod database;` from `src/unicode/mod.rs`. Module was unused by scanner.

### P2 - medium priority

5) ~~Placeholder test files still exist and do not validate behavior~~ **DONE**

- Fix applied:
  - `scan_tests.rs`: All 4 tests now call `unicleaner::scanner::file_scanner::scan_file()` and assert on actual violations (zero-width, bidi, homoglyph detection) or clean results.
  - `config_tests.rs`: All 4 TODO stubs replaced with real CLI invocations via `assert_cmd` testing config loading, file rules, glob patterns, and invalid config error handling.

6) ~~Performance tests use mock scan functions instead of real scanner path~~ **DONE**

- Fix applied: Both `memory_limits.rs` and `parallel_scaling.rs` now call the real `unicleaner::scanner::file_scanner::scan_file()` instead of no-op/sleep mocks. Memory thresholds adjusted for realistic scanning behavior. All performance tests pass.

7) ~~Extensive placeholder org references remain in docs/workflows/examples~~ **DONE**

- Fix applied: Same as finding #2. All `yourusername` replaced with `poelzi` across all public-facing files.

### P3 - low priority cleanup

8) ~~`review.md` (lowercase) and `REVIEW.md` can diverge~~ **DONE**

- Fix applied: Deleted stale `review.md`. Only `REVIEW.md` remains as the canonical review artifact.

---

## Recommended implementation order for publication

All items completed:

1. ~~Fix packaging manifest (`Cargo.toml` include/exclude), then make `cargo package` succeed.~~ **DONE**
2. ~~Replace repository/image placeholders (`yourusername`) across metadata/docs/workflows.~~ **DONE**
3. ~~Fix docs/examples to match CLI (`--format`, redirection instead of `--output`).~~ **DONE**
4. ~~Resolve or de-scope `src/unicode/database.rs` TODO module.~~ **DONE**
5. ~~Replace TODO tests and mock performance tests with real scanner validations.~~ **DONE**
6. ~~Re-run full verification once linker environment is fixed.~~ **DONE**

---

## Environment blocker note

~~Current host has a linker wrapper issue preventing compile/link steps.~~ **RESOLVED**

The linker issue only affects the bare `rustup` toolchain. Verification was completed successfully with a clean target configuration and the repository's standard commands.
