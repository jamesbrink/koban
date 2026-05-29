# Repository Instructions

Always use conventional commits and proper branch names like `fix/*`, `feat/*`,
and `chore/*`.

Use TDD for behavior changes: add or update the narrow failing test first, then
implement the smallest change that makes it pass. Keep tests close to the
behavior they protect, prefer mocked Invoice Ninja API tests for command
behavior, and add regression tests for every bug fix.

Protect coverage and code shape proactively. New code must keep the Codecov
project and patch targets green, and local changes should run the narrowest
relevant coverage check before review. Avoid god files: keep Rust modules small,
focused, and easy to review. If a file starts accumulating unrelated command,
API, rendering, and validation logic, split it before adding more features.
Run `code-health` or `scripts/check-code-health.sh` when moving Rust modules.

The `main` branch is protected. Use pull requests for changes and keep the
required CI contexts green: `fmt`, `check`, `clippy`, `test`, and `build`.
Resolve review conversations before merging.

Koban is a Cargo workspace with two crates: `crates/koban` is the publishable
Invoice Ninja API client library (`ApiClient`, `Config`, `Resource`, typed
`models`, errors), and `crates/koban-cli` is the CLI. The CLI package is
`koban-cli` but produces a binary named `koban` (via `[[bin]] name = "koban"`),
so release assets, `install.sh`, and `koban update` keep using `koban`. The
library keeps `miette` behind an optional feature (the CLI enables it); CLI-only
input parsing (clap structs, payload/invoice/render/update) stays in `koban-cli`.
Run `cargo run -p koban-cli -- ...` from the workspace root.

The implemented API surface spans the official resource families with guarded
write commands:

- Prefer the public Invoice Ninja demo endpoint for live smoke tests:
  `INVOICE_NINJA_BASE_URL=https://demo.invoiceninja.com` and
  `INVOICE_NINJA_API_TOKEN=TOKEN`.
- Write commands must keep `--dry-run` previews and `--yes` confirmation gates
  for destructive or externally visible mutations.
- Do not live-smoke high-risk endpoints such as imports, purges, refunds,
  merges, scheduler, support, or admin utility routes unless the smoke helper is
  demo-only, opt-in, and creates/cleans up its own fixtures.
- Use production or personal accounts only for intentional checks.
- Keep token handling environment-first with `INVOICE_NINJA_API_TOKEN` and
  optional `INVOICE_NINJA_BASE_URL`. `koban auth login` may also persist a token,
  but env vars always win. Credential resolution lives in `koban-cli`'s
  `config_store` (env → OS keychain → `0600` config file at the platform config
  dir, overridable with `KOBAN_CONFIG_DIR`); the publishable library stays free
  of disk/keychain I/O. Keychain support is the default-on `keychain` cargo
  feature in `koban-cli`.
- `koban skill generate`/`install` emit the agent skill. Keep one shared body in
  `skill/templates.rs` with target-correct frontmatter per kind (Claude Code,
  Codex, pi, Cursor `.mdc`, plugin JSON, and a marker-wrapped `AGENTS.md` block).
  `--target all` = `claude-code` + `codex` + `agents-md`.
- Redact tokens in errors, traces, fixtures, and docs.
- Preserve stable JSON output for agents alongside useful table output for
  humans.
- Prefer mocked API tests for new command behavior.

The Nix devshell intentionally exposes the project helper menu. Keep these
helpers in sync with README.md and CI when editing `flake.nix`: `build`,
`build-release`, `check`, `clippy`, `fmt`, `fmt-check`, `run-tests`, `ci-local`,
`coverage`, `code-health`, `koban`, `koban-help`, `smoke-statics`,
`smoke-invoice-write-demo`, `smoke-all-demo`, `docs-dev`, `docs-build`,
`docs-preview`, `docs-fmt`, and `docs-fmt-check`.
The devshell loads `INVOICE_NINJA_API_TOKEN` and `INVOICE_NINJA_BASE_URL` from
the gitignored `.env` file when those variables are not already set.

The documentation website lives in `website/` (VitePress + Tailwind v4 + Vue 3,
built with `bun`) and deploys to GitHub Pages via `.github/workflows/pages.yml`.
Keep the `docs-*` devshell helpers, README, and `website/package.json` scripts in
sync. Docs-only changes are excluded from the nightly build via `paths-ignore`.
Mutating smoke helpers must hard-code the public demo API internally so they
cannot inherit a production or personal endpoint.

Release automation lives in `.github/workflows/release-please.yml`. release-please
runs in workspace mode (`cargo-workspace` + `linked-versions` plugins) keeping
`koban` and `koban-cli` on one linked version; the `koban-cli` component owns the
prefix-free `vX.Y.Z` tag that carries the binary assets, while the library uses a
`koban-v*` tag. Do not add code signing or notarization unless explicitly
requested. Release assets must stay in sync with `koban update` asset names and
`install.sh`, and each release must publish `SHA256SUMS`. crates.io publishing
must remain gated on `CARGO_REGISTRY_TOKEN`, publishing the `koban` library
before `koban-cli`. Repository Actions workflow permissions must allow read/write
tokens and GitHub Actions PR creation so release-please can open release PRs.

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
