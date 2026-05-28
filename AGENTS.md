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
