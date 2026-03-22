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

## Submitting Changes

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the MIT license, shall be
licensed as MIT, without any additional terms or conditions.
