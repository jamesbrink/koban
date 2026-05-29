# Updating

`koban update` keeps direct binary installs current by downloading GitHub
release tarballs and verifying their checksums.

```sh
koban update --check          # report whether a newer release exists
koban update                  # download and install the latest release
koban update --tag v0.1.0     # install a specific tagged release
koban update --nightly --check
koban update --nightly        # install the latest rolling nightly
```

| Flag        | Purpose                                          |
| ----------- | ------------------------------------------------ |
| `--check`   | Report available updates without installing.     |
| `--tag <t>` | Install a specific release tag.                  |
| `--nightly` | Target the rolling `nightly` prerelease channel. |
| `--force`   | Reinstall even if already up to date.            |

## Package-manager installs

If koban was installed via a package manager (rather than the `curl | sh`
installer or a release tarball), `koban update` leaves the installed binary
alone and prints an upgrade recipe for that package manager instead.

## Nightly channel

The nightly workflow builds the current `main` branch into a rolling `nightly`
prerelease. It stages assets in a `nightly-staging` release while compiling, so
the previous nightly stays available to updater clients until the new assets are
ready. `koban update --nightly` tracks that channel.
