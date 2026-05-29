# Installation

## curl | sh (macOS & Linux)

The supported installer downloads GitHub release tarballs, verifies `SHA256SUMS`
when available, installs the `koban` binary into `~/.local/bin` by default, and
prints the installed version:

```sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | sh
```

It uses the same macOS/Linux asset names as release CI and `koban update`.

### Installer options

Pin a version or change the install directory with environment variables:

```sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | \
  KOBAN_VERSION="v0.1.0" KOBAN_INSTALL_DIR="$HOME/.local/bin" sh
```

| Variable            | Default        | Purpose                                            |
| ------------------- | -------------- | -------------------------------------------------- |
| `KOBAN_VERSION`     | `latest`       | Release tag to install (e.g. `v0.1.0`, `nightly`). |
| `KOBAN_INSTALL_DIR` | `~/.local/bin` | Directory to install the binary into.              |

```sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | KOBAN_VERSION=nightly sh
curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | KOBAN_INSTALL_DIR=/usr/local/bin sh
```

If `~/.local/bin` is not on your `PATH`, the installer prints the line to add to
your shell profile.

## Cargo

Install the CLI from crates.io (produces a `koban` binary):

```sh
cargo install koban-cli
```

## Nix

Run straight from the flake without installing:

```sh
nix run github:jamesbrink/koban -- --help
```

The flake exports `packages.default`, `packages.koban`, `apps.default`,
`apps.koban`, `checks.koban`, and a development shell for Linux and Darwin on
both x86_64 and aarch64.

## Verify

```sh
koban --version
koban --help
```

Once installed, head to the [Quickstart](/guide/quickstart).
