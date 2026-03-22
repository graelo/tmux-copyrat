# Contributing

## Build

```sh
cargo build                # debug
make release               # release with native CPU opts
```

## Test

```sh
cargo test                 # all tests
cargo test test_name       # single test
cargo test module::tests   # module tests
```

## Code Coverage

Requires nightly and `grcov`:

```sh
rustup component add llvm-tools-preview
cargo install grcov
rustup toolchain install nightly
make coverage
```

Report output: `./coverage/index.html`

## Manpages

Manpages live in `man/` as roff source: `tmux-copyrat.1` and `copyrat.1`.

Preview with:

```sh
mandoc man/tmux-copyrat.1 | less
mandoc man/copyrat.1 | less
```

Lint with:

```sh
mandoc -Tlint man/tmux-copyrat.1
mandoc -Tlint man/copyrat.1
```

When to update them:

- Adding, removing, or renaming a CLI flag
- Changing a default value
- Adding or removing a named pattern
- Changing key bindings or runtime controls

The version and date in the `.TH` header should be updated on each release.

## Submitting Changes

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the MIT license, shall be
licensed as MIT, without any additional terms or conditions.
