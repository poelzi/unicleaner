{ pkgs, lib, config, inputs, ... }:

{
  # Project metadata
  name = "unicleaner";

  # https://devenv.sh/basics/
  env = {
    CARGO_TERM_COLOR = "always";
  };

  # https://devenv.sh/packages/
  packages = with pkgs; [
    git

    # Rust development tools
    rust-analyzer
    cargo-edit
    cargo-watch
    cargo-tarpaulin

    # Nix tools
    nixpkgs-fmt

    # Development utilities
    jq
  ];

  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
    channel = "stable";
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-src" ];
  };

  # https://devenv.sh/processes/
  # processes.cargo-watch.exec = "cargo-watch -x test";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/scripts/
  scripts = {
    build-static.exec = ''
      echo "Building static musl binary..."
      nix build .#unicleaner-static
    '';

    build-docker.exec = ''
      echo "Building Docker image..."
      nix build .#docker
    '';

    verify-static.exec = ''
      if [ -f result/bin/unicleaner ]; then
        echo "Verifying static binary..."
        file result/bin/unicleaner
        ldd result/bin/unicleaner 2>&1 || echo "✅ Static binary confirmed"
      else
        echo "❌ No binary found. Run 'build-static' first."
        exit 1
      fi
    '';

    coverage.exec = ''
      echo "Generating code coverage..."
      cargo tarpaulin \
        --out Html \
        --out Xml \
        --output-dir coverage \
        --exclude-files 'fuzz/*' \
        --all-features \
        --workspace \
        --timeout 300
      echo "Coverage report: coverage/tarpaulin-report.html"
    '';

    fuzz.exec = ''
      target="''${1:-fuzz-unicode}"
      duration="''${2:-60}"
      echo "Running fuzzer: $target for ''${duration}s..."
      nix run .#$target -- -max_total_time=$duration
    '';
  };

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests..."
    cargo test --all-features --workspace
  '';

  # https://devenv.sh/pre-commit-hooks/
  pre-commit.hooks = {
    # Rust
    rustfmt.enable = true;
    clippy.enable = true;
    cargo-check.enable = true;

    # Nix
    nixpkgs-fmt.enable = true;

    # General
    trailing-whitespace = {
      enable = true;
      name = "trim trailing whitespace";
      entry = "${pkgs.python3Packages.pre-commit-hooks}/bin/trailing-whitespace-fixer";
      types = [ "text" ];
    };

    end-of-file-fixer = {
      enable = true;
      name = "fix end of files";
      entry = "${pkgs.python3Packages.pre-commit-hooks}/bin/end-of-file-fixer";
      types = [ "text" ];
    };
  };

  # https://devenv.sh/integrations/cachix/
  cachix = {
    enable = true;
    push = "unicleaner";
  };

  # https://devenv.sh/integrations/dotenv/
  dotenv.enable = false;

  # Shell hook - runs when entering the devenv shell
  enterShell = ''
    echo ""
    echo "🦀 Unicleaner Development Environment"
    echo "======================================"
    echo ""
    echo "Rust version: $(rustc --version)"
    echo "Cargo version: $(cargo --version)"
    echo ""
    echo "Available scripts:"
    echo "  build-static     - Build static musl binary"
    echo "  build-docker     - Build Docker image"
    echo "  verify-static    - Verify static binary has no dependencies"
    echo "  coverage         - Generate code coverage report"
    echo "  fuzz [target]    - Run fuzzer (default: fuzz-unicode)"
    echo ""
    echo "Standard commands:"
    echo "  cargo build      - Build the project"
    echo "  cargo test       - Run all tests"
    echo "  cargo clippy     - Run linter"
    echo "  cargo fmt        - Format code"
    echo "  devenv test      - Run test suite with pre-commit hooks"
    echo ""
    echo "Nix commands:"
    echo "  nix build .#unicleaner        - Build standard binary"
    echo "  nix build .#unicleaner-static - Build static musl binary"
    echo "  nix build .#docker            - Build Docker image"
    echo "  nix flake check               - Run all Nix checks"
    echo ""
    echo "Pre-commit hooks are enabled!"
    echo ""
  '';
}
