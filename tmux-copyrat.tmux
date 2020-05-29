#!/usr/bin/env bash

CURRENT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

DEFAULT_COPYRAT_KEY="space"
COPYRAT_KEY=$(tmux show-option -gqv @copyrat-key)
COPYRAT_KEY=${COPYRAT_KEY:-$DEFAULT_COPYRAT_KEY}

DEFAULT_COPYRAT_WINDOW_NAME="[copyrat]"
COPYRAT_WINDOW_NAME=$(tmux show-option -gqv @copyrat-window-name)
COPYRAT_WINDOW_NAME=${COPYRAT_WINDOW_NAME:-$DEFAULT_COPYRAT_WINDOW_NAME}

BINARY="${CURRENT_DIR}/target/release/tmux-copyrat"

tmux bind-key ${COPYRAT_KEY} new-window -d -n ${COPYRAT_WINDOW_NAME} "${BINARY} --window-name ${COPYRAT_WINDOW_NAME} --reverse --unique"

if [ ! -f "$BINARY" ]; then
  cd "${CURRENT_DIR}" && cargo build --release
fi
