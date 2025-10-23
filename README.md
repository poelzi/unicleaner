# Unicleaner

**Detect malicious Unicode characters in source code**

Unicleaner is a security-focused CLI tool that scans source code repositories to detect potentially malicious Unicode characters that could hide backdoors or exploits, including:

- **Zero-width characters** (U+200B, U+200C, U+200D, U+FEFF)
- **Bidirectional override characters** (U+202A-U+202E) - Trojan Source attacks
- **Homoglyphs** - visually similar characters from different scripts
- **Non-printable control characters** outside standard ASCII range

## Features

- 🔒 **Deny-by-default security** - only explicitly allowed characters pass
- ⚙️ **Configurable** - TOML-based configuration with language presets
- 🚀 **Fast** - parallel scanning with Rayon
- 🎨 **Colored output** - human-readable terminal output with automatic TTY detection
- 📊 **JSON output** - machine-parseable format for CI/CD integration
- 🔄 **Git integration** - scan only changed files in pull requests
- 🌍 **Multilingual support** - 50+ language presets for legitimate Unicode

## Installation

### Using Cargo

```bash
cargo install unicleaner
```

### Using Nix

```bash
nix run github:yourusername/unicleaner
```

### From Source

```bash
git clone https://github.com/yourusername/unicleaner
cd unicleaner
cargo build --release
./target/release/unicleaner --version
```

## Quick Start

```bash
# Scan current directory
unicleaner .

# Generate default config
unicleaner init

# Scan with custom config
unicleaner . --config unicleaner.toml

# Scan only Git changes
unicleaner . --diff

# Output JSON for CI
unicleaner . --json
```

## Documentation

- [Quickstart Guide](specs/001-unicode-malicious-detector/quickstart.md)
- [Configuration Examples](examples/unicleaner.toml)
- [CI/CD Integration](examples/)

## Development

### Setup

```bash
# Using Nix
nix develop

# Or install Rust manually
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build and Test

```bash
cargo build
cargo test
cargo clippy
cargo fmt
```

### Run

```bash
cargo run -- .
```

## License

Dual-licensed under MIT OR Apache-2.0

## Security

This tool helps detect Unicode-based security vulnerabilities. For security issues in the tool itself, please report responsibly via GitHub Security Advisories.

## References

- [Trojan Source: Invisible Vulnerabilities](https://trojansource.codes/)
- [Unicode Security Mechanisms (TR39)](https://unicode.org/reports/tr39/)
- [Unicode Confusables](https://www.unicode.org/Public/security/latest/confusables.txt)
