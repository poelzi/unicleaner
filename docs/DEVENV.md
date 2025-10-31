# devenv.sh Development Environment

This project uses [devenv.sh](https://devenv.sh) for a reproducible development environment with integrated tooling and pre-commit hooks.

## Quick Start

### Installation

1. Install Nix (if not already installed):
```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

2. Install devenv:
```bash
nix profile install nixpkgs#devenv
```

3. Enter the development environment:
```bash
devenv shell
```

The first run will take a few minutes to download and build all dependencies. Subsequent runs are instant thanks to Cachix caching.

## Features

### Automatic Setup

When you enter the devenv shell, you get:

- ✅ **Rust stable toolchain** with all components (rustc, cargo, clippy, rustfmt, rust-src)
- ✅ **Development tools** (rust-analyzer, cargo-edit, cargo-watch, cargo-tarpaulin)
- ✅ **Pre-commit hooks** automatically installed and configured
- ✅ **Helper scripts** for common tasks
- ✅ **Cachix integration** for binary caching

### Custom Scripts

devenv provides convenient scripts for common tasks:

```bash
# Build static musl binary
build-static

# Build Docker image
build-docker

# Verify static binary has no dependencies
verify-static

# Generate code coverage report
coverage

# Run fuzzer (default target: fuzz_unicode, 60s)
fuzz
fuzz fuzz-config 120  # Custom target and duration
```

### Pre-commit Hooks

The following hooks run automatically on `git commit`:

- **rustfmt** - Enforces code formatting
- **clippy** - Rust linter
- **cargo-check** - Checks compilation
- **nixpkgs-fmt** - Formats Nix files
- **trailing-whitespace** - Removes trailing whitespace
- **end-of-file-fixer** - Ensures files end with newline

To run hooks manually on all files:
```bash
devenv shell
git add .
pre-commit run --all-files
```

## Usage

### Daily Development

```bash
# Enter the development shell
devenv shell

# Standard Rust workflow
cargo build
cargo test
cargo clippy
cargo fmt

# Or use devenv test (runs tests with pre-commit hooks)
devenv test
```

### Running Commands Without Entering Shell

```bash
# Run a single command
devenv shell cargo test

# Run multiple commands
devenv shell bash -c "cargo build && cargo test"
```

### Environment Variables

The following environment variables are set automatically:

- `CARGO_TERM_COLOR=always` - Always colorize cargo output

## Configuration

### devenv.nix

The main configuration file that defines:
- Languages and toolchains
- Packages and development tools
- Scripts and tasks
- Pre-commit hooks
- Shell initialization

### devenv.yaml

Project metadata and input sources:
```yaml
inputs:
  nixpkgs:
    url: github:NixOS/nixpkgs/nixos-unstable
```

## Cachix Integration

This project uses Cachix for binary caching to speed up builds:

- **Cache name**: `unicleaner`
- **Push enabled**: Automatically pushes build artifacts (requires auth token)
- **Public cache**: Available at https://app.cachix.org/cache/unicleaner

### Using the Cache

The cache is automatically configured when you enter the devenv shell. To manually use it:

```bash
cachix use unicleaner
```

### Contributing to Cache

If you have push access, set up your auth token:

```bash
cachix authtoken <YOUR_TOKEN>
```

Or set it in GitHub Actions secrets as `CACHIX_AUTH_TOKEN`.

## GitHub Actions Integration

The CI workflows use devenv for consistent environments:

### Test Workflow

```yaml
- name: Install devenv
  run: nix profile install nixpkgs#devenv

- name: Build the devenv shell and run tests
  run: devenv test
```

### Running Commands in CI

```yaml
# Single command
- name: Run clippy
  shell: devenv shell bash -- -e {0}
  run: cargo clippy --all-targets --all-features -- -D warnings

# Multiple commands in same shell
- name: Build and test
  shell: devenv shell bash -- -e {0}
  run: |
    cargo build --all-features
    cargo test --all-features
```

## Advanced Usage

### Custom Tasks

Add tasks to `devenv.nix`:

```nix
tasks = {
  "myproject:build".exec = "cargo build --release";
  "devenv:enterShell".after = [ "myproject:build" ];
};
```

### Processes

Run long-running processes in the background:

```nix
processes.cargo-watch.exec = "cargo-watch -x test";
```

Then use:
```bash
devenv up  # Start all processes
```

### Services

Enable services like PostgreSQL:

```nix
services.postgres = {
  enable = true;
  initialDatabases = [{ name = "unicleaner_dev"; }];
};
```

## Troubleshooting

### Shell doesn't activate

```bash
# Clean and rebuild
devenv gc
devenv shell
```

### Pre-commit hooks not running

```bash
# Reinstall hooks
devenv shell
pre-commit install
```

### Cache not working

```bash
# Verify Cachix setup
cachix use unicleaner

# Check cache status
nix-store --verify --check-contents
```

### Slow initial setup

The first `devenv shell` downloads and builds everything. This is normal and only happens once. Subsequent runs are instant.

To see what's being built:
```bash
devenv shell --impure --show-trace
```

## Comparison with Nix Flakes

Both devenv and Nix flakes work together:

| Feature | Nix Flakes | devenv |
|---------|-----------|--------|
| **Reproducible builds** | ✅ | ✅ |
| **Development shell** | ✅ (`nix develop`) | ✅ (`devenv shell`) |
| **Pre-commit hooks** | Manual setup | ✅ Built-in |
| **Custom scripts** | Manual | ✅ Easy configuration |
| **Process management** | ❌ | ✅ `devenv up` |
| **Services** | Manual | ✅ Built-in |
| **CI Integration** | ✅ | ✅ Optimized |

**Recommendation**: Use both!
- **devenv** for daily development (better DX, pre-commit hooks, scripts)
- **Nix flakes** for CI/CD and production builds (reproducibility, cachability)

## Migration from `nix develop`

If you were using `nix develop` before:

### Before (Nix flakes)
```bash
nix develop
cargo build
```

### After (devenv)
```bash
devenv shell
cargo build
```

**Advantages with devenv:**
- Pre-commit hooks automatically installed
- Custom scripts available (`build-static`, `coverage`, etc.)
- Better shell initialization messages
- Easier to configure and maintain

Both environments are equivalent - use whichever you prefer!

## Resources

- [devenv.sh Documentation](https://devenv.sh/)
- [devenv GitHub](https://github.com/cachix/devenv)
- [Cachix Documentation](https://docs.cachix.org/)
- [Nix Flakes Manual](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html)

## Quick Reference

```bash
# Enter development shell
devenv shell

# Run tests with pre-commit hooks
devenv test

# Run single command
devenv shell cargo test

# Start background processes
devenv up

# Stop processes
devenv down

# Clean environment
devenv gc

# Update dependencies
devenv update

# Custom scripts
build-static      # Build static binary
build-docker      # Build Docker image
verify-static     # Verify static binary
coverage          # Generate coverage report
fuzz [target]     # Run fuzzer
```
