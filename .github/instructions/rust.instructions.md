---
applyTo: "**/*.rs,Cargo.toml,Cargo.lock,rust-toolchain.toml"
---

Keep Rust changes idiomatic and small. Preserve clap's helpful usage messages,
stable JSON output for agents, and table output for humans.

Use TDD for behavior changes and regressions. Add or update tests first, then
make the smallest implementation change. Keep modules focused; run
`scripts/check-code-health.sh` after refactors and split growing files before
they mix CLI parsing, HTTP, rendering, validation, and command execution.

Use mocked HTTP tests for Invoice Ninja command behavior. Live smoke tests
should prefer the public demo API at `https://demo.invoiceninja.com` with token
`TOKEN`. Implemented write commands must keep `--dry-run` previews and `--yes`
confirmation gates where mutations are destructive or externally visible. Do
not live-smoke high-risk destructive endpoints unless the helper is demo-only,
opt-in, and cleans up its own fixtures.

Run `cargo fmt --all -- --check`, `cargo check`,
`scripts/check-code-health.sh`, `cargo clippy -- -D warnings`, and `cargo test`
for general Rust changes.
