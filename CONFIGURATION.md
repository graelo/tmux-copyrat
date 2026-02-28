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
- `@copyrat-focused-fg/bg` - Focused match colors
- `@copyrat-selected-fg/bg` - Selected match colors (multi-select mode)
- `@copyrat-hint-fg/bg` - Hint colors

### Behavior

- `@copyrat-alphabet` - Hint character set (default: `dvorak`)
- `@copyrat-reverse` - Hint direction (default: `true`)
- `@copyrat-unique-hint` - Same hints for identical text (default: `true`)
- `@copyrat-focus-wrap-around` - Focus wrapping (default: `false`)
- `@copyrat-hint-alignment` - Hint position: `leading/center/trailing`
- `@copyrat-hint-style` - Styling: `bold/italic/underline/surround`
- `@copyrat-hint-surroundings` - Surround characters (default: `{}`)
- `@copyrat-default-output` - Default output: `tmux` or `clipboard` (default:
  `tmux`)
- `@copyrat-multi-select` - Enable multi-select mode (default: `false`)
- `@copyrat-separator` - Separator when joining multi-selected texts (default: 1
  space)

### Custom Bindings

- `@copyrat-bind-{key}` - Override or add pattern bindings

## Basic Configuration

### Key Binding Setup

Two keyswitches are available:

- `@copyrat-keyswitch` (default: `t`) - searches the **visible pane area**
- `@copyrat-keyswitch-history` (default: `T`) - searches the **entire scrollback
  history**

**Tip**: The default `t` key overrides tmux's built-in `prefix + t` (show time).
If you use that feature, change the keyswitch to a non-conflicting key, for
example:

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

# Currently focused match
set -g @copyrat-focused-fg "magenta"     # Default
set -g @copyrat-focused-bg "none"        # Default

# Selected matches (multi-select mode)
set -g @copyrat-selected-fg "green"      # Default
set -g @copyrat-selected-bg "none"       # Default

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

If you don't want a keyswitch at all, you'll have to create your own bindings,
as described in
[BYOB - Bring Your Own Bindings](#byob---bring-your-own-bindings).

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

### Multi-Select Mode

By default, typing a complete hint immediately copies the matched text and
exits. Multi-select mode lets you select multiple spans before confirming.

```tmux
# Enable multi-select
set -g @copyrat-multi-select "true"

# Separator used to join selected texts (default: space)
set -g @copyrat-separator " "
```

When multi-select is enabled:

| Key                              | Action                                                                |
| -------------------------------- | --------------------------------------------------------------------- |
| <kbd>hint chars</kbd>            | Toggle that span's selection on/off                                   |
| <kbd>Tab</kbd>                   | Toggle the currently focused span                                     |
| <kbd>n</kbd> / <kbd>N</kbd>      | Move focus to next/previous span                                      |
| <kbd>Enter</kbd> or <kbd>y</kbd> | Confirm: copy all selected texts (joined by separator) to tmux buffer |
| <kbd>Y</kbd>                     | Confirm: copy all selected texts to system clipboard                  |
| <kbd>Space</kbd>                 | Toggle output destination                                             |
| <kbd>Esc</kbd>                   | Cancel and exit                                                       |

Selected spans are highlighted with the `selected-fg/bg` colors. Hints remain
visible on selected spans so you can toggle them off again.

If you press <kbd>Enter</kbd> or <kbd>y</kbd> without selecting any spans, the
focused span is copied (same as single-select behavior).

Mistyped keys are silently ignored in multi-select mode instead of exiting.

The `copyrat` standalone binary also supports multi-select via the
`--multi-select` (`-m`) and `--separator` (`-S`) flags:

```console
echo "127.0.0.1 and 192.168.1.1 and hello@world.com" | copyrat -A --multi-select
```

## Customize Copyrat Key Bindings

This shows how to use the `@copyrat-bind-{key}` options to override defaults or
add new patterns. Read the next section if you want to bypass these mechanics.

### Syntax

- `pattern-name {name}` - Use built-in pattern
- `custom-pattern {regex}` - Use custom regex
- `""` (empty) - Disable binding

### Available Patterns

| Pattern name        | Description                       | Default key |
| ------------------- | --------------------------------- | ----------- |
| `command-line-args` | Command line arguments            | `a`         |
| `hexcolor`          | Hex color codes (`#aa00f5`)       | `c`         |
| `datetime`          | Dates or datetimes                | `d`         |
| `docker`            | Docker/Podman container IDs       | `D`         |
| `email`             | Email addresses                   | `e`         |
| `digits`            | Strings of 4+ digits              | `G`         |
| `sha`               | SHA-1/SHA-2 hashes (short & long) | `h`         |
| `markdown-url`      | Markdown URLs `[...](url)`        | `m`         |
| `path`              | Absolute & relative file paths    | `p`         |
| `pointer-address`   | Hex numbers / pointer addresses   | `P`         |
| `quoted-single`     | Strings inside single quotes      | `q`\*       |
| `quoted-double`     | Strings inside double quotes      | `q`\*       |
| `quoted-backtick`   | Strings inside backticks          | `q`\*       |
| `url`               | URLs                              | `u`         |
| `uuid`              | UUIDs                             | `U`         |
| `version`           | Version numbers                   | `v`         |
| `ipv4`              | IPv4 addresses                    | `4`         |
| `ipv6`              | IPv6 addresses                    | `6`         |

\* The `q` key activates all three `quoted-*` patterns together.

The `space` key activates all patterns at once (noisy and potentially slower).

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

## BYOB - Bring Your Own Bindings

You can of course create manual bindings that invoke `tmux-copyrat run`
directly. This is useful when you want to bypass the keytable for single-key
access (`prefix + <key>` instead of `prefix + t + <key>`), or when you need
flags that `@copyrat-bind-*` doesn't expose.

**Simple example** — bind `prefix + u` to search URLs:

```tmux
bind-key u new-window -d -n "[copyrat]" \
    'tmux-copyrat run --window-name "[copyrat]" \
    --clipboard-exe "pbcopy" \
    --alphabet dvorak --reverse \
    --pattern-name url'
```

**Trade-off**: manual bindings don't automatically pick up all `@copyrat-*`
options (colors, hint style, etc.) — you must pass the flags you need explicitly
or read them from tmux options as shown above.

**History search**: to search the entire scrollback instead of the visible area,
change `--capture-region visible-area` to `--capture-region entire-history`.

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
set -g @copyrat-selected-fg "green"
set -g @copyrat-selected-bg "none"
set -g @copyrat-hint-fg "yellow"
set -g @copyrat-hint-bg "black"

# Behavior
set -g @copyrat-alphabet "qwerty-homerow"
set -g @copyrat-reverse "false"
set -g @copyrat-focus-wrap-around "true"
set -g @copyrat-hint-alignment "center"
set -g @copyrat-hint-style "bold"
set -g @copyrat-default-output "clipboard"
set -g @copyrat-multi-select "true"
set -g @copyrat-separator "\n"

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
