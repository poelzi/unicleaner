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
unicleaner scan .

# Generate default config
unicleaner init

# Scan with custom config
unicleaner scan . --config unicleaner.toml

# Scan only Git changes (for CI/CD)
unicleaner scan . --diff

# Output JSON for machine parsing
unicleaner scan . --format json

# Filter by severity level
unicleaner scan . --severity error

# Control color output
unicleaner scan . --color always
unicleaner scan . --color never
unicleaner scan . --no-color  # deprecated but supported

# Quiet mode (summary only)
unicleaner scan . --quiet

# Verbose mode (show progress)
unicleaner scan . --verbose

# List available language presets
unicleaner list-presets
```

## CLI Reference

### Commands

- `scan [PATH]` - Scan files for malicious Unicode (default command)
- `init [FILE]` - Generate a default configuration file
- `list-presets` - Show available language presets

### Global Flags

- `-c, --config <FILE>` - Path to configuration file
- `-f, --format <FORMAT>` - Output format: human, json, github, gitlab (default: human)
- `--color <WHEN>` - Color output: auto, always, never (default: auto)
- `--no-color` - Disable color output (deprecated, use --color=never)
- `-q, --quiet` - Show only summary (suppress individual violations)
- `-v, --verbose` - Show verbose output with progress messages
- `--severity <LEVEL>` - Minimum severity to report: error, warning, info

### Scan Flags

- `--diff` - Scan only files changed in Git (requires Git repository)
- `-j, --jobs <N>` - Number of parallel threads (default: number of CPUs)
- `--encoding <ENC>` - Force specific encoding: utf8, utf16-le, utf16-be, utf32-le, utf32-be

## Exit Codes

- `0` - Success: No violations found
- `1` - Violations found
- `2` - Error: Invalid arguments, file read errors, etc.

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
