# Nix Build System Guide

This document explains the Nix-based build system for Unicleaner, including static binary creation and Docker image generation.

## Overview

Unicleaner uses Nix Flakes for reproducible builds with three main build targets:

1. **`unicleaner`** - Standard dynamically-linked binary (glibc)
2. **`unicleaner-static`** - Statically-linked musl binary (no dependencies)
3. **`docker`** - Minimal Docker image with static binary

## Build Targets

### Standard Build

```bash
nix build .#unicleaner
# Result: result/bin/unicleaner (dynamically linked with glibc)
```

Uses the standard Rust toolchain with glibc for maximum compatibility with development tools.

### Static Musl Build

```bash
nix build .#unicleaner-static
# Result: result/bin/unicleaner (statically linked, no dependencies)
```

**Key Features:**
- Uses `pkgsStatic.rustPlatform` for automatic musl compilation
- Target: `x86_64-unknown-linux-musl`
- Fully static - no runtime dependencies
- Perfect for Docker containers and minimal deployments
- Binary size: ~5-10 MB (stripped)

**Verification:**
```bash
ldd result/bin/unicleaner
# Should output: "not a dynamic executable" or "statically linked"

file result/bin/unicleaner
# Should show: "statically linked"
```

### Docker Image

```bash
nix build .#docker
# Result: Docker image tarball

# Load and use:
docker load < result
docker run unicleaner:1.0.0-alpha1 --version
```

**Image Details:**
- Base: `FROM scratch` (no OS, only the binary)
- Size: ~5-10 MB total
- Contains only the static musl binary
- No shell, no utilities - maximum security
- Working directory: `/workspace`
- Volume mount point: `/workspace`

## Flake Structure

### Packages

```nix
{
  packages = {
    default = unicleaner;           # Standard build
    unicleaner = unicleaner;        # Standard build (explicit)
    unicleaner-static = ...;        # Static musl build
    docker = ...;                   # Docker image
  };
}
```

### Apps

Fuzz testing apps (requires nightly Rust):
- `fuzz-unicode`, `fuzz-config`, `fuzz-file-scan`
- `fuzz-encoding`, `fuzz-homoglyph`
- `fuzz-git-integration`, `fuzz-walker`
- `fuzz-parallel-scanner`, `fuzz-unicode-ranges`
- `fuzz-config-policy`, `fuzz-glob-patterns`
- `fuzz-all` - Run all fuzz targets

Coverage apps:
- `coverage` - Generate full coverage report
- `coverage-summary` - Quick coverage summary

### Checks

```bash
nix flake check  # Run all checks

# Individual checks:
nix build .#checks.x86_64-linux.test
nix build .#checks.x86_64-linux.clippy
nix build .#checks.x86_64-linux.fmt
```

## Static Build Implementation

The static build uses a simple approach recommended by the NixOS community:

```nix
unicleaner-static = pkgs.pkgsStatic.rustPlatform.buildRustPackage {
  pname = "unicleaner-static";
  # ... standard Rust package configuration
  nativeBuildInputs = [ pkgs.pkgsStatic.binutils ];  # For strip
};
```

**Why pkgsStatic?**
- Automatically handles musl libc linking
- All transitive dependencies are static
- No manual target or linker configuration needed
- Works out-of-the-box for pure Rust projects

**Reference:** [NixOS Discourse - Building Rust packages against musl](https://discourse.nixos.org/t/building-rust-packages-against-musl/21345)

## CI/CD Integration

### GitHub Actions

The workflows use Nix/Cachix for reproducible builds:

```yaml
# ci.yml
- name: Build static binary
  run: nix build .#unicleaner-static --print-build-logs

# release.yml
- name: Build Docker image
  run: |
    nix build .#docker --print-build-logs
    docker load < result
```

**Benefits:**
- Binary caching with Cachix (instant rebuilds)
- Same build everywhere (local, CI, prod)
- No "works on my machine" issues
- Deterministic builds

### Cachix Setup

1. Create Cachix cache: https://app.cachix.org/
2. Add auth token to GitHub secrets: `CACHIX_AUTH_TOKEN`
3. Workflows automatically push/pull from cache

**First build:** Slow (compiles everything)
**Subsequent builds:** Fast (downloads from cache)

## Development Workflow

### Using the Dev Shell

```bash
nix develop
# Enters shell with:
# - Rust stable toolchain
# - All build dependencies
# - Development tools (rust-analyzer, cargo-edit, etc.)
# - Pre-commit hooks installed
```

### Building Locally

```bash
# Quick iteration during development
cargo build
cargo test

# Build for release
nix build .#unicleaner

# Build static for Docker
nix build .#unicleaner-static

# Verify static
./scripts/verify-static.sh result/bin/unicleaner
```

### Testing Static Binary

```bash
# Build
nix build .#unicleaner-static

# Run verification script
./scripts/verify-static.sh result/bin/unicleaner

# Manual checks
ldd result/bin/unicleaner        # Should fail or say "statically linked"
file result/bin/unicleaner       # Check ELF info
result/bin/unicleaner --version  # Smoke test
```

## Docker Usage

### Building and Loading

```bash
# Build image
nix build .#docker

# Load into Docker
docker load < result

# Check loaded image
docker images | grep unicleaner
```

### Running

```bash
# Scan current directory
docker run --rm -v "$(pwd):/workspace" unicleaner:1.0.0-alpha1 .

# With custom config
docker run --rm \
  -v "$(pwd):/workspace" \
  -v "$(pwd)/config.toml:/config.toml:ro" \
  unicleaner:1.0.0-alpha1 scan . --config /config.toml
```

### Publishing

```bash
# Tag for registry
docker tag unicleaner:1.0.0-alpha1 ghcr.io/username/unicleaner:latest

# Push
docker push ghcr.io/username/unicleaner:latest
```

## Troubleshooting

### Build Failures

**Issue:** Strip command not found
**Fix:** Added `pkgs.pkgsStatic.binutils` to `nativeBuildInputs`

**Issue:** Binary still dynamically linked
**Fix:** Use `pkgsStatic.rustPlatform` instead of custom cross-compilation

### Cache Issues

```bash
# Clear local build cache
rm -rf result result-*

# Force rebuild
nix build .#unicleaner-static --rebuild

# Clear all Nix build artifacts
nix-collect-garbage -d
```

### Docker Issues

**Issue:** Permission denied in container
```bash
# Run as current user
docker run --user $(id -u):$(id -g) ...
```

**Issue:** Can't see files in /workspace
```bash
# Use absolute paths
docker run -v "$PWD:/workspace" ...
```

## Performance

### Build Times

**First build (no cache):**
- Standard build: ~5-10 minutes
- Static build: ~10-15 minutes (includes musl toolchain)
- Docker image: ~10-15 minutes

**With Cachix (cache hit):**
- All builds: <1 minute (download from cache)

### Binary Sizes

- Standard (dynamically linked): ~3 MB
- Static (musl): ~5-10 MB
- Docker image: ~5-10 MB (scratch + static binary)

**Size optimization:**
- Release profile: LTO enabled, strip enabled
- Cargo.toml: `lto = true`, `strip = true`, `opt-level = 3`
- Post-build strip with binutils

## Future Improvements

### Multi-Architecture Support

```nix
# TODO: Add ARM64 static build
unicleaner-static-aarch64 = pkgsStatic.rustPlatform.buildRustPackage {
  # Cross-compile for ARM64
  CARGO_BUILD_TARGET = "aarch64-unknown-linux-musl";
};
```

### Multi-arch Docker

```bash
# TODO: Build for multiple architectures
docker buildx build --platform linux/amd64,linux/arm64 ...
```

## References

- [Nix Flakes Manual](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html)
- [nixpkgs Manual - Rust](https://nixos.org/manual/nixpkgs/stable/#rust)
- [Building Rust with musl (Discourse)](https://discourse.nixos.org/t/building-rust-packages-against-musl/21345)
- [Docker Tools (nixpkgs)](https://nixos.org/manual/nixpkgs/stable/#sec-pkgs-dockerTools)
- [Cachix Documentation](https://docs.cachix.org/)

## Quick Reference

```bash
# Development
nix develop                                # Enter dev shell
cargo build                                # Quick build
cargo test                                 # Run tests

# Production builds
nix build .#unicleaner                     # Standard build
nix build .#unicleaner-static             # Static build
nix build .#docker                        # Docker image

# Verification
./scripts/verify-static.sh result/bin/unicleaner

# CI/CD
nix flake check                           # Run all checks
nix build .#checks.x86_64-linux.test     # Run tests
nix build .#checks.x86_64-linux.clippy   # Run clippy

# Docker
docker load < result                       # Load image
docker run unicleaner:1.0.0-alpha1 --help # Run container

# Cleanup
rm -rf result*                            # Remove build artifacts
nix-collect-garbage -d                    # Deep clean
```
