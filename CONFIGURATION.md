# Configuration

You can customize tmux-copyrat's behavior by adding options to your `~/.tmux.conf` file. 

**Important**: Never modify the plugin files directly in `~/.tmux/plugins/tmux-copyrat/` - your changes will be lost when the plugin is updated via TPM.

**Note**: For changes to take effect, you'll need to reload your tmux configuration with `tmux source-file ~/.tmux.conf`.

## Available Options

### Plugin Configuration

- [@copyrat-keyswitch](#copyrat-keyswitch) - Key to enter copyrat mode
- [@copyrat-keytable](#copyrat-keytable) - Keytable name for bindings
- [@copyrat-window-name](#copyrat-window-name) - Window name for copyrat
- [@copyrat-clipboard-exe](#copyrat-clipboard-exe) - Clipboard command

### Color Options

- [@copyrat-text-fg](#copyrat-text-fg) - Foreground color of base (unmatched) text
- [@copyrat-text-bg](#copyrat-text-bg) - Background color of base (unmatched) text
- [@copyrat-span-fg](#copyrat-span-fg) - Foreground color of matched text
- [@copyrat-span-bg](#copyrat-span-bg) - Background color of matched text
- [@copyrat-focused-fg](#copyrat-focused-fg) - Foreground color of focused match
- [@copyrat-focused-bg](#copyrat-focused-bg) - Background color of focused match
- [@copyrat-hint-fg](#copyrat-hint-fg) - Foreground color of hint text
- [@copyrat-hint-bg](#copyrat-hint-bg) - Background color of hint text

### Behavior Options

- [@copyrat-alphabet](#copyrat-alphabet) - Character set for hints
- [@copyrat-reverse](#copyrat-reverse) - Direction of hint assignment
- [@copyrat-unique-hint](#copyrat-unique-hint) - Consistent hints for identical text
- [@copyrat-focus-wrap-around](#copyrat-focus-wrap-around) - Focus wrapping behavior
- [@copyrat-hint-alignment](#copyrat-hint-alignment) - Hint alignment relative to text
- [@copyrat-hint-style](#copyrat-hint-style) - Visual style of hints
- [@copyrat-hint-surroundings](#copyrat-hint-surroundings) - Characters for surround style
- [@copyrat-capture-region](#copyrat-capture-region) - Text capture area

### Custom Patterns

- [Custom bindings](#custom-bindings) - Add your own key bindings and patterns

## Plugin Configuration Options

### @copyrat-keyswitch

**Default**: `t`

The key used after the tmux prefix to enter copyrat mode. All copyrat bindings start with `prefix + keyswitch`.

```tmux
# Use 'c' instead of 't' as the keyswitch
set -g @copyrat-keyswitch "c"

# Now bindings become: prefix + c + h (for hashes), prefix + c + u (for URLs), etc.
```

### @copyrat-keytable

**Default**: `cpyrt`

The tmux keytable name used for copyrat bindings. This creates a separate key namespace.

```tmux
# Use a custom keytable name
set -g @copyrat-keytable "search"
```

### @copyrat-window-name

**Default**: `[copyrat]`

The name of the tmux window that copyrat creates when highlighting patterns.

```tmux
# Use a custom window name
set -g @copyrat-window-name "[search]"
```

### @copyrat-clipboard-exe

**Default**: Auto-detected based on your system
- macOS: `pbcopy`
- Wayland: `wl-copy`  
- X11: `xclip -selection clipboard`

The command used to copy text to the system clipboard.

```tmux
# Use a custom clipboard command
set -g @copyrat-clipboard-exe "xsel --clipboard --input"
```

## Color Configuration Options

### @copyrat-text-fg

**Default**: `bright-cyan`

Foreground color of the base (unmatched) text.

```tmux
set -g @copyrat-text-fg "white"
```

### @copyrat-text-bg

**Default**: `none`

Background color of the base (unmatched) text.

```tmux
set -g @copyrat-text-bg "black"
```

### @copyrat-span-fg

**Default**: `blue`

Foreground color of the matched text spans.

```tmux
set -g @copyrat-span-fg "white"
```

### @copyrat-span-bg

**Default**: `none`

Background color of the matched text spans.

```tmux
set -g @copyrat-span-bg "blue"
```

### @copyrat-focused-fg

**Default**: `magenta`

Foreground color of the currently focused (selected) match.

```tmux
set -g @copyrat-focused-fg "white"
```

### @copyrat-focused-bg

**Default**: `none`

Background color of the currently focused (selected) match.

```tmux
set -g @copyrat-focused-bg "red"
```

### @copyrat-hint-fg

**Default**: `yellow`

Foreground color of the hint characters.

```tmux
set -g @copyrat-hint-fg "yellow"
```

### @copyrat-hint-bg

**Default**: `none`

Background color of the hint characters.

```tmux
set -g @copyrat-hint-bg "black"
```

## Behavior Configuration Options

### @copyrat-alphabet

**Default**: `dvorak`

The character set used for generating hints. Available options:
- `qwerty`, `azerty`, `qwertz`, `dvorak`, `colemak`
- Add modifiers: `-homerow`, `-left-hand`, `-right-hand`

```tmux
# Use QWERTY keyboard layout
set -g @copyrat-alphabet "qwerty"

# Use only home row keys from QWERTY
set -g @copyrat-alphabet "qwerty-homerow"

# Use only left-hand keys from Dvorak
set -g @copyrat-alphabet "dvorak-left-hand"
```

### @copyrat-reverse

**Default**: `true`

Whether to assign hints starting from the bottom of the screen (`true`) or from the top (`false`).

```tmux
# Assign hints from top to bottom
set -g @copyrat-reverse "false"
```

### @copyrat-unique-hint

**Default**: `true`

Whether to keep the same hint for identical text spans.

```tmux
# Give different hints to identical spans
set -g @copyrat-unique-hint "false"
```

### @copyrat-focus-wrap-around

**Default**: `false`

Whether focus should wrap around when reaching the first/last match.

```tmux
# Enable focus wrap-around
set -g @copyrat-focus-wrap-around "true"
```

### @copyrat-hint-alignment

**Default**: `leading`

How hints are aligned relative to their text spans:
- `leading` - At the beginning of the span
- `center` - In the middle of the span  
- `trailing` - At the end of the span

```tmux
# Center hints within their spans
set -g @copyrat-hint-alignment "center"
```

### @copyrat-hint-style

**Default**: `` (none)

Visual styling applied to hints:
- `` (empty) - No additional styling
- `bold` - Bold text
- `italic` - Italic text
- `underline` - Underlined text
- `surround` - Surrounded by characters (see @copyrat-hint-surroundings)

```tmux
# Make hints bold
set -g @copyrat-hint-style "bold"

# Surround hints with brackets
set -g @copyrat-hint-style "surround"
set -g @copyrat-hint-surroundings "[]"
```

### @copyrat-hint-surroundings

**Default**: `{}`

Characters used when `@copyrat-hint-style` is set to `surround`. Must be exactly 2 characters: opening and closing.

```tmux
# Use brackets instead of braces
set -g @copyrat-hint-surroundings "[]"

# Use parentheses
set -g @copyrat-hint-surroundings "()"
```

### @copyrat-capture-region

**Default**: `visible-area`

Which part of the terminal to search for patterns:
- `visible-area` - Only the currently visible screen
- `entire-history` - The entire scrollback history

```tmux
# Search through entire scrollback history
set -g @copyrat-capture-region "entire-history"
```

## Custom Bindings

All default key bindings automatically use your configured options from `@copyrat-*` settings. If you want to create additional bindings with custom patterns, you can add them to your `~/.tmux.conf`.

The plugin's existing bindings will respect all your configuration options, so in most cases you won't need custom bindings unless you want:
- Additional pattern types not covered by the defaults
- Bindings with different option combinations than your global settings

### Adding Custom Patterns

```tmux
# Get your configured settings
keytable=$(tmux show-option -gv @copyrat-keytable)
window_name=$(tmux show-option -gv @copyrat-window-name)

# Add a custom binding for MAC addresses
bind-key -T $keytable M new-window -d -n "$window_name" 'tmux-copyrat run --window-name "'"$window_name"'" --custom-pattern "[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}"'
```

### Advanced: Custom Bindings with Different Options

If you need bindings that behave differently from your global configuration:

```tmux
# Custom binding that uses different colors and behavior
bind-key -T cpyrt H new-window -d -n '[copyrat]' 'tmux-copyrat run --window-name "[copyrat]" --clipboard-exe pbcopy --alphabet qwerty --hint-style bold --span-bg red --pattern-name sha'
```

**Note**: The plugin automatically includes all your `@copyrat-*` options in the default bindings. Custom bindings bypass this automatic inclusion, so you'll need to specify options manually.

## Available Pattern Names

These are the built-in patterns you can use with `--pattern-name`:

- `command-line-args` - Command-line arguments  
- `hexcolor` - Hex color codes (#aa00f5)
- `datetime` - Dates and datetimes
- `docker` - Docker/Podman IDs
- `email` - Email addresses
- `digits` - Strings of 4+ digits
- `sha` - SHA-1/2 hashes (short and long)
- `markdown-url` - Markdown URLs `[text](url)`
- `path` - File paths (absolute and relative)
- `pointer-address` - Hex numbers and pointer addresses
- `url` - URLs
- `uuid` - UUIDs
- `version` - Version numbers
- `ipv4` - IPv4 addresses
- `ipv6` - IPv6 addresses

## Complete Example

Here's a complete configuration example for your `~/.tmux.conf`:

```tmux
# Plugin configuration
set -g @copyrat-keyswitch "c"
set -g @copyrat-keytable "copy"
set -g @copyrat-window-name "[copy]"
set -g @copyrat-clipboard-exe "xsel --clipboard --input"

# Color customization - dark theme
set -g @copyrat-span-fg "white"
set -g @copyrat-span-bg "blue"
set -g @copyrat-focused-fg "black"
set -g @copyrat-focused-bg "cyan"
set -g @copyrat-hint-fg "yellow"
set -g @copyrat-hint-bg "black"

# Behavior customization
set -g @copyrat-alphabet "qwerty-homerow"
set -g @copyrat-reverse "false"
set -g @copyrat-unique-hint "true"
set -g @copyrat-focus-wrap-around "true"
set -g @copyrat-hint-alignment "center"
set -g @copyrat-hint-style "bold"
set -g @copyrat-capture-region "entire-history"

# Custom binding for MAC addresses
keytable=$(tmux show-option -gv @copyrat-keytable)
window_name=$(tmux show-option -gv @copyrat-window-name)
bind-key -T $keytable M new-window -d -n "$window_name" 'tmux-copyrat run --window-name "'"$window_name"'" --custom-pattern "[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}:[a-f0-9]{2}"'
```

With this configuration:
- Press `prefix + c + h` to highlight hashes (using all your custom settings)
- Press `prefix + c + u` to highlight URLs (using all your custom settings)
- Press `prefix + c + M` to highlight MAC addresses (custom pattern)
- All copyrat windows will be named `[copy]`
- Text will be copied using `xsel`
- Uses QWERTY home row keys for hints
- Assigns hints from top to bottom
- Searches entire scrollback history
- Uses custom colors and bold hint styling

