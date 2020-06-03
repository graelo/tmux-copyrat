#!/usr/bin/env bash

# This script is run only once at tmux launch.
# It provides configuration based on the @copyrat-* options set in your `tmux.conf`.

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

# DEFAULT_COPYRAT_WINDOW_NAME="[copyrat]"
# COPYRAT_WINDOW_NAME=$(tmux show-option -gqv @copyrat-window-name)
# COPYRAT_WINDOW_NAME=${COPYRAT_WINDOW_NAME:-$DEFAULT_COPYRAT_WINDOW_NAME}
# # overrides with the same value, I did not bother writing a test
# tmux set-option -g @copyrat-window-name ${COPYRAT_WINDOW_NAME}

# Sets the keytable for all bindings, providing a default if @copyrat-keytable was not defined.
# Keytables open a new shortcut space: if 't' is the switcher (see below), prefix + t + <your-shortcut>
setup_option "keytable" "cpyrt"
# DEFAULT_COPYRAT_KEYTABLE="cpyrt"
# COPYRAT_KEYTABLE=$(tmux show-option -gqv @copyrat-keytable)
# COPYRAT_KEYTABLE=${COPYRAT_KEYTABLE:-$DEFAULT_COPYRAT_KEYTABLE}

# Sets the key to access the keytable: prefix + <key> + <your-shortcut>
# providing a default if @copyrat-keyswitch is not defined.
setup_option "keyswitch" "u"

local keyswitch=$(tmux show-option @copyrat-keyswitch)
local keytable=$(tmux show-option @copyrat-keytable)
tmux bind-key ${keyswitch} -T ${keytable}

setup_binding() {
	local key=$1
	local pattern_name=$2
	local window_name=$(tmux show-option -gqv @copyrat-window-name)
	tmux bind-key -T ${keytable} $key new-window -d -n ${window_name} "${BINARY} --window-name ${window_name} --reverse --unique"
}

# Search
setup_pattern_binding "space" ""
setup_pattern_binding "p" "--pattern-name path"
setup_pattern_binding "u" "--pattern-name url"
setup_pattern_binding "h" "--pattern-name sha"
setup_pattern_binding "d" "--pattern-name docker"
setup_pattern_binding "c" "--pattern-name hexcolor"
setup_pattern_binding "i" "--pattern-name ip"

# DEFAULT_COPYRAT_KEY="space"
# COPYRAT_KEY=$(tmux show-option -gqv @copyrat-key)
# COPYRAT_KEY=${COPYRAT_KEY:-$DEFAULT_COPYRAT_KEY}


# tmux bind-key -T ${keytable} "/" command-prompt "search:" new-window -d -n ${COPYRAT_WINDOW_NAME} "${BINARY} --window-name ${COPYRAT_WINDOW_NAME} --reverse --unique --custom-regex '%%'"

# if [ ! -f "$BINARY" ]; then
#   cd "${CURRENT_DIR}" && cargo build --release
# fi
