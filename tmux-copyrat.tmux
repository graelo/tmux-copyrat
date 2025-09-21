#!/usr/bin/env bash

# This scripts provides a default configuration for tmux-copyrat options and
# key bindings. It is run only once at tmux launch.
#
# IMPORTANT: DO NOT MODIFY THIS FILE DIRECTLY!
# If you're using TPM, your changes will be lost when the plugin is updated.
#
# Instead, customize the plugin by adding options to your ~/.tmux.conf:
#
#   set -g @copyrat-keytable "foobar"
#   set -g @copyrat-keyswitch "z"
#   set -g @copyrat-span-bg "magenta"
#   set -g @copyrat-window-name "[search]"
#   set -g @copyrat-clipboard-exe "pbcopy"
#
# and custom bindings like:
#
#   bind-key -T foobar h new-window -d -n "[copyrat]" '/path/to/tmux-copyrat --window-name "[copyrat]" --pattern-name urls'
#                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
#
# Please avoid modifying this script as it may break the integration with
# `tmux-copyrat`.
#
#
# Just make sure you first open a named window in the background and provide
# that name to the binary `tmux-copyrat`.
#
# Don't even try to run tmux-copyrat with run-shell, this cannot work because
# Tmux launches these processes without attaching them to a pty.

# You can also entirely ignore this file (not even source it) and define all
# options and bindings in your `tmux.conf`.

# Get the current directory where this script is located
CURRENT_DIR="$( cd "$( dirname "$0" )" && pwd )"

# Function to ensure binary is available
ensure_binary_available() {
    local installer_script="${CURRENT_DIR}/install-binary.sh"
    local binary_path="${CURRENT_DIR}/tmux-copyrat"
    
    # If binary already exists and is executable, we're good
    if [[ -x "$binary_path" ]]; then
        return 0
    fi
    
    # If installer script exists, run it quietly
    if [[ -x "$installer_script" ]]; then
        if "$installer_script" --quiet; then
            return 0
        else
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] tmux-copyrat: Warning: Failed to install binary automatically" >&2
            return 1
        fi
    else
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] tmux-copyrat: Warning: Installer script not found" >&2
        return 1
    fi
}

# Try to ensure binary is available (install if needed)
ensure_binary_available

# Set BINARY variable - prefer local downloaded binary, fall back to system PATH
if [[ -x "${CURRENT_DIR}/tmux-copyrat" ]]; then
    BINARY="${CURRENT_DIR}/tmux-copyrat"
elif [[ -x "${CURRENT_DIR}/copyrat" ]]; then
    BINARY="${CURRENT_DIR}/copyrat"
else
    BINARY=$(which tmux-copyrat 2>/dev/null || which copyrat 2>/dev/null || echo "")
fi

# Check if we have a usable binary
if [[ -z "$BINARY" || ! -x "$BINARY" ]]; then
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] tmux-copyrat: Error: tmux-copyrat binary not found." >&2
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] tmux-copyrat: Please install manually or ensure it's in your PATH." >&2
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] tmux-copyrat: Visit https://github.com/graelo/tmux-copyrat/releases for manual download." >&2
    exit 1
fi


#
# Top-level options
#

setup_option () {
    opt_name=$1
    default_value=$2
    current_value=$(tmux show-option -gqv @copyrat-${opt_name})
    value=${current_value:-${default_value}}
    tmux set-option -g @copyrat-${opt_name} ${value}
}


# Sets the window name which copyrat should use when running, providing a
# default value in case @copyrat-window-name was not defined.
setup_option "window-name" "[copyrat]"

# Get that window name as a local variable for use in pattern bindings below.
window_name=$(tmux show-option -gqv @copyrat-window-name)

# Sets the keytable for all bindings, providing a default if @copyrat-keytable
# was not defined. Keytables open a new shortcut space: if 't' is the switcher
# (see below), prefix + t + <your-shortcut>
setup_option "keytable" "cpyrt"

# Sets the key to access the keytable: prefix + <key> + <your-shortcut>
# providing a default if @copyrat-keyswitch is not defined.
setup_option "keyswitch" "t"

keyswitch=$(tmux show-option -gv @copyrat-keyswitch)
keytable=$(tmux show-option -gv @copyrat-keytable)
tmux bind-key ${keyswitch} switch-client -T ${keytable}


#
# Pattern bindings
#

if [[ "$OSTYPE" == darwin* ]]; then
  setup_option "clipboard-exe" "pbcopy"
else
  if [[ "$XDG_SESSION_TYPE" == wayland ]]; then
    setup_option "clipboard-exe" "wl-copy"
  else
    setup_option "clipboard-exe" "xclip -selection clipboard"
  fi
fi
clipboard_exe=$(tmux show-option -gv @copyrat-clipboard-exe)

setup_pattern_binding () {
    key=$1
    pattern_arg="$2"
    # The default window name `[copyrat]` has to be single quoted because it is
    # interpreted by the shell when launched by tmux.
    tmux bind-key -T ${keytable} ${key} new-window -d -n ${window_name} "${BINARY} run --window-name '"${window_name}"' --clipboard-exe ${clipboard_exe} --reverse --unique-hint ${pattern_arg}"
}

# prefix + t + a searches for command-line arguments
setup_pattern_binding "a" "--pattern-name command-line-args"
# prefix + t + c searches for hex colors #aa00f5
setup_pattern_binding "c" "--pattern-name hexcolor"
# prefix + t + d searches for dates or datetimes
setup_pattern_binding "d" "--pattern-name datetime"
# prefix + t + D searches for docker shas
setup_pattern_binding "D" "--pattern-name docker"
# prefix + t + e searches for email addresses (see https://www.regular-expressions.info/email.html)
setup_pattern_binding "e" "--pattern-name email"
# prefix + t + G searches for any string of 4+ digits
setup_pattern_binding "G" "--pattern-name digits"
# prefix + t + h searches for SHA1/2 short or long hashes
setup_pattern_binding "h" "--pattern-name sha"
# prefix + t + m searches for Markdown URLs [...](matched.url)
setup_pattern_binding "m" "--pattern-name markdown-url"
# prefix + t + p searches for absolute & relative paths
setup_pattern_binding "p" "--pattern-name path"
# prefix + t + P searches for hex numbers: 0xbedead
setup_pattern_binding "P" "--pattern-name pointer-address"
# prefix + t + q searches for strings inside single|double|backticks
setup_pattern_binding "q" "-x quoted-single -x quoted-double -x quoted-backtick"
# prefix + t + u searches for URLs
setup_pattern_binding "u" "--pattern-name url"
# prefix + t + U searches for UUIDs
setup_pattern_binding "U" "--pattern-name uuid"
# prefix + t + v searches for version numbers
setup_pattern_binding "v" "--pattern-name version"
# prefix + t + 4 searches for IPV4
setup_pattern_binding "4" "--pattern-name ipv4"
# prefix + t + 6 searches for IPV6
setup_pattern_binding "6" "--pattern-name ipv6"
# prefix + t + Space searches for all known patterns (noisy and potentially slower)
setup_pattern_binding "space" "--all-patterns"

# prefix + t + / prompts for a pattern and search for it
tmux bind-key -T ${keytable} "/" command-prompt -p "search:" "new-window -d -n '${window_name}' \"${BINARY}\" run --window-name '${window_name}' --reverse --unique-hint --custom-pattern %%"
