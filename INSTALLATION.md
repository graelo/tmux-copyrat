# Installation

## Prerequisites

- [tmux](https://tmux.github.io) 3.0+
- `curl` or `wget` (for automatic binary download)

## TPM (recommended)

Add to `~/.tmux.conf`:

```tmux
set -g @plugin 'graelo/tmux-copyrat'
```

Then press <kbd>prefix</kbd> + <kbd>I</kbd> to install. TPM clones the repo into
`~/.tmux/plugins/tmux-copyrat/`. On the next tmux startup (or reload), the
plugin script runs `install-binary.sh` to download the pre-built binary from
[GitHub Releases](https://github.com/graelo/tmux-copyrat/releases).

To update, press <kbd>prefix</kbd> + <kbd>U</kbd>. If a new version was
released, the plugin script automatically downloads the new binary.

> **Note**: never modify files in `~/.tmux/plugins/tmux-copyrat/` directly —
> changes are lost on update. Customize via `@copyrat-*` options in
> `~/.tmux.conf` (see [CONFIGURATION.md](CONFIGURATION.md)).

## Manual

Same as TPM, but you maintain the cloned repo yourself. Requires `git`.

```bash
git clone https://github.com/graelo/tmux-copyrat ~/.tmux/plugins/tmux-copyrat
```

Add to `~/.tmux.conf`:

```tmux
run-shell ~/.tmux/plugins/tmux-copyrat/tmux-copyrat.tmux
```

Reload with `tmux source-file ~/.tmux.conf`. The plugin automatically downloads
the binary when it is missing or outdated.

## Homebrew

```bash
brew install graelo/tap/tmux-copyrat
```

With Homebrew the binary is managed by `brew upgrade`. You still need the plugin
script for tmux integration — either via TPM or manual clone (see above), or
even just the script file alone. The plugin script won't attempt to download the
binary because it sees it's present in the `PATH`.

## Cargo (build from source)

Requires `rustc 1.85+`.

```bash
cargo install copyrat
```

This installs both `tmux-copyrat` and `copyrat` binaries. As with Homebrew, you
still need the plugin script for tmux integration.

## Verification

```bash
~/.tmux/plugins/tmux-copyrat/tmux-copyrat --version
```

Try <kbd>prefix</kbd> + <kbd>t</kbd> + <kbd>u</kbd> to highlight URLs. If hints
appear and copying works, the installation is complete.

See [README.md](README.md) for default keys and
[CONFIGURATION.md](CONFIGURATION.md) for customization.
