# Changelog

All notable changes to DevKit are documented in this file.

## [0.2.1] - Unreleased

### Changed

- Unified kernel-driven formatting for JSON, JSONL, TOML, and plain text.
- JSON Parser and Content Diff now use the Rust kernel's resolved input type.
- Added persisted input-type recovery for JSONL and empty-input handling.
- Deferred inline text diff support to a later release.

## [0.2.0] - Released

### Added

- JSONL parsing with JSON Array output, JSONPath queries, and line-numbered errors.
- Automatic content detection for JSON, JSONL, TOML, YAML, Rust, JS/TS, Java, C/C++, Lua, and plain text.
- Stable structured-data formatting plus safe plain-text fallback.
- Application-internal, line-level text Diff with type overrides, highlighting, copy, save, and external-tool entry points.
- CI, tag-release, checksum, and Homebrew Tap update workflows.
