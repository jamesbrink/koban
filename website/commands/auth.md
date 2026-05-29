# Authentication

`koban auth` stores your Invoice Ninja API token so you don't have to export it
in every shell. Environment variables always take precedence, so agents and CI
stay deterministic.

## Log in

```sh
koban auth login --token <token>            # verifies, then stores in the config file
koban auth login --keychain                 # store in the OS keychain instead
koban auth login --base-url https://my.host # self-hosted Invoice Ninja
echo "$TOKEN" | koban auth login --no-verify # pipe a token (agent/CI friendly)
```

By default `login` makes one safe, read-only call (`GET /api/v1/statics`) to
confirm the token works before saving. Pass `--no-verify` to skip the check
(offline or CI). If no token is supplied on a TTY, koban prompts for it without
echoing.

## Status & logout

```sh
koban auth status                # shows the active credential source, never the token
koban auth status --output json
koban auth logout                # removes the stored token (config file + keychain)
```

## Resolution order

A token is resolved in this order:

1. `INVOICE_NINJA_API_TOKEN` (and `INVOICE_NINJA_BASE_URL`) environment variables
2. the OS keychain (when you logged in with `--keychain`)
3. the stored config file

## Storage

The config file is written with owner-only (`0600`) permissions at the platform
config directory (for example `~/.config/koban/config.json` on Linux,
`~/Library/Application Support/koban/config.json` on macOS). Set
`KOBAN_CONFIG_DIR` to override the location. With `--keychain`, the token lives
in your OS keychain and the config file only records that fact (plus the base
URL). Tokens are redacted from output, errors, and traces.

## Build without keychain support

Keychain storage is enabled by default. Minimal or headless builds can drop the
backend with `cargo build -p koban-cli --no-default-features`; `--keychain` then
returns a clear error and file storage continues to work.
