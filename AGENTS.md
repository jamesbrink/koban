# Repository Instructions

Always use conventional commits and proper branch names like `fix/*`, `feat/*`,
and `chore/*`.

The `main` branch is protected. Use pull requests for changes and keep the
required CI contexts green: `fmt`, `check`, `clippy`, `test`, and `build`.
Resolve review conversations before merging.

Koban is an early Rust CLI for Invoice Ninja. The implemented API surface is
read-first, with guarded invoice write commands:

- Prefer the public Invoice Ninja demo endpoint for live smoke tests:
  `INVOICE_NINJA_BASE_URL=https://demo.invoiceninja.com` and
  `INVOICE_NINJA_API_TOKEN=TOKEN`.
- Invoice write commands must keep `--dry-run` previews and `--yes`
  confirmation gates for destructive or externally visible mutations.
- Do not smoke test unimplemented write families such as client/payment writes,
  imports, purges, refunds, merges, or scheduler endpoints against any
  environment unless that support has been explicitly implemented and reviewed.
- Use production or personal accounts only for intentional checks.
- Keep token handling environment-first with `INVOICE_NINJA_API_TOKEN` and
  optional `INVOICE_NINJA_BASE_URL`.
- Redact tokens in errors, traces, fixtures, and docs.
- Preserve stable JSON output for agents alongside useful table output for
  humans.
- Prefer mocked API tests for new command behavior.

The Nix devshell intentionally exposes the project helper menu. Keep these
helpers in sync with README.md and CI when editing `flake.nix`: `build`,
`build-release`, `check`, `clippy`, `fmt`, `fmt-check`, `run-tests`, `ci-local`,
`coverage`, `koban`, `koban-help`, `smoke-statics`, and
`smoke-invoice-write-demo`, and `smoke-all-demo`.
The devshell loads `INVOICE_NINJA_API_TOKEN` and `INVOICE_NINJA_BASE_URL` from
the gitignored `.env` file when those variables are not already set.

Release automation lives in `.github/workflows/release-please.yml`. Koban is a
plain CLI: do not add code signing or notarization unless explicitly requested.
Release assets must stay in sync with `koban update` asset names and
`install.sh`, and each release must publish `SHA256SUMS`. crates.io publishing
must remain gated on `CARGO_REGISTRY_TOKEN`. Repository Actions workflow
permissions must allow read/write tokens and GitHub Actions PR creation so
release-please can open release PRs.

Nightly automation lives in `.github/workflows/nightly.yml`. It builds current
`main` into a rolling `nightly` prerelease through `nightly-staging`, then
promotes the staged assets only after all target builds and checksums succeed.
Keep `koban update --nightly`, nightly release assets, and `SHA256SUMS` in sync.

`install.sh` is the supported `curl | sh` installer. Keep its asset matrix,
checksum handling, and environment variables (`KOBAN_INSTALL_DIR`,
`KOBAN_VERSION`) in sync with README.md and release assets.

AGENTS.md is the single source of truth for agent instructions. `CLAUDE.md`
must remain a symlink to `AGENTS.md`, not a separate copy.

Copilot repository instructions live in `.github/copilot-instructions.md`, with
path-specific frontmatter instructions in `.github/instructions/*.instructions.md`.
Keep them short and consistent with this file.
