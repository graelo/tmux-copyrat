#!/usr/bin/env bash

CURRENT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

DEFAULT_COPYRAT_KEY="space"
COPYRAT_KEY=$(tmux show-option -gqv @copyrat-key)
COPYRAT_KEY=${COPYRAT_KEY:-$DEFAULT_COPYRAT_KEY}

BINARY="${CURRENT_DIR}/target/release/tmux-copyrat"

tmux bind-key $COPYRAT_KEY run-shell -b "${BINARY} -T"

if [ ! -f "$BINARY" ]; then
  cd "${CURRENT_DIR}" && cargo build --release
fi
