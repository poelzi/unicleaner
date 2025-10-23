<!--
SYNC IMPACT REPORT

Version Change: [NEW] → 1.0.0
Reason: Initial constitution ratification for unicleaner project

Modified Principles: N/A (initial creation)
Added Sections:
  - Core Principles (I-VIII): Rust-First, CLI Interface, Test-First (NON-NEGOTIABLE), Comprehensive Testing Strategy, Color Output Support, Nix Integration, Code Quality, Documentation
  - Technology Stack: Rust, Nix, Testing frameworks
  - Development Workflow: TDD cycle, testing gates, quality checks
  - Governance: Amendment procedures, version control, compliance verification

Removed Sections: N/A (initial creation)

Templates Requiring Updates:
  ✅ .specify/templates/plan-template.md - Already generic, compatible with Rust/Nix requirements
  ✅ .specify/templates/spec-template.md - Technology-agnostic, no changes needed
  ✅ .specify/templates/tasks-template.md - Already supports test-first workflow
  ✅ .specify/templates/checklist-template.md - Generic template, compatible
  ✅ .specify/templates/agent-file-template.md - Will auto-populate from plans

Follow-up TODOs: None - all placeholders resolved
-->

# Unicleaner Constitution

## Core Principles

### I. Rust-First
Every component MUST be written in Rust using idiomatic patterns and leveraging the Rust ecosystem. The project embraces Rust's safety guarantees, zero-cost abstractions, and strong type system.

**Rationale**: Rust provides memory safety without garbage collection, prevents data races at compile time, and delivers predictable performance—essential for a reliable CLI tool.

**Requirements**:
- Use Cargo for dependency management and builds
- Follow Rust API guidelines and naming conventions
- Leverage the type system for correctness (Result, Option, strong typing)
- Use idiomatic error handling with thiserror or anyhow
- Prefer standard library and well-maintained crates
- Code MUST compile without warnings on stable Rust

### II. CLI Interface
All functionality MUST be exposed via a clean, well-designed command-line interface following Unix philosophy principles.

**Rationale**: CLI tools should be composable, scriptable, and provide clear, parseable output for both humans and machines.

**Requirements**:
- Text-based input: arguments, flags, stdin
- Output protocol: stdout for results, stderr for errors/warnings/diagnostics
- Support both human-readable and machine-parseable formats (JSON when appropriate)
- Exit codes MUST follow conventions: 0 = success, non-zero = specific error conditions
- Provide --help and --version flags
- Support standard Unix patterns (pipes, redirection, signals)

### III. Test-First (NON-NEGOTIABLE)
Test-Driven Development (TDD) is MANDATORY. Tests MUST be written before implementation, reviewed by user, verified to fail, then implementation proceeds.

**Rationale**: TDD catches design flaws early, ensures testability, provides living documentation, and prevents regressions. This is non-negotiable because it fundamentally improves code quality and maintainability.

**Requirements**:
- Red-Green-Refactor cycle MUST be strictly followed
- Write test → Get user approval → Verify test fails → Implement → Verify test passes
- No implementation code without corresponding failing tests first
- Tests MUST be clear, focused, and independently runnable
- Test names MUST clearly describe what they verify

### IV. Comprehensive Testing Strategy
The project MUST maintain multiple layers of testing to ensure correctness, robustness, and reliability at all levels.

**Rationale**: Different testing strategies catch different types of bugs. Unit tests verify components, integration tests verify interactions, fuzz testing discovers edge cases, and property-based testing validates invariants.

**Requirements**:
- **Unit Tests** (MANDATORY): Test individual functions, modules, and components in isolation using Rust's built-in test framework
- **Integration Tests** (MANDATORY): Test CLI behavior end-to-end, verify contract adherence, test cross-module interactions
- **Fuzz Testing** (MANDATORY): Use cargo-fuzz or similar to discover input edge cases and panics
- **Property-Based Testing**: Use proptest or quickcheck for invariant validation where applicable
- All tests MUST pass before merging
- Maintain test coverage visibility (use cargo-tarpaulin or similar)
- Tests MUST be fast enough to run frequently during development

### V. Color Output Support
CLI output MUST support both colored and non-colored modes with automatic and manual control.

**Rationale**: Colors improve readability for humans but break parsing and are inappropriate for non-TTY contexts (pipes, CI logs, accessibility tools).

**Requirements**:
- Automatically detect TTY vs non-TTY output (disable colors for pipes/redirects)
- Provide explicit flags: --color=auto|always|never (or similar)
- Respect NO_COLOR environment variable (https://no-color.org)
- Use a robust color library (colored, termcolor, or owo-colors)
- Ensure all output remains readable without colors
- NEVER emit ANSI codes when colors are disabled

### VI. Nix Integration
The project MUST provide a comprehensive Nix flake for reproducible builds, development environments, and distribution.

**Rationale**: Nix ensures reproducible builds across machines, simplifies dependency management, enables consistent development environments, and facilitates distribution.

**Requirements**:
- Provide flake.nix at repository root
- **Packages**: Expose the built CLI binary as default package
- **Overlays**: Provide overlay for integrating into existing Nix configurations
- **Checks**: Include all tests, linting (clippy), formatting (rustfmt) as flake checks
- **DevShell**: Provide development shell with Rust toolchain, cargo tools (clippy, rustfmt, cargo-fuzz, cargo-tarpaulin), and any runtime dependencies
- Lock dependencies using flake.lock for reproducibility
- Support both x86_64-linux and aarch64-linux (and darwin if applicable)
- Document flake usage in README

### VII. Code Quality
Code MUST maintain high quality standards through static analysis, formatting, and review.

**Rationale**: Consistent code quality reduces cognitive load, prevents bugs, and makes maintenance easier.

**Requirements**:
- MUST pass `cargo clippy` with no warnings
- MUST pass `cargo fmt --check` (use standard rustfmt configuration)
- MUST compile with warnings denied in CI: `RUSTFLAGS="-D warnings"`
- Use meaningful variable and function names
- Avoid premature optimization—prioritize clarity
- Prefer explicitness over cleverness
- Document public APIs with rustdoc comments
- Keep functions focused and reasonably sized

### VIII. Documentation
Code MUST be documented for both users and developers.

**Rationale**: Good documentation enables adoption, reduces support burden, and helps future maintainers.

**Requirements**:
- README.md MUST include: project purpose, installation, basic usage examples, development setup
- Public functions and modules MUST have rustdoc comments
- Complex algorithms or non-obvious code MUST have inline comments explaining why, not what
- CLI MUST provide comprehensive --help output
- Document environment variables and configuration files if used
- Keep documentation up-to-date with code changes

## Technology Stack

**Language**: Rust (stable channel)
**Build System**: Cargo
**Package Manager**: Cargo + Nix
**Nix**: Flakes-enabled Nix with flake.nix
**Testing Frameworks**:
- Built-in Rust test framework (`#[test]`, `#[cfg(test)]`)
- cargo-fuzz for fuzz testing
- proptest or quickcheck for property-based testing (optional but recommended)
- assert_cmd and predicates for CLI integration tests
- cargo-tarpaulin for coverage reporting

**Recommended Crates**:
- clap (CLI argument parsing)
- anyhow or thiserror (error handling)
- colored, termcolor, or owo-colors (color support)
- serde (serialization for JSON output)

## Development Workflow

### TDD Cycle
1. **Red**: Write a failing test that describes desired behavior
2. **User Review**: Get test approval before proceeding
3. **Verify Failure**: Confirm test fails for the right reason
4. **Green**: Write minimal code to make test pass
5. **Refactor**: Improve code quality while keeping tests passing
6. **Repeat**: Continue for next behavior

### Testing Gates
- All tests MUST pass locally before committing
- Fuzzing SHOULD be run periodically on critical parsers/inputs
- Integration tests MUST cover main user workflows
- New features MUST include tests before implementation
- Bug fixes MUST include regression tests

### Quality Gates
Before merging any code:
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Fuzz tests run without panics (for affected modules)
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo fmt --check` passes
- [ ] Documentation updated
- [ ] Nix flake checks pass: `nix flake check`

### Code Review
- Verify tests were written first and failed before implementation
- Check test quality: clear, focused, meaningful assertions
- Ensure compliance with all constitution principles
- Validate error handling and edge cases
- Confirm documentation completeness

## Governance

This constitution supersedes all other practices and conventions. Any code, design, or process decision MUST align with these principles.

### Amendment Procedure
1. Propose changes with clear rationale
2. Document impact on existing code and templates
3. Update constitution with new version number
4. Propagate changes to all dependent templates
5. Update agent-file-template.md if technology stack changes
6. Commit with message: `docs: amend constitution to vX.Y.Z (description)`

### Version Control
Version changes follow semantic versioning:
- **MAJOR**: Backward incompatible principle removal or redefinition requiring code changes
- **MINOR**: New principle added or existing principle materially expanded
- **PATCH**: Clarifications, wording improvements, typo fixes

### Compliance Verification
- All feature specifications MUST include constitution compliance check
- Plan phase MUST verify alignment before implementation begins
- Violations MUST be explicitly justified in complexity tracking table
- Regular audits SHOULD verify ongoing compliance

**Version**: 1.0.0 | **Ratified**: 2025-10-23 | **Last Amended**: 2025-10-23
