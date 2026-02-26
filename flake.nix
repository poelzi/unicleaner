{
  description = "Unicleaner - Detect malicious Unicode in source code";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    spec-kitty = {
      url = "github:poelzi/spec-kitty/nix-flake";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      pre-commit-hooks,
      spec-kitty,
    }:
    let
      # Common function to build unicleaner
      mkUnicleaner =
        { pkgs, rustPlatform }:
        rustPlatform.buildRustPackage {
          pname = "unicleaner";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [ ];

          buildInputs = with pkgs; [ ];

          meta = with pkgs.lib; {
            description = "Detect malicious Unicode characters in source code";
            homepage = "https://github.com/poelzi/unicleaner";
            license = with licenses; [
              mit
            ];
            maintainers = [ ];
          };
        };

      # Define overlay at top level
      overlay = final: prev: {
        unicleaner = mkUnicleaner {
          pkgs = prev;
          rustPlatform = prev.rustPlatform;
        };
      };
    in
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Pin toolchain to avoid unexpected breakages.
        rustToolchain = pkgs.rust-bin.stable."1.93.0".default.override {
          extensions = [
            "rust-src"
            "clippy"
            "rustfmt"
          ];
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        unicleaner = mkUnicleaner {
          inherit pkgs rustPlatform;
        };

        # Static musl build for Docker/standalone
        # Use pkgsStatic to ensure all dependencies are statically linked
        unicleaner-static = pkgs.pkgsStatic.rustPlatform.buildRustPackage {
          pname = "unicleaner-static";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          # Add binutils for strip command
          nativeBuildInputs = [ pkgs.pkgsStatic.binutils ];
          buildInputs = [ ];

          # Strip the binary for minimal size
          postInstall = ''
            $STRIP $out/bin/unicleaner
          '';

          meta = with pkgs.lib; {
            description = "Detect malicious Unicode characters in source code (static musl build)";
            homepage = "https://github.com/poelzi/unicleaner";
            license = with licenses; [
              mit
            ];
            maintainers = [ ];
            platforms = [ "x86_64-linux" ];
          };
        };

        # Nightly Rust toolchain for fuzzing
        rustNightly = pkgs.rust-bin.selectLatestNightlyWith (
          toolchain:
          toolchain.default.override {
            extensions = [
              "rust-src"
              "llvm-tools-preview"
            ];
          }
        );

        # Helper function for check derivations
        mkCheck =
          {
            name,
            buildPhase,
            resultMessage,
          }:
          let
            version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          in
          rustPlatform.buildRustPackage {
            pname = "unicleaner-${name}";
            inherit version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [ ];
            buildInputs = with pkgs; [ ];

            buildPhase = ''
              runHook preBuild
              ${buildPhase}
              runHook postBuild
            '';

            checkPhase = ":";
            doCheck = false;

            installPhase = ''
              runHook preInstall
              mkdir -p $out
              echo "${resultMessage}" > $out/${name}-results
              runHook postInstall
            '';
          };

        # Pre-commit hooks configuration
        pre-commit-check = pre-commit-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            # Rust formatting
            rustfmt = {
              enable = true;
              entry = "${rustToolchain}/bin/cargo-fmt fmt -- --check --color always";
              pass_filenames = false;
            };

            # Rust linting
            clippy = {
              enable = true;
              entry = "${rustToolchain}/bin/cargo-clippy clippy --profile test --all-targets --all-features -- -D warnings";
              files = "\\.rs$";
              pass_filenames = false;
            };

            # Cargo check
            cargo-check = {
              enable = true;
              entry = "${rustToolchain}/bin/cargo check --all-features";
              files = "\\.rs$";
              pass_filenames = false;
            };

            # Run tests
            cargo-test = {
              enable = true;
              entry = "${rustToolchain}/bin/cargo test";
              files = "\\.rs$";
              pass_filenames = false;
            };
          };
        };

      in
      {
        packages = {
          default = unicleaner;
          unicleaner = unicleaner;
          unicleaner-static = unicleaner-static;

          # Minimal Docker image with static binary
          docker = pkgs.dockerTools.buildImage {
            name = "unicleaner";
            tag = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
            created = "now";

            copyToRoot = pkgs.buildEnv {
              name = "image-root";
              paths = [ unicleaner-static ];
              pathsToLink = [ "/bin" ];
            };

            config = {
              Entrypoint = [ "/bin/unicleaner" ];
              WorkingDir = "/workspace";
              Volumes = {
                "/workspace" = { };
              };
            };
          };
        };

        apps =
          let
            # Helper to create a fuzz app
            # Note: These correspond to the fuzz targets in .github/workflows/fuzz.yml
            mkFuzzApp = name: {
              type = "app";
              program = "${pkgs.writeShellScript "fuzz-${name}" ''
                set -e
                export PATH="${rustNightly}/bin:${pkgs.cargo-fuzz}/bin:$PATH"
                export CARGO_TARGET_DIR="''${CARGO_TARGET_DIR:-$PWD/target}"

                echo "Running fuzzer: ${name}"
                echo "Corpus directory: fuzz/corpus/${name}"
                echo "Rust version: $(rustc --version)"
                exec cargo fuzz run ${name} -- "$@"
              ''}";
              meta = {
                description = "Fuzz ${name}";
                homepage = "https://github.com/poelzi/unicleaner";
                license = pkgs.lib.licenses.mit;
              };
            };
          in
          {
            # Individual fuzz targets (matching .github/workflows/fuzz.yml matrix)
            fuzz-unicode = mkFuzzApp "fuzz_unicode";
            fuzz-config = mkFuzzApp "fuzz_config";
            fuzz-file-scan = mkFuzzApp "fuzz_file_scan";
            fuzz-encoding = mkFuzzApp "encoding_detection";
            fuzz-homoglyph = mkFuzzApp "homoglyph_detector";

            # Additional security-critical fuzz targets
            fuzz-git-integration = mkFuzzApp "fuzz_git_integration";
            fuzz-walker = mkFuzzApp "fuzz_walker";
            fuzz-parallel-scanner = mkFuzzApp "fuzz_parallel_scanner";
            fuzz-unicode-ranges = mkFuzzApp "fuzz_unicode_ranges";
            fuzz-config-policy = mkFuzzApp "fuzz_config_policy";
            fuzz-glob-patterns = mkFuzzApp "fuzz_glob_patterns";
            fuzz-block-resolve = mkFuzzApp "fuzz_block_resolve";

            # Run all fuzz targets sequentially
            fuzz-all = {
              type = "app";
              program = "${pkgs.writeShellScript "fuzz-all" ''
                set -e
                export PATH="${rustNightly}/bin:${pkgs.cargo-fuzz}/bin:$PATH"
                export CARGO_TARGET_DIR="''${CARGO_TARGET_DIR:-$PWD/target}"

                FUZZ_TARGETS=(
                  "fuzz_unicode"
                  "fuzz_config"
                  "fuzz_file_scan"
                  "encoding_detection"
                  "homoglyph_detector"
                  "fuzz_git_integration"
                  "fuzz_walker"
                  "fuzz_parallel_scanner"
                  "fuzz_unicode_ranges"
                  "fuzz_config_policy"
                  "fuzz_glob_patterns"
                  "fuzz_block_resolve"
                )

                echo "Running all fuzz targets with timeout of 60s each..."
                echo "Rust version: $(rustc --version)"
                echo "==============================================="

                FAILED_TARGETS=()

                for target in "''${FUZZ_TARGETS[@]}"; do
                  echo ""
                  echo ">>> Running: $target"
                  echo "---"
                  if ! cargo fuzz run "$target" -- -max_total_time=60 "$@"; then
                    echo "Warning: Fuzzer $target exited with code $?"
                    FAILED_TARGETS+=("$target")
                  fi
                done

                echo ""
                echo "==============================================="
                if [ ''${#FAILED_TARGETS[@]} -gt 0 ]; then
                  echo "FAILED fuzz targets (''${#FAILED_TARGETS[@]}):"
                  for t in "''${FAILED_TARGETS[@]}"; do
                    echo "  - $t"
                  done
                  exit 1
                else
                  echo "All fuzz targets completed successfully!"
                fi
              ''}";
            };

            # Coverage report generation
            coverage = {
              type = "app";
              program = "${pkgs.writeShellScript "coverage" ''
                set -e
                export PATH="${rustToolchain}/bin:$PATH"

                echo "Generating code coverage report with cargo-tarpaulin..."
                echo "This may take a few minutes..."
                echo ""

                # Ensure cargo-tarpaulin is available
                if ! command -v cargo-tarpaulin &> /dev/null; then
                  echo "Installing cargo-tarpaulin..."
                  cargo install cargo-tarpaulin
                fi

                # Run tarpaulin with various output formats
                cargo tarpaulin \
                  --out Html \
                  --out Xml \
                  --out Lcov \
                  --output-dir coverage \
                  --exclude-files 'fuzz/*' \
                  --exclude-files 'target/*' \
                  --all-features \
                  --workspace \
                  --timeout 300 \
                  "$@"

                echo ""
                echo "==============================================="
                echo "Coverage report generated!"
                echo "  HTML: coverage/tarpaulin-report.html"
                echo "  XML:  coverage/cobertura.xml"
                echo "  LCOV: coverage/lcov.info"
                echo ""
                echo "Open coverage/tarpaulin-report.html in your browser"
              ''}";
            };

            # Quick coverage summary (faster, less detailed)
            coverage-summary = {
              type = "app";
              program = "${pkgs.writeShellScript "coverage-summary" ''
                set -e
                export PATH="${rustToolchain}/bin:$PATH"

                # Ensure cargo-tarpaulin is available
                if ! command -v cargo-tarpaulin &> /dev/null; then
                  echo "Installing cargo-tarpaulin..."
                  cargo install cargo-tarpaulin
                fi

                echo "Running quick coverage analysis..."
                cargo tarpaulin \
                  --out Stdout \
                  --exclude-files 'fuzz/*' \
                  --all-features \
                  --workspace \
                  --timeout 120
              ''}";
            };
          };

        checks = {
          # Main build check
          build = unicleaner;

          # Test check
          test = mkCheck {
            name = "test";
            buildPhase = "cargo test --all --no-fail-fast";
            resultMessage = "Tests passed";
          };

          # Clippy check
          clippy = mkCheck {
            name = "clippy";
            buildPhase = "cargo clippy --profile test --all-targets --all-features -- -D warnings";
            resultMessage = "Clippy checks passed";
          };

          # Format check
          fmt = mkCheck {
            name = "fmt";
            buildPhase = "cargo fmt --all -- --check";
            resultMessage = "Format checks passed";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            devenv

            # Development tools
            rust-analyzer
            cargo-edit
            cargo-watch
            cargo-tarpaulin
            cargo-fuzz
            spec-kitty.packages.${system}.default

            # For fuzzing (requires nightly)
            (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default))
          ];

          shellHook = ''
            ${pre-commit-check.shellHook}
            echo "🦀 Unicleaner development environment"
            echo "Rust version: $(rustc --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build          - Build the project"
            echo "  cargo test           - Run all tests"
            echo "  cargo clippy         - Run linter"
            echo "  cargo fmt            - Format code"
            echo "  cargo run -- [args]  - Run unicleaner"
            echo "  cargo tarpaulin      - Code coverage"
            echo "  cargo +nightly fuzz  - Run fuzzing (nightly)"
            echo ""
            echo "Pre-commit hooks are installed!"
            echo "  Run 'pre-commit run --all-files' to check all files"
            echo ""
          '';
        };
      }
    )
    // {
      # Top-level outputs (not per-system)
      overlays.default = overlay;
    };
}
