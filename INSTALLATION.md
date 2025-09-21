# Installation

This guide provides multiple ways to install `tmux-copyrat`. Choose the method that best suits your setup.

## Prerequisites

- [tmux](https://tmux.github.io) 3.0+
- `curl` or `wget` (for automatic binary download)

## Method 1: Tmux Plugin Manager (TPM) - Recommended

If you use [TPM](https://github.com/tmux-plugins/tpm), add this line to your `~/.tmux.conf`:

```tmux
set -g @plugin 'graelo/tmux-copyrat'
```

Then reload your tmux configuration:

1. Press <kbd>prefix</kbd> + <kbd>I</kbd> to install the plugin
2. The `tmux-copyrat.tmux` script will automatically detect your system and download the appropriate binary from GitHub releases

## Method 2: Manual Installation

### Clone the Repository

```bash
# Clone the repository
git clone https://github.com/graelo/tmux-copyrat ~/.tmux/plugins/tmux-copyrat
```

### Add to tmux Configuration

Add this line to your `~/.tmux.conf`:

```tmux
run-shell ~/.tmux/plugins/tmux-copyrat/tmux-copyrat.tmux
```

### Reload Configuration

```bash
tmux source-file ~/.tmux.conf
```

The `tmux-copyrat.tmux` script will automatically detect your system (macOS/Linux) and architecture, then download the appropriate binary from GitHub releases on first run.

## Method 3: Custom Key Bindings

For advanced users who prefer to manage their own key bindings, you can:

1. Download the binary using Method 2 (or manually download from [GitHub releases](https://github.com/graelo/tmux-copyrat/releases))
2. Skip sourcing the `tmux-copyrat.tmux` file
3. Create custom key bindings in your `~/.tmux.conf`

**Important**: When creating custom bindings, avoid using `run-shell` as tmux launches these processes without attaching them to a pty. Refer to the [`tmux-copyrat.tmux`](https://raw.githubusercontent.com/graelo/tmux-copyrat/main/tmux-copyrat.tmux) file for proper syntax examples.

## Verification

After installation, you should be able to use the default key bindings:

- <kbd>prefix</kbd> + <kbd>t</kbd> + <kbd>h</kbd> - highlight hashes
- <kbd>prefix</kbd> + <kbd>t</kbd> + <kbd>u</kbd> - highlight URLs
- <kbd>prefix</kbd> + <kbd>t</kbd> + <kbd>d</kbd> - highlight dates
- <kbd>prefix</kbd> + <kbd>t</kbd> + <kbd>i</kbd> - highlight IP addresses

See [CONFIGURATION.md](CONFIGURATION.md) for customization options.
