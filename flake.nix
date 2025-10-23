{
  description = "Unicleaner - Detect malicious Unicode in source code";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
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

      in
      {
        packages = {
          default = unicleaner;
          unicleaner = unicleaner;
        };

        overlays.default = final: prev: {
          unicleaner = unicleaner;
        };

        checks = {
          build = unicleaner;

          test = pkgs.runCommand "unicleaner-test" {
            buildInputs = [ rustToolchain ];
          } ''
            cd ${./.}
            cargo test --all
            touch $out
          '';

          clippy = pkgs.runCommand "unicleaner-clippy" {
            buildInputs = [ rustToolchain ];
          } ''
            cd ${./.}
            cargo clippy --all-targets --all-features -- -D warnings
            touch $out
          '';

          fmt = pkgs.runCommand "unicleaner-fmt" {
            buildInputs = [ rustToolchain ];
          } ''
            cd ${./.}
            cargo fmt --all -- --check
            touch $out
          '';
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
          '';
        };
      }
    );
}
