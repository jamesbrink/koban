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
          # Cargo sources plus the workflow files and READMEs the test suite and
          # package metadata reference (code_health_tests reads .github/workflows).
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              (lib.fileset.fileFilter (
                file:
                file.hasExt "rs"
                || file.name == "Cargo.toml"
                || file.name == "Cargo.lock"
                || file.name == "README.md"
              ) ./.)
              ./.github/workflows
            ];
          };

          # The root manifest is a virtual workspace; crate metadata lives in the
          # member manifests, and shared fields live under [workspace.package].
          workspaceToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          cliToml = builtins.fromTOML (builtins.readFile ./crates/koban-cli/Cargo.toml);

          meta = {
            description = cliToml.package.description;
            homepage = workspaceToml.workspace.package.homepage;
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
            pname = "koban";
            version = cliToml.package.version;
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
              # Build the CLI package; it pulls in the koban library as a
              # workspace dependency and produces the `koban` binary.
              cargoExtraArgs = "--package koban-cli";
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

            devshell.startup.load-env = {
              text = ''
                if [ -f .env ]; then
                  koban_load_dotenv_var() {
                    local key="$1"
                    local value
                    value="$(
                      awk -F= -v key="$key" '
                        /^[[:space:]]*(#|$)/ { next }
                        {
                          raw_key = $1
                          sub(/^[[:space:]]*/, "", raw_key)
                          sub(/[[:space:]]*$/, "", raw_key)
                          if (raw_key != key) { next }

                          value = substr($0, index($0, "=") + 1)
                          sub(/^[[:space:]]*/, "", value)
                          sub(/[[:space:]]*$/, "", value)
                          sub(/\r$/, "", value)
                          if (substr(value, 1, 1) == "\"" && substr(value, length(value), 1) == "\"") {
                            value = substr(value, 2, length(value) - 2)
                          }
                          print value
                          exit
                        }
                      ' .env
                    )"
                    if [ -n "$value" ] && [ -z "''${!key:-}" ]; then
                      export "$key=$value"
                    fi
                  }

                  koban_load_dotenv_var INVOICE_NINJA_API_TOKEN
                  koban_load_dotenv_var INVOICE_NINJA_BASE_URL
                  unset -f koban_load_dotenv_var
                fi
              '';
            };

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
                  scripts/check-code-health.sh
                  cargo clippy -- -D warnings
                  cargo test
                  cargo build --release
                '';
              }
              {
                category = "check";
                name = "code-health";
                help = "check Rust source files against module size budgets";
                command = "scripts/check-code-health.sh";
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
                command = "cargo run -p koban-cli -- \"$@\"";
              }
              {
                category = "run";
                name = "koban-help";
                help = "show koban help";
                command = "cargo run -p koban-cli -- --help";
              }
              {
                category = "run";
                name = "smoke-statics";
                help = "safe live GET /api/v1/statics smoke test";
                command = "cargo run -p koban-cli -- statics \"$@\"";
              }
              {
                category = "run";
                name = "smoke-invoice-write-demo";
                help = "explicit demo-only invoice create/update/delete smoke test";
                command = ''
                  set -euo pipefail
                  if [ "''${KOBAN_LIVE_WRITE_SMOKE:-}" != "1" ]; then
                    echo "Set KOBAN_LIVE_WRITE_SMOKE=1 to run this mutating demo smoke test." >&2
                    exit 2
                  fi
                  readonly INVOICE_NINJA_BASE_URL="https://demo.invoiceninja.com"
                  readonly INVOICE_NINJA_API_TOKEN="TOKEN"
                  export INVOICE_NINJA_BASE_URL INVOICE_NINJA_API_TOKEN
                  echo "Using Invoice Ninja public demo API: $INVOICE_NINJA_BASE_URL"

                  client_id="$(
                    cargo run -p koban-cli -- --output json clients list --per-page 1 \
                      | jq -r '.data[0].id // empty'
                  )"
                  if [ -z "$client_id" ]; then
                    echo "No demo client was available for invoice write smoke testing." >&2
                    exit 1
                  fi

                  invoice_id="$(
                    cargo run -p koban-cli -- --output json invoices create \
                      --client-id "$client_id" \
                      --line-item product_key=KobanSmoke,quantity=1,cost=1 \
                      --private-notes "Koban demo write smoke" \
                      | jq -r '.data.id // .id // empty'
                  )"
                  if [ -z "$invoice_id" ]; then
                    echo "Invoice creation did not return an id." >&2
                    exit 1
                  fi

                  cargo run -p koban-cli -- --output json invoices update "$invoice_id" \
                    --private-notes "Koban demo write smoke updated" >/dev/null
                  cargo run -p koban-cli -- --output json invoices delete "$invoice_id" --yes >/dev/null
                  echo "Created, updated, and deleted demo invoice $invoice_id"
                '';
              }
              {
                category = "run";
                name = "smoke-all-demo";
                help = "explicit demo-only smoke test for every implemented command family";
                command = "./scripts/smoke-all-demo.sh \"$@\"";
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
