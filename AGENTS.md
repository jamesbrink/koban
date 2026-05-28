# Repository Instructions

Always use conventional commits and proper branch names like `fix/*`, `feat/*`,
and `chore/*`.

Koban is an early Rust CLI for Invoice Ninja. Keep the current implemented API
surface read-only unless James explicitly asks for write support:

- Use only `GET` requests in CLI commands.
- Do not smoke test destructive, write, bulk, upload, import, email, purge,
  refund, merge, archive, or delete endpoints against an active account.
- Keep token handling environment-first with `INVOICE_NINJA_API_TOKEN` and
  optional `INVOICE_NINJA_BASE_URL`.
- Redact tokens in errors, traces, fixtures, and docs.
- Preserve stable JSON output for agents alongside useful table output for
  humans.
- Prefer mocked API tests for new command behavior.

The Nix devshell intentionally exposes the project helper menu. Keep these
helpers in sync with README.md and CI when editing `flake.nix`: `build`,
`build-release`, `check`, `clippy`, `fmt`, `fmt-check`, `run-tests`, `ci-local`,
`coverage`, `koban`, `koban-help`, and `smoke-statics`.

Release automation lives in `.github/workflows/release-please.yml`. Koban is a
plain CLI: do not add code signing or notarization unless James explicitly asks.
Release assets must stay in sync with `koban update` asset names and
`SHA256SUMS`, and crates.io publishing must remain gated on
`CARGO_REGISTRY_TOKEN`.
