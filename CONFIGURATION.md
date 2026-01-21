# Configuration

Customize tmux-copyrat by adding options to your `~/.tmux.conf` file.

**Important**: Never modify plugin files directly in
`~/.tmux/plugins/tmux-copyrat/` - changes will be lost on updates.

**Note**: Reload your configuration with `tmux source-file ~/.tmux.conf` to
apply changes.

## Configuration Options

### Plugin Settings

- `@copyrat-keyswitch` - Key to search visible area (default: `t`)
- `@copyrat-keyswitch-history` - Key to search entire history (default: `T`)
- `@copyrat-keytable` - Keytable name (default: `cpyrt`)
- `@copyrat-window-name` - Window name (default: `[copyrat]`)
- `@copyrat-clipboard-exe` - Clipboard command (auto-detected)

### Colors

- `@copyrat-text-fg/bg` - Base text colors
- `@copyrat-span-fg/bg` - Matched text colors
- `@copyrat-focused-fg/bg` - Selected match colors
- `@copyrat-hint-fg/bg` - Hint colors

### Behavior

- `@copyrat-alphabet` - Hint character set (default: `dvorak`)
- `@copyrat-reverse` - Hint direction (default: `true`)
- `@copyrat-unique-hint` - Same hints for identical text (default: `true`)
- `@copyrat-focus-wrap-around` - Focus wrapping (default: `false`)
- `@copyrat-hint-alignment` - Hint position: `leading/center/trailing`
- `@copyrat-hint-style` - Styling: `bold/italic/underline/surround`
- `@copyrat-hint-surroundings` - Surround characters (default: `{}`)
- `@copyrat-default-output` - Default output: `tmux` or `clipboard` (default: `tmux`)

### Custom Bindings

- `@copyrat-bind-{key}` - Override or add pattern bindings

## Basic Configuration

### Key Binding Setup

Two keyswitches are available:
- `@copyrat-keyswitch` (default: `t`) - searches the **visible pane area**
- `@copyrat-keyswitch-history` (default: `T`) - searches the **entire scrollback history**

```tmux
# Change the main keys from 't'/'T' to 'c'/'C'
set -g @copyrat-keyswitch "c"
set -g @copyrat-keyswitch-history "C"

# Use custom keytable name
set -g @copyrat-keytable "search"

# Custom window name
set -g @copyrat-window-name "[search]"
```

### Clipboard Integration

Auto-detected by system:

- **macOS**: `pbcopy`
- **Wayland**: `wl-copy`
- **X11**: `xclip -selection clipboard`

```tmux
# Override with custom command
set -g @copyrat-clipboard-exe "xsel --clipboard --input"
```

## Color Configuration

Colors support standard names (`black`, `red`, `blue`, etc.), bright variants
(`bright-red`, `bright-blue`), or `none` for transparency.

```tmux
# Base text (unmatched areas)
set -g @copyrat-text-fg "bright-cyan"    # Default
set -g @copyrat-text-bg "none"           # Default

# Matched patterns
set -g @copyrat-span-fg "blue"           # Default
set -g @copyrat-span-bg "none"           # Default

# Currently selected match
set -g @copyrat-focused-fg "magenta"     # Default
set -g @copyrat-focused-bg "none"        # Default

# Hint characters
set -g @copyrat-hint-fg "yellow"         # Default
set -g @copyrat-hint-bg "none"           # Default
```

## Behavior Configuration

### Hint Generation

```tmux
# Keyboard layouts: qwerty, azerty, qwertz, dvorak, colemak
# Modifiers: -homerow, -left-hand, -right-hand
set -g @copyrat-alphabet "qwerty-homerow"

# Hint assignment direction (true = bottom-up, false = top-down)
set -g @copyrat-reverse "false"

# Keep same hints for identical text
set -g @copyrat-unique-hint "true"

# Wrap focus at first/last match
set -g @copyrat-focus-wrap-around "true"
```

### Hint Appearance

```tmux
# Position: leading, center, trailing
set -g @copyrat-hint-alignment "center"

# Style: bold, italic, underline, surround, or empty for none
set -g @copyrat-hint-style "bold"

# Characters for surround style (exactly 2 chars)
set -g @copyrat-hint-surroundings "[]"
```

### Search Area

The search area is determined by which keyswitch you use:
- `prefix + t + <key>` - searches the visible pane area
- `prefix + T + <key>` - searches the entire scrollback history

### Output Destination

By default, selections are copied to the tmux buffer. You can change this to
copy to the system clipboard instead:

```tmux
# tmux (buffer) or clipboard
set -g @copyrat-default-output "clipboard"
```

The output destination can be toggled at runtime with the `space` key.

## Custom Key Bindings

Override defaults or add new patterns using `@copyrat-bind-{key}` options.

### Syntax

- `pattern-name {name}` - Use built-in pattern
- `custom-pattern {regex}` - Use custom regex
- `""` (empty) - Disable binding

### Available Patterns

Command line: `command-line-args`, `digits`, `path`
Web: `url`, `email`, `markdown-url`
Network: `ipv4`, `ipv6`
Code: `sha`, `docker`, `uuid`, `hexcolor`, `pointer-address`
Other: `datetime`, `version`

### Examples

```tmux
# Override defaults
set -g @copyrat-bind-u "pattern-name email"    # u = emails not URLs
set -g @copyrat-bind-6 "pattern-name ipv4"     # 6 = IPv4 not IPv6

# Add custom patterns
set -g @copyrat-bind-M "custom-pattern \
'[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}'"

# Remove bindings
set -g @copyrat-bind-D ""                      # Disable D
```

## Advanced Custom Bindings

For patterns or options not supported by `@copyrat-bind-*`, create manual
bindings. Most users won't need this section.

```tmux
# Get configured settings
keytable=$(tmux show-option -gv @copyrat-keytable)
window_name=$(tmux show-option -gv @copyrat-window-name)

# Manual binding with custom options
bind-key -T $keytable H \
    new-window -d -n "$window_name" \
    'tmux-copyrat run --window-name "'"$window_name"'" \
    --alphabet qwerty --hint-style bold --span-bg red \
    --pattern-name sha'
```

**Note**: Manual bindings bypass automatic `@copyrat-*` option inclusion.

## Complete Example

```tmux
# Basic setup
set -g @copyrat-keyswitch "c"
set -g @copyrat-keyswitch-history "C"
set -g @copyrat-keytable "copy"
set -g @copyrat-window-name "[copy]"
set -g @copyrat-clipboard-exe "xsel --clipboard --input"

# Colors (dark theme)
set -g @copyrat-span-fg "white"
set -g @copyrat-span-bg "blue"
set -g @copyrat-focused-fg "black"
set -g @copyrat-focused-bg "cyan"
set -g @copyrat-hint-fg "yellow"
set -g @copyrat-hint-bg "black"

# Behavior
set -g @copyrat-alphabet "qwerty-homerow"
set -g @copyrat-reverse "false"
set -g @copyrat-focus-wrap-around "true"
set -g @copyrat-hint-alignment "center"
set -g @copyrat-hint-style "bold"
set -g @copyrat-default-output "clipboard"

# Custom bindings
set -g @copyrat-bind-m "pattern-name email"
set -g @copyrat-bind-M "custom-pattern \
'[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}'"
set -g @copyrat-bind-D ""  # Disable docker binding
```

Result:

- `prefix + c + h` → hashes (visible area)
- `prefix + C + h` → hashes (entire history)
- `prefix + c + u` → URLs
- `prefix + c + m` → emails
- `prefix + c + M` → MAC addresses
- All use your custom colors and behavior settings
