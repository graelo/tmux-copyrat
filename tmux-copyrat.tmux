#!/usr/bin/env bash

# This scripts provides a default configuration for tmux-copyrat options and
# key bindings. It is run only once at tmux launch.
#
# Each option and binding can be overridden in your `tmux.conf` by defining
# options like
#
#   set -g @copyrat-keytable "foobar"
#   set -g @copyrat-keyswitch "z"
#   set -g @copyrat-span-bg "magenta"
#
# and bindings like
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

BINARY=$(which tmux-copyrat)
# CURRENT_DIR="$( cd "$( dirname "$0" )" && pwd )"
# BINARY=${CURRENT_DIR}/tmux-copyrat


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

setup_pattern_binding () {
    key=$1
    pattern_arg="$2"
    # The default window name `[copyrat]` has to be single quoted because it is
    # interpreted by the shell when launched by tmux.
    tmux bind-key -T ${keytable} ${key} new-window -d -n ${window_name} "${BINARY} run --window-name '"${window_name}"' --reverse --unique-hint ${pattern_arg}"
}

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
