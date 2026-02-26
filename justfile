# Unicleaner development commands
# Run `just` to see all available recipes

set shell := ["bash", "-uc"]

# List available recipes
default:
    @just --list

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

# Build in debug mode
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Build static musl binary via Nix
build-static:
    nix build .#unicleaner-static

# Build Docker image via Nix
build-docker:
    nix build .#docker

# Verify static binary has no dynamic dependencies
verify-static:
    @if [ -f result/bin/unicleaner ]; then \
        file result/bin/unicleaner; \
        ldd result/bin/unicleaner 2>&1 || echo "Static binary confirmed"; \
    else \
        echo "No binary found. Run 'just build-static' first."; \
        exit 1; \
    fi

# ---------------------------------------------------------------------------
# Test
# ---------------------------------------------------------------------------

# Run all tests
test:
    cargo test --all-features --workspace

# Run unit tests only
test-unit:
    cargo test --lib

# Run integration tests only
test-integration:
    cargo test --test integration

# Run doc tests only
test-doc:
    cargo test --doc

# Run property-based tests with extended cases
test-proptest cases="1000":
    PROPTEST_CASES={{cases}} cargo test --test integration proptest

# ---------------------------------------------------------------------------
# Lint & Format
# ---------------------------------------------------------------------------

# Run clippy with deny warnings
check:
    cargo clippy --all-targets --all-features -- -D warnings

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check

# ---------------------------------------------------------------------------
# Pre-commit & CI
# ---------------------------------------------------------------------------

# Run all pre-commit hooks on all files
pre-commit:
    pre-commit run --all-files

# Full CI pipeline: format check, lint, then test
ci: fmt-check check test

# ---------------------------------------------------------------------------
# Coverage
# ---------------------------------------------------------------------------

# Generate code coverage report (HTML + XML)
coverage:
    cargo tarpaulin \
        --out Html \
        --out Xml \
        --out Lcov \
        --output-dir coverage \
        --exclude-files 'fuzz/*' \
        --all-features \
        --workspace \
        --timeout 300
    @echo "Coverage report: coverage/tarpaulin-report.html"

# ---------------------------------------------------------------------------
# Fuzzing
# ---------------------------------------------------------------------------

# Run a fuzz target (default: fuzz-unicode, duration: 60s)
fuzz target="fuzz-unicode" duration="60":
    nix run .#{{target}} -- -max_total_time={{duration}}

# Run all fuzz targets sequentially
fuzz-all:
    nix run .#fuzz-all

# ---------------------------------------------------------------------------
# Packaging & Publishing
# ---------------------------------------------------------------------------

# List files that would be included in the crate
package-list:
    cargo package --allow-dirty --list

# Dry-run package build
package:
    cargo package --allow-dirty

# Dry-run publish (validates everything without uploading)
publish-dry:
    cargo publish --dry-run

# Publish to crates.io (requires authentication)
publish:
    cargo publish

# ---------------------------------------------------------------------------
# Nix
# ---------------------------------------------------------------------------

# Run Nix flake checks
nix-check:
    nix flake check

# Build with Nix (standard binary)
nix-build:
    nix build .#unicleaner

# ---------------------------------------------------------------------------
# Cleanup
# ---------------------------------------------------------------------------

# Remove build artifacts
clean:
    cargo clean
