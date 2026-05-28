---
applyTo: "**/*.rs,Cargo.toml,Cargo.lock,rust-toolchain.toml"
---

Keep Rust changes idiomatic and small. Preserve clap's helpful usage messages,
stable JSON output for agents, and table output for humans.

Use mocked HTTP tests for Invoice Ninja command behavior. Live smoke tests
should prefer the public demo API at `https://demo.invoiceninja.com` with token
`TOKEN`. Implemented invoice write commands must keep `--dry-run` previews and
`--yes` confirmation gates where mutations are destructive or externally
visible. Do not hit unimplemented live destructive endpoints.

Run `cargo fmt --all -- --check`, `cargo clippy -- -D warnings`, and
`cargo test` for general Rust changes.
