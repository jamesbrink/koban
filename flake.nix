{
  description = "koban - Invoice Ninja from the terminal";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
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

          devShells.default = pkgs.mkShell {
            packages = [
              rustToolchain
              pkgs.rust-analyzer
              pkgs.cargo-llvm-cov
              pkgs.git
              pkgs.libiconv
              pkgs.jq
            ];

            env = {
              RUST_BACKTRACE = "1";
            };
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
