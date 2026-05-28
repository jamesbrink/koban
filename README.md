# koban

`koban` is an early-stage Rust CLI for [Invoice Ninja](https://www.invoiceninja.com/).
The goal is a small, scriptable tool that feels good for humans at a terminal
and predictable for AI agents that need stable JSON output, explicit errors, and
shell completions.

The crate name is claimed on crates.io as `koban` at `0.0.1`. This repository is
still foundation work: the CLI boots, reports its version, generates shell
completions, and has CI/Nix coverage, but it does not talk to Invoice Ninja yet.

## Current CLI

```sh
koban --help
koban --version
koban completions zsh
koban completions bash
koban completions fish
```

The global `--output` flag is already reserved for future commands:

```sh
koban --output json ...
koban --output table ...
```

## Invoice Ninja Direction

Invoice Ninja v5 exposes an API under `/api/v1`. Hosted production is
`https://invoicing.co`, and self-hosted installs use the same namespace under
their own base URL.

Authentication is token based. Requests require `X-API-TOKEN`, and the developer
guide also documents `X-Requested-With: XMLHttpRequest` as a required security
header. JSON write requests must send `Content-Type: application/json`.

The first useful `koban` API surface should stay boring and durable:

```text
koban auth check
koban clients list
koban clients get <id>
koban invoices list
koban invoices get <id>
koban payments list
```

After that, creation and update commands can grow around explicit files or
stdin-first JSON so agent workflows do not depend on prompts.

## Configuration Plan

The intended configuration model is environment-first:

```sh
export KOBAN_BASE_URL="https://invoicing.co"
export KOBAN_API_TOKEN="..."
```

Future config files should only add convenience. Tokens must never be printed by
default, and human-facing output should have a matching JSON mode before it
ships.

## Development

This repo pins Rust in `rust-toolchain.toml` and exposes the same toolchain
through the Nix flake.

```sh
cargo fmt --all -- --check
cargo check
cargo clippy -- -D warnings
cargo test
cargo build --release
nix flake check
```

With Nix:

```sh
nix develop
nix build
nix run . -- --help
```

The flake exports `packages.default`, `packages.koban`, `apps.default`,
`apps.koban`, `checks.koban`, and a development shell for Linux and Darwin on
both x86_64 and aarch64.

## API Notes

The working notes in [docs/invoice-ninja-api.md](docs/invoice-ninja-api.md) are
grounded in the current Invoice Ninja documentation and should be refreshed
before adding the first networked commands.
