{
  description = "Unicleaner - Detect malicious Unicode in source code";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, pre-commit-hooks }:
    let
      # Define overlay at top level
      overlay = final: prev: {
        unicleaner = prev.callPackage ({ rustPlatform, pkg-config, openssl, lib, darwin }:
          rustPlatform.buildRustPackage {
            pname = "unicleaner";
            version = "1.0.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [ pkg-config ];

            buildInputs = [ openssl ] ++ lib.optionals prev.stdenv.isDarwin [
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.SystemConfiguration
            ];

            meta = with lib; {
              description = "Detect malicious Unicode characters in source code";
              homepage = "https://github.com/yourusername/unicleaner";
              license = with licenses; [ mit asl20 ];
              maintainers = [ ];
            };
          }
        ) {};
      };
    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        unicleaner = rustPlatform.buildRustPackage {
          pname = "unicleaner";
          version = "1.0.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          meta = with pkgs.lib; {
            description = "Detect malicious Unicode characters in source code";
            homepage = "https://github.com/yourusername/unicleaner";
            license = with licenses; [ mit asl20 ];
            maintainers = [ ];
          };
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
              entry = "${rustToolchain}/bin/cargo-clippy clippy --all-targets --all-features -- -D warnings";
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
        };

        checks = {
          # Main build check
          build = unicleaner;

          # Pre-commit hooks
          pre-commit = pre-commit-check;

          # Test check - reuses the main package's cargo artifacts
          test = rustPlatform.buildRustPackage {
            pname = "unicleaner-test";
            version = "1.0.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ openssl ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];

            # Override build and check phases
            buildPhase = ''
              runHook preBuild
              cargo test --all --no-fail-fast
              runHook postBuild
            '';

            checkPhase = ":";  # Skip default check phase
            doCheck = false;

            installPhase = ''
              runHook preInstall
              mkdir -p $out
              echo "Tests passed" > $out/test-results
              runHook postInstall
            '';
          };

          # Clippy check
          clippy = rustPlatform.buildRustPackage {
            pname = "unicleaner-clippy";
            version = "1.0.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ openssl ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];

            buildPhase = ''
              runHook preBuild
              cargo clippy --all-targets --all-features -- -D warnings
              runHook postBuild
            '';

            checkPhase = ":";
            doCheck = false;

            installPhase = ''
              runHook preInstall
              mkdir -p $out
              echo "Clippy checks passed" > $out/clippy-results
              runHook postInstall
            '';
          };

          # Format check - simpler since it doesn't need compilation
          fmt = rustPlatform.buildRustPackage {
            pname = "unicleaner-fmt";
            version = "1.0.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ openssl ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];

            buildPhase = ''
              runHook preBuild
              cargo fmt --all -- --check
              runHook postBuild
            '';

            checkPhase = ":";
            doCheck = false;

            installPhase = ''
              runHook preInstall
              mkdir -p $out
              echo "Format checks passed" > $out/fmt-results
              runHook postInstall
            '';
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl

            # Development tools
            rust-analyzer
            cargo-edit
            cargo-watch
            cargo-tarpaulin
            cargo-fuzz

            # For fuzzing (requires nightly)
            (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default))
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
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
    ) // {
      # Top-level outputs (not per-system)
      overlays.default = overlay;
    };
}
