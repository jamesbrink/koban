---
applyTo: "**/*.rs,Cargo.toml,Cargo.lock,rust-toolchain.toml"
---

Keep Rust changes idiomatic and small. Preserve clap's helpful usage messages,
stable JSON output for agents, and table output for humans.

Use mocked HTTP tests for Invoice Ninja command behavior. Do not hit live
destructive endpoints. Commands that talk to Invoice Ninja should keep using
`GET` unless the task explicitly adds a reviewed write path.

Run `cargo fmt --all -- --check`, `cargo clippy -- -D warnings`, and
`cargo test` for general Rust changes.
