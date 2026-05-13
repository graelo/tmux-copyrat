#!/bin/bash

set -e

CRATE=tmux-copyrat
MSRV=1.95

get_rust_version() {
  local array=($(rustc --version));
  echo "${array[1]}";
  return 0;
}
RUST_VERSION=$(get_rust_version)

check_version() {
  IFS=. read -ra rust <<< "$RUST_VERSION"
  IFS=. read -ra want <<< "$1"
  [[ "${rust[0]}" -gt "${want[0]}" ||
   ( "${rust[0]}" -eq "${want[0]}" &&
     "${rust[1]}" -ge "${want[1]}" )
  ]]
}

echo "Testing $CRATE on rustc $RUST_VERSION"
if ! check_version $MSRV ; then
  echo "The minimum for $CRATE is rustc $MSRV"
  exit 1
fi

NEXTEST_PROFILE=""
if [ -n "$CI" ]; then
  NEXTEST_PROFILE="--profile ci"
fi

set -x

# test the default build
cargo build --locked
cargo nextest run --locked $NEXTEST_PROFILE

# doc tests (not supported by nextest)
cargo test --locked --doc

# CLI smoke test (release binaries). CARGO_BUILD_TARGET (set in the compat
# matrix) redirects output to target/<target>/release; Git Bash on Windows
# reports OSTYPE=msys.
cargo build --locked --release

for bin in copyrat tmux-copyrat; do
  BIN="target/${CARGO_BUILD_TARGET:+${CARGO_BUILD_TARGET}/}release/${bin}"
  case "${OSTYPE:-}" in
    msys*|cygwin*) BIN="${BIN}.exe" ;;
  esac
  "${BIN}" --help
done
