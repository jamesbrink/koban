# koban

`koban` is an early-stage Rust CLI for [Invoice Ninja](https://www.invoiceninja.com/).
The goal is a small, scriptable tool that feels good for humans at a terminal
and predictable for AI agents that need stable JSON output, explicit errors, and
shell completions.

The crate name is claimed on crates.io as `koban` at `0.0.1`. This repository is
still early work: the CLI boots, reports its version, generates shell
completions, and exposes a small read-only Invoice Ninja API surface.

## Install

For direct binary installs on macOS and Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | sh
```

Installer options:

```sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | \
  KOBAN_VERSION="v0.1.0" KOBAN_INSTALL_DIR="$HOME/.local/bin" sh
```

The installer downloads GitHub release tarballs, verifies `SHA256SUMS` when
available, installs `koban` into `~/.local/bin` by default, and prints the
installed version. It uses the same macOS/Linux asset names as release CI and
`koban update`.

Other install paths:

```sh
cargo install koban
nix run github:jamesbrink/koban -- --help
```

## Current CLI

```sh
koban --help
koban --version
koban update --check
koban completions zsh
koban completions bash
koban completions fish
koban completions nushell
```

The first API commands are read-only and use `GET` requests only:

```sh
koban statics --output json
koban clients list --page 1 --per-page 20
koban clients show <id> --output json
koban clients template --output json
koban clients edit-template <id> --output json
koban invoices list
koban invoices show <id>
koban invoices template --output json
koban invoices edit-template <id> --output json
koban payments list
koban payments show <id>
koban payments template --output json
koban payments edit-template <id> --output json
```

## Invoice Ninja Direction

Invoice Ninja v5 exposes an API under `/api/v1`. Hosted production is
`https://invoicing.co`, and self-hosted installs use the same namespace under
their own base URL.

Authentication is token based. Requests require `X-API-TOKEN`, and the developer
guide also documents `X-Requested-With: XMLHttpRequest` as a required security
header. JSON write requests must send `Content-Type: application/json`.

The implemented `koban` API surface is intentionally boring and durable:

```text
koban statics
koban clients list
koban clients show <id>
koban clients template
koban clients edit-template <id>
koban invoices list
koban invoices show <id>
koban invoices template
koban invoices edit-template <id>
koban payments list
koban payments show <id>
koban payments template
koban payments edit-template <id>
```

The `template` and `edit-template` commands use Invoice Ninja's read-only
`GET /create` and `GET /{id}/edit` routes. They return default/editable payloads
for schema discovery; they do not create or update records.

Future creation and update commands should grow around explicit files or
stdin-first JSON so agent workflows do not depend on prompts.

## Configuration Plan

The intended configuration model is environment-first:

```sh
export INVOICE_NINJA_BASE_URL="https://invoicing.co"
export INVOICE_NINJA_API_TOKEN="..."
```

`INVOICE_NINJA_BASE_URL` is optional and defaults to `https://invoicing.co`.
Tokens must never be printed by default, and human-facing output should have a
matching JSON mode before it ships.

## Safety

Current commands issue only `GET` requests. Do not smoke test write, bulk,
upload, import, email, purge, refund, merge, archive, or delete endpoints against
an active account.

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

Inside `nix develop`, the devshell menu exposes helper commands:

```text
build           cargo build (debug)
build-release   cargo build --release
check           cargo check
clippy          cargo clippy -- -D warnings
fmt             cargo fmt
fmt-check       cargo fmt --all -- --check
run-tests       cargo test
ci-local        run the Rust-side CI sequence
coverage        cargo llvm-cov summary, or --html for a report
koban           cargo run -- ...
koban-help      show koban help
smoke-statics   safe live GET /api/v1/statics smoke test
```

The flake exports `packages.default`, `packages.koban`, `apps.default`,
`apps.koban`, `checks.koban`, and a development shell for Linux and Darwin on
both x86_64 and aarch64.

## Releases

Releases are managed by release-please. When a release is cut, CI builds
unsigned CLI tarballs for macOS and Linux, uploads `SHA256SUMS`, and publishes
the crate to crates.io using `CARGO_REGISTRY_TOKEN`.

The `install.sh` script is the supported `curl | sh` path for direct installs:

```sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | KOBAN_VERSION=v0.1.0 sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | KOBAN_INSTALL_DIR=/usr/local/bin sh
```

`koban update` downloads those GitHub release tarballs and verifies checksums
for direct installs. Package-manager installs are left alone and get an upgrade
recipe instead:

```sh
koban update --check
koban update
koban update --tag v0.1.0
```

## API Notes

The working notes in [docs/invoice-ninja-api.md](docs/invoice-ninja-api.md) are
grounded in the current Invoice Ninja documentation and should be refreshed
before adding new networked command groups.
