#!/usr/bin/env bash

# This scripts provides a default configuration for tmux-copyrat options and key bindings.
# It is run only once at tmux launch.
#
# Each option and binding can be overridden in your `tmux.conf` by defining options like
#
#   set -g @copyrat-keytable "foobar"
#   set -g @copyrat-keyswitch "z"
#   set -g @copyrat-match-bg "magenta"
#
# and bindings like
#
#   bind-key -T foobar h new-window -d -n "[copyrat]" '/path/to/tmux-copyrat --window-name "[copyrat]" --pattern-name urls'
#                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
#
# changing this will probably break integration     -
#
# Just make sure you first open a named window in the background and provide that name to tmux-copyrat.
#
# Don't even try to run tmux-copyrat with run-shell, this cannot work because Tmux launches these processes
# without attaching them to a pty.

# You could also entirely ignore this file (not even source it) and define all options and bindings
# in your `tmux.conf`.

CURRENT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BINARY="${CURRENT_DIR}/tmux-copyrat"

setup_option() {
	local opt_name=$1
	local default_value=$2
	local current_value=$(tmux show-option -gqv @copyrat-${opt_name})
	value=${current_value:-${default_value}}
	tmux set-option -g @copyrat-${opt_name} ${value}
}


# Sets the window name when copyrat is run, providing a default if @copyrat-window-name was not defined.
setup_option "window-name" "[copyrat]"

# Sets the keytable for all bindings, providing a default if @copyrat-keytable was not defined.
# Keytables open a new shortcut space: if 't' is the switcher (see below), prefix + t + <your-shortcut>
setup_option "keytable" "cpyrt"

# Sets the key to access the keytable: prefix + <key> + <your-shortcut>
# providing a default if @copyrat-keyswitch is not defined.
setup_option "keyswitch" "t"

local keyswitch=$(tmux show-option @copyrat-keyswitch)
local keytable=$(tmux show-option @copyrat-keytable)
tmux bind-key ${keyswitch} -T ${keytable}

setup_binding() {
	local key=$1
	local pattern_name="$2"
	local window_name=$(tmux show-option -gqv @copyrat-window-name)
	tmux bind-key -T ${keytable} $key new-window -d -n ${window_name} "${BINARY} --window-name ${window_name} --reverse --unique ${pattern_name}"
}

# prefix + t + p searches for absolute & relative paths
setup_pattern_binding "p" "--pattern-name path"
# prefix + t + u searches for URLs
setup_pattern_binding "u" "--pattern-name url"
# prefix + t + m searches for Markdown URLs [...](matched.url)
setup_pattern_binding "m" "--pattern-name markdown-url"
# prefix + t + h searches for SHA1/2 (hashes)
setup_pattern_binding "h" "--pattern-name sha"
# prefix + t + e searches for email addresses (see https://www.regular-expressions.info/email.html)
setup_pattern_binding "e" "--pattern-name email"
# prefix + t + D searches for docker shas
setup_pattern_binding "D" "--pattern-name docker"
# prefix + t + c searches for hex colors #aa00f5
setup_pattern_binding "c" "--pattern-name hexcolor"
# prefix + t + U searches for UUIDs
setup_pattern_binding "U" "--pattern-name uuid"
# prefix + t + d searches for any string of 4+ digits
setup_pattern_binding "d" "--pattern-name digits"
# prefix + t + m searches for hex numbers: 0xbedead
setup_pattern_binding "m" "--pattern-name mem-address"
# prefix + t + 4 searches for IPV4
setup_pattern_binding "4" "--pattern-name ipv4"
# prefix + t + 6 searches for IPV6
setup_pattern_binding "6" "--pattern-name ipv6"
# prefix + t + Space searches for all known patterns (noisy and potentially slower)
setup_pattern_binding "space" "--all-patterns"

# prefix + t + / prompts for a pattern and search for it
tmux bind-key -T ${keytable} "/" command-prompt -p "search:" 'new-window -d -n ${COPYRAT_WINDOW_NAME} "${BINARY} --window-name ${COPYRAT_WINDOW_NAME} --reverse --unique --custom-pattern %%"'


# Auto-install is currently disabled as it requires the user to have cargo installed.
# if [ ! -f "$BINARY" ]; then
#   cd "${CURRENT_DIR}" && cargo build --release
# fi
