# Contributing

## Build, test, check

The `Makefile` is the canonical definition of every local task; run
`make help` to list them. The ones you need day to day:

```sh
cargo build                # debug build
make release               # release build with native CPU opts
make test                  # full test suite
make check                 # fmt + lint + test — run before `git push`
make check-all             # adds audits, commit lint, docs — run before a PR
make fix                   # auto-format and apply clippy fixes
```

To run a single test, use nextest directly:

```sh
cargo nextest run test_name
cargo nextest run module::tests
```

## Code coverage

```sh
make coverage
```

Report output: `./target/llvm-cov/html/index.html`

## Manpages

Manpages live in `man/` as roff source: `tmux-copyrat.1` and `copyrat.1`.

Preview with:

```sh
mandoc man/tmux-copyrat.1 | less
mandoc man/copyrat.1 | less
```

Lint with `make man`.

When to update them:

- Adding, removing, or renaming a CLI flag
- Changing a default value
- Adding or removing a named pattern
- Changing key bindings or runtime controls

Key bindings and named patterns are also defined in `tmux-copyrat.tmux`, which
is compiled into the binary — update it alongside the manpages.

The version and date in the `.TH` header should be updated on each release.

## Submitting Changes

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the MIT license, shall be
licensed as MIT, without any additional terms or conditions.
