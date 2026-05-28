{
  description = "koban - Invoice Ninja from the terminal";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.devshell.flakeModule
        inputs.treefmt-nix.flakeModule
      ];

      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem =
        {
          system,
          lib,
          self',
          ...
        }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;
          src = craneLib.cleanCargoSource ./.;

          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

          meta = {
            description = cargoToml.package.description;
            homepage = cargoToml.package.homepage;
            license = lib.licenses.mit;
            mainProgram = "koban";
            maintainers = [
              {
                name = "James Brink";
                email = "brink.james@gmail.com";
                github = "jamesbrink";
                githubId = 28793;
              }
            ];
            platforms = lib.platforms.unix;
          };

          commonArgs = {
            inherit src;
            pname = cargoToml.package.name;
            version = cargoToml.package.version;
            strictDeps = true;
            buildInputs = lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];
            inherit meta;
          }
          // lib.optionalAttrs pkgs.stdenv.isDarwin {
            LIBRARY_PATH = "${pkgs.libiconv}/lib";
            NIX_LDFLAGS = "-L${pkgs.libiconv}/lib";
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          koban = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
        in
        {
          packages = {
            inherit koban;
            default = koban;
          };

          apps = {
            koban = {
              type = "app";
              program = "${koban}/bin/koban";
              inherit meta;
            };
            default = self'.apps.koban;
          };

          checks = {
            inherit koban;
          };

          devshells.default = {
            motd = ''
              {202}koban{reset} — Invoice Ninja from the terminal ({bold}${system}{reset})
              $(type menu &>/dev/null && menu)
            '';

            packages = [
              rustToolchain
              pkgs.rust-analyzer
              pkgs.cargo-llvm-cov
              pkgs.git
              pkgs.gh
              pkgs.jq
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];

            env = [
              {
                name = "RUST_BACKTRACE";
                value = "1";
              }
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              {
                name = "LIBRARY_PATH";
                value = "${pkgs.libiconv}/lib";
              }
            ];

            commands = [
              {
                category = "build";
                name = "build";
                help = "cargo build (debug)";
                command = "cargo build \"$@\"";
              }
              {
                category = "build";
                name = "build-release";
                help = "cargo build --release";
                command = "cargo build --release \"$@\"";
              }
              {
                category = "check";
                name = "check";
                help = "cargo check";
                command = "cargo check \"$@\"";
              }
              {
                category = "check";
                name = "clippy";
                help = "cargo clippy -- -D warnings (matches CI)";
                command = "cargo clippy \"$@\" -- -D warnings";
              }
              {
                category = "check";
                name = "fmt";
                help = "cargo fmt";
                command = "cargo fmt \"$@\"";
              }
              {
                category = "check";
                name = "fmt-check";
                help = "cargo fmt --all -- --check (matches CI)";
                command = "cargo fmt --all -- --check \"$@\"";
              }
              {
                category = "check";
                name = "run-tests";
                help = "cargo test (matches CI)";
                command = "cargo test \"$@\"";
              }
              {
                category = "check";
                name = "ci-local";
                help = "run the Rust-side CI sequence";
                command = ''
                  set -euo pipefail
                  cargo fmt --all -- --check
                  cargo check
                  cargo clippy -- -D warnings
                  cargo test
                  cargo build --release
                '';
              }
              {
                category = "check";
                name = "coverage";
                help = "test coverage summary (pass --html for a browsable report)";
                command = ''
                  set -euo pipefail
                  LLVM_COV="$(find /nix/store -maxdepth 3 -name llvm-cov 2>/dev/null | head -1)"
                  LLVM_PROFDATA="$(find /nix/store -maxdepth 3 -name llvm-profdata 2>/dev/null | head -1)"
                  export LLVM_COV LLVM_PROFDATA
                  if [ "''${1:-}" = "--html" ]; then
                    cargo llvm-cov --workspace --html --output-dir target/coverage
                    echo "Report: target/coverage/html/index.html"
                  else
                    cargo llvm-cov --workspace --summary-only
                  fi
                '';
              }
              {
                category = "run";
                name = "koban";
                help = "run koban";
                command = "cargo run -- \"$@\"";
              }
              {
                category = "run";
                name = "koban-help";
                help = "show koban help";
                command = "cargo run -- --help";
              }
              {
                category = "run";
                name = "smoke-statics";
                help = "safe live GET /api/v1/statics smoke test";
                command = "cargo run -- statics \"$@\"";
              }
            ];
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs.nixfmt.enable = true;
            programs.rustfmt = {
              enable = true;
              edition = "2024";
            };
          };
        };
    };
}
