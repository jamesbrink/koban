# Shell completions

koban generates shell completion scripts straight from the binary with the
`completions` command:

```sh
koban completions bash
koban completions zsh
koban completions fish
koban completions nushell
koban completions elvish
koban completions powershell
```

Supported shells: **bash**, **zsh**, **fish**, **nushell**, **elvish**, and
**powershell**.

## Installing completions

Each command prints the completion script to stdout. Redirect it to the location
your shell expects. For example:

::: code-group

```sh [bash]
koban completions bash | sudo tee /etc/bash_completion.d/koban > /dev/null
```

```sh [zsh]
koban completions zsh > "${fpath[1]}/_koban"
```

```sh [fish]
koban completions fish > ~/.config/fish/completions/koban.fish
```

:::

Reload your shell (or `source` your profile) afterward to pick up the new
completions.
