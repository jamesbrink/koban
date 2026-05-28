---
applyTo: "**/*.md"
---

Keep docs current with the actual CLI, release workflow, and devshell helpers.
`AGENTS.md` is the source of truth for agent instructions; `CLAUDE.md` must stay
a symlink to `AGENTS.md`.

When documenting Invoice Ninja behavior, preserve the read-only safety posture
unless write support is explicitly requested. Document the public demo API as
the preferred target for read-only live smoke tests:
`https://demo.invoiceninja.com` with token `TOKEN`.
