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
2. The plugin will automatically download the appropriate binary for your system on first use

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

The plugin will automatically download the appropriate binary for your system on first use.

## Method 3: Custom Key Bindings

For advanced users who prefer to manage their own key bindings, you can:

1. Download the binary using Method 2 (or manually download from [GitHub
   releases](https://github.com/graelo/tmux-copyrat/releases))
2. Skip sourcing the `tmux-copyrat.tmux` file
3. Create custom key bindings in your `~/.tmux.conf`

**Important**: When creating custom bindings, avoid using `run-shell` as tmux
*launches these processes without attaching them to a pty. Refer to the
*[`tmux-copyrat.tmux`](https://raw.githubusercontent.com/graelo/tmux-copyrat/main/tmux-copyrat.tmux)
*file for proper syntax examples.

## Customization

**Important**: If using TPM, never modify the plugin files directly in
`~/.tmux/plugins/tmux-copyrat/`!

When using TPM, any changes you make to plugin files will be lost when you
update the plugin. Instead, customize the plugin by adding options to your
`~/.tmux.conf`:

```tmux
# Example customizations in ~/.tmux.conf
set -g @copyrat-keyswitch "c"           # Use 'c' instead of 't'
set -g @copyrat-window-name "[search]"  # Custom window name
set -g @copyrat-keytable "search"       # Custom keytable name
```

See [CONFIGURATION.md](CONFIGURATION.md) for all available options and examples.

## Updating

The plugin only downloads the binary if it doesn't exist. To get the latest version:

```bash
# Navigate to plugin directory
cd ~/.tmux/plugins/tmux-copyrat

# Force download latest binary
./install-binary.sh --force
```

## Verification

After installation, you should be able to use the default key bindings:

- <kbd>prefix</kbd> + <kbd>t</kbd> + <kbd>h</kbd> - highlight hashes
- <kbd>prefix</kbd> + <kbd>t</kbd> + <kbd>u</kbd> - highlight URLs
- <kbd>prefix</kbd> + <kbd>t</kbd> + <kbd>d</kbd> - highlight dates
- <kbd>prefix</kbd> + <kbd>t</kbd> + <kbd>i</kbd> - highlight IP addresses

See [CONFIGURATION.md](CONFIGURATION.md) for customization options.
