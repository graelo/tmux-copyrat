# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

- Capture region is now determined by keyswitch, not by `@copyrat-capture-region`
  option

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

[Unreleased]: https://github.com/graelo/tmux-copyrat/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/graelo/tmux-copyrat/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/graelo/tmux-copyrat/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/graelo/tmux-copyrat/releases/tag/v0.6.0
