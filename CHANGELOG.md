# Changelog

All notable changes to Carve are listed here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-11-14

### Added

- First release of Carve, a Skript-like scripting plugin for Pumpkin
  (plugin API v0.1).
- Triggers: `on join`, `on quit`, `on chat`, `on break`, `on place`, and
  script commands with `<text>`, `<number>` and `<player>` arguments,
  optional arguments, `permission:` and `description:`.
- Effects: `send`, `broadcast`, `send title`, `send action bar`, `heal`,
  `set health`, `kill`, `teleport`, `execute console command`,
  `cancel event`, `wait`, `set`, `add`, `remove`, `delete`.
- Conditions: `is set`, `is not set`, `is` / `=`, `is not` / `!=`, `>`, `<`,
  `>=`, `<=`, `has permission`, `does not have permission`, with `if`,
  `else if` and `else`.
- Expressions: `player`, `name of`, `message`, `arg` / `arg-N`, numbers, math,
  text templates with `%...%`, and variables.
- Global variables saved to `variables.json`, local `{_variables}`, and
  `{list::*}` delete.
- Load errors with file, line and "did you mean" suggestions. All-or-nothing
  `/carve reload`.
- Runaway guard: 100,000 instructions per run slice.
- Docs in `docs/`, example scripts in `examples/`, and a benchmark
  (`cargo run --release -p carve-core --example bench`).

### Performance

- Players are looked up at most once per run slice, and the plugin caches
  player handles per call. A join trigger with 20 effects went from 74 host
  calls to 21.
- Variable names without `%...%` are lowercased at load time. Constant math
  and constant text are worked out at load time.
- Faster pattern matching at load time.
- The release build uses `opt-level = 3`.

[Unreleased]: https://github.com/OWNER/carve/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/OWNER/carve/releases/tag/v0.1.0
