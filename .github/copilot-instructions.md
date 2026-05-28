# Koban Copilot Instructions

Koban is an early-stage Rust CLI for Invoice Ninja. It is built for humans at a
terminal and AI agents that need stable JSON, explicit errors, and predictable
shell completion.

Treat `AGENTS.md` as the primary source of repository instructions. `CLAUDE.md`
is intentionally a symlink to `AGENTS.md`; do not replace it with a copied file.

## Architecture

- `src/main.rs` is the thin binary entry point.
- `src/lib.rs` owns CLI parsing, config, Invoice Ninja HTTP calls, output
  shaping, and most unit tests.
- `src/update.rs` owns direct release-tarball self-update behavior.
- `tests/cli_tests.rs` and `tests/completions_tests.rs` cover user-facing CLI
  behavior.
- `docs/invoice-ninja-api.md` records API research and safe starting points.
- `flake.nix` defines the pure package/app/checks and the devshell helper menu.

## Safety

The implemented Invoice Ninja API surface is read-first, with guarded invoice
write commands. Invoice download commands may save PDF bytes to explicit local
paths, but must still use `GET`.

Prefer the public demo API for live smoke tests:
`INVOICE_NINJA_BASE_URL=https://demo.invoiceninja.com` and
`INVOICE_NINJA_API_TOKEN=TOKEN`. Invoice write commands must keep `--dry-run`
previews and `--yes` confirmation gates for destructive or externally visible
mutations. Do not add or smoke test unimplemented write families in any
environment unless that support has been explicitly implemented and reviewed.
Prefer mocked API tests for command behavior.

Use `INVOICE_NINJA_API_TOKEN` and optional `INVOICE_NINJA_BASE_URL` for config.
Redact tokens from errors, traces, fixtures, and docs.

## Validation

Before proposing a change, run the narrowest relevant check. For general Rust
changes, prefer:

```sh
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
```

For release, Nix, or installer work, also consider:

```sh
nix flake check
sh -n install.sh
```

Inside `nix develop`, helper commands include `fmt-check`, `clippy`,
`run-tests`, `ci-local`, `coverage`, `koban`, `koban-help`, and
`smoke-statics`. `smoke-invoice-write-demo` is demo-only and requires
`KOBAN_LIVE_WRITE_SMOKE=1`.

## Releases

Release automation lives in `.github/workflows/release-please.yml`. Koban is a
plain CLI, so do not add code signing or notarization unless explicitly requested. Release tarball asset names, `SHA256SUMS`, `koban update`, and
`install.sh` must stay in sync.

Nightly automation lives in `.github/workflows/nightly.yml`. It builds current
`main` into a rolling `nightly` prerelease via `nightly-staging`. Keep
`koban update --nightly` and nightly assets aligned with stable release assets.
