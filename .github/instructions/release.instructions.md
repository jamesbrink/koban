---
applyTo: "install.sh,README.md,AGENTS.md,CLAUDE.md,.github/workflows/*.yml,release-please-config.json,.release-please-manifest.json,flake.nix"
---

Keep release assets aligned across release CI, `install.sh`, and `src/update.rs`:

- `koban-aarch64-apple-darwin.tar.gz`
- `koban-x86_64-apple-darwin.tar.gz`
- `koban-x86_64-unknown-linux-gnu.tar.gz`
- `koban-aarch64-unknown-linux-gnu.tar.gz`
- `SHA256SUMS`

Koban is not code-signed or notarized. Do not add signing unless explicitly requested.

The nightly workflow builds current `main` into a rolling `nightly` prerelease
through `nightly-staging`. Keep `koban update --nightly` working with the same
asset names and `SHA256SUMS`.

For installer changes, run `sh -n install.sh` and inspect README examples for
copy/paste correctness. For Nix or release workflow changes, run
`nix flake check` when practical.
