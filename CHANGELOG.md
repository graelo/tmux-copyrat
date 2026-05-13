# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0] - 2026-05-13

### Added

- Doc tests for the public API and unit tests for previously untested modules
- `cargo-nextest` as the test runner (CI and local)

### Changed

- Bump MSRV to 1.95
- Dual-license under MIT or Apache-2.0 (was MIT only)
- Release workflow now uploads standalone binaries alongside archives, attests
  build provenance per target, and aligns with the github-actions-playbook
  conventions; archives are tagged and shipped via subdir
- `tmux-copyrat.tmux` installer fetches the standalone binary instead of
  extracting the release archive
- Renovate manages dependency and GitHub Action updates with automerge for
  patch and minor PRs and a 3-day `minimumReleaseAge`

### Performance

- Compile static regexes once with `LazyLock`

## [0.8.6] - 2026-04-16

### Changed

- Bump to Rust edition 2024 and MSRV 1.88

### Security

- Harden GitHub Actions workflows: pin third-party actions to commit SHAs,
  scope per-job permissions, drop persisted git credentials, move shell
  interpolations into `env` to block template injection, tighten the release
  tag glob to semver, set short artifact retention, and add build provenance
  attestation
- Extract dependency audit (`cargo-deny`, `cargo-pants`) and CI security scans
  (zizmor, poutine) into reusable workflows, called on demand by `essentials`
  and unconditionally on a Tue/Fri schedule
- Switch the Homebrew bump and a new Renovate workflow to short-lived GitHub
  App tokens via dedicated `release` and `renovate` environments

## [0.8.5] - 2026-03-22

### Added

- Manpages for `tmux-copyrat` and `copyrat`

### Changed

- Rewrite README and INSTALLATION docs in a terser man-page style
- Improve CONFIGURATION overview and fix minor inconsistencies
- Bump common GitHub Actions (`actions/checkout`, `actions/cache`, etc.)

## [0.8.4] - 2026-03-19

### Changed

- Plugin script always runs the installer so it can self-update; the installer
  itself decides whether to skip when the binary is already at the right
  version

## [0.8.3] - 2026-03-19

### Added

- Pattern reference table, keyswitch conflict tip, and direct-binding examples
  in docs

### Changed

- Clarify TPM installation steps and verification
- Bump dependencies

### Fixed

- Installer now auto-updates the binary when the plugin version changes

## [0.8.2] - 2026-02-28

### Fixed

- Clipboard command parsing now handles multi-word commands on Linux (e.g.,
  `wl-copy --primary`)
- Release Linux binaries are statically linked against musl

## [0.8.1] - 2026-02-28

### Fixed

- `--separator` is only passed when `--multi-select` is enabled, and its value
  is quoted so separators containing whitespace work correctly

## [0.8.0] - 2026-02-28

### Added

- Multi-select mode (`--multi-select`) for selecting multiple spans before
  copying, with configurable `--separator`

### Changed

- Add pre-commit config for local checks
- Bump dependencies

## [0.7.0] - 2026-01-21

### Added

- Separate keyswitch for searching entire pane history (`prefix+T`) vs visible
  area (`prefix+t`)
- `@copyrat-keyswitch-history` option to customize the history keyswitch
  (default: `T`)
- `@copyrat-default-output` option to set default output destination (`tmux` or
  `clipboard`)
- `Ctrl-B` and `Ctrl-F` as alternatives to `PageUp`/`PageDown` for scrolling
- Debug logging for tmux options parsing (requires a log implementation)

### Changed

- Capture region is now determined by keyswitch, not by
    `@copyrat-capture-region` option

### Removed

- `@copyrat-capture-region` option (replaced by keyswitch-based behavior)

### Fixed

- Color options now correctly passed to binary
- Binary fallback in tmux-copyrat.tmux script

## [0.6.1] - 2024-12-15

### Fixed

- Binary installation and fallback handling

## [0.6.0] - 2024-11-20

### Added

- Initial scrollback history support
- Viewport scrolling with PageUp/PageDown

[Unreleased]: https://github.com/graelo/tmux-copyrat/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/graelo/tmux-copyrat/compare/v0.8.6...v0.9.0
[0.8.6]: https://github.com/graelo/tmux-copyrat/compare/v0.8.5...v0.8.6
[0.8.5]: https://github.com/graelo/tmux-copyrat/compare/v0.8.4...v0.8.5
[0.8.4]: https://github.com/graelo/tmux-copyrat/compare/v0.8.3...v0.8.4
[0.8.3]: https://github.com/graelo/tmux-copyrat/compare/v0.8.2...v0.8.3
[0.8.2]: https://github.com/graelo/tmux-copyrat/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/graelo/tmux-copyrat/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/graelo/tmux-copyrat/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/graelo/tmux-copyrat/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/graelo/tmux-copyrat/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/graelo/tmux-copyrat/releases/tag/v0.6.0
