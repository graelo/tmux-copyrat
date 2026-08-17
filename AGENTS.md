# AGENTS.md

This file contains instructions for coding agents working in this repository.

- Repository: <https://github.com/graelo/tmux-copyrat>
- Prefer `gh` for GitHub operations.
- Do not mention an agent or assistant in issues, pull requests, comments, or
  commit messages.
- Do not expose private local information, including machine-specific paths.

## Project

`tmux-copyrat` is a tmux plugin that finds pattern-matched text in a pane,
shows keyboard hints, and copies a selected match. The crate ships two binaries:

- `tmux-copyrat`: tmux integration; captures a pane, presents the TUI, and
  writes the selection to the tmux buffer or system clipboard. It takes
  subcommands: `run` does the work, `init` prints the plugin config.
- `copyrat`: standalone program; reads stdin and writes the selected text to
  stdout.

Rust 1.95 or later is required. The crate uses edition 2024.

## Architecture

1. `tmux-copyrat` captures the active pane using `tmux capture-pane`.
2. `textbuf::Model` parses lines, finds matches, assigns hints, and builds the
   hint lookup trie.
3. `ui::ViewController` renders the alternate-screen TUI and handles input.
4. A `ui::Selection` is returned and copied to the configured destination.

Key modules:

- `src/textbuf/`: matching, hint generation, spans, and predefined regexes.
  Add or change named patterns in `regexes.rs`.
- `src/ui/`: terminal rendering, input handling, selection, colors, and hint
  styling.
- `src/config/`: Clap configuration for the standalone and tmux binaries.
- `src/tmux.rs`: pane capture, tmux options, and pane swapping.
- `src/bin/`: binary entry points.
- `tmux-copyrat.tmux`: the bash plugin entry point. Defines the default key
  bindings, the key-to-pattern-name mapping, and the `@copyrat-*` option
  surface. It is compiled into the binary with `include_str!` and printed by
  `tmux-copyrat init`, so changes to it ship with the crate.

## Verification

The `Makefile` is the canonical definition of local verification tasks. **Read
it before choosing or running verification commands**; do not duplicate its
command implementations here. `make help` lists every target.

The primary targets are:

- `make check`: pre-push gate (formatting, linting, and tests).
- `make check-all`: pre-PR gate (adds dependency, commit-message, Markdown,
  manpage, and GitHub Actions security checks).
- `make fix`: formats code and applies Clippy fixes.
- `make md`: lints Markdown against `rumdl.toml`. Note the 80-column `MD013`
  reflow rule — run this after editing any Markdown file.
- `make man`: lints both roff manpages.
- `make ci-security`: runs the Poutine and Zizmor GitHub Actions scans.

The check targets mirror the GitHub workflows and use locked dependency
resolution where applicable. They assume their external tools (for example
`cargo-nextest`, `cargo-deny`, `cargo-pants`, `convco`, `poutine`, `zizmor`,
`rumdl`, `mandoc`, and `cargo-llvm-cov`) are already installed locally.

For focused Rust tests, use `cargo nextest run <test_name>` or
`cargo nextest run <module::tests::name>`. The complete CI test sequence is
implemented in `ci/test_full.sh`; its Nextest CI profile is configured in
`.config/nextest.toml`.

## Documentation and releases

Keep user-facing documentation in sync with behavior:

- Update `README.md`, `CONFIGURATION.md`, or `INSTALLATION.md` when their
  relevant behavior changes.
- Update both roff manpages in `man/` **and** `tmux-copyrat.tmux` when changing
  a CLI flag, default, named pattern, key binding, or runtime control. The
  manpages document the behavior; `tmux-copyrat.tmux` implements the bindings,
  so a change to one without the other ships a feature that is inert from
  tmux. Lint the manpages with `make man`, preview them per `CONTRIBUTING.md`,
  and update `.TH` version and date for releases.
- Commit messages must follow `.convco` Conventional Commit rules. Use
  `make commits` to check them.

`Cargo.toml`, `Cargo.lock`, `deny.toml`, and the GitHub workflows define the
release and supply-chain constraints. Preserve `--locked` behavior in Cargo
commands that resolve dependencies.
