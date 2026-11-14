# Changelog

All notable changes to Carve are listed here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Loading works like Skript: a broken line is reported and skipped, a broken
  `if`/`loop`/`while` skips its block, a broken trigger header skips that
  trigger, and the rest of the file loads. `/carve reload` replaces the old
  version with the new one, errors and all (it no longer keeps the old
  version). Calls to a function that failed to load are errors at the call.
- Load and reload messages follow Skript: `Reloading x...`, errors as
  `Line 9: (x.carve)` / message / `Line: ...`, then `Successfully reloaded x.`
  or `Encountered 1 error while reloading x!`, with the time. Unknown names
  give `Can't find the script 'x'.` and match in any case. Server start logs
  `Loaded N scripts with a total of M structures in S seconds`.

### Added (v2)

- Loops: `loop N times`, `loop {list::*}`, `loop all players`, `loop a, b
  and c`, `while`.
  - Loop values: `loop-value[-N]`, `loop-player`, `loop-index`,
    `loop-number`, `loop-iteration`.
  - Leaving a loop: `exit loop`, `continue`, and `stop` for the whole trigger.
- Lists: `add`/`remove` items, `size of`, printed as `a, b and c`,
  `random element of`.
- Conditions:
  - `and`, `or`, comma lists and brackets.
  - `x is a or b`.
  - `contains`, `starts with` and `ends with`, without case.
  - `is between`, `chance of N%`.
  - A condition on its own line stops the trigger when false.
  - `doesn't`-style wording.
- Functions with typed parameters, defaults, `return` and recursion (64 deep),
  across all scripts and checked at load. Options with `{@name}`.
- Types:
  - Items (checked against Pumpkin's ids), locations and worlds.
  - Item effects and checks: `give`, `tool of`, `has`,
    `inventory contains`.
  - Locations: `location(...)`, `x/y/z of`, `world of`, `distance between`,
    teleport to a location.
  - `uuid of`, `health of`, possessive `'s`.
- Events: death, damage (`damage`, `attacker`, `victim`), respawn, right and
  left click with `with`/`on` filters and `clicked block`, `on load`,
  `every N seconds`.
- Expressions: `a ? b`, `floor`/`ceil`/`round`/`abs`/`sqrt`/`min`/`max`/`mod`,
  `random integer/number between`, `parsed as number`, `true`/`false`.
- Commands: `aliases:`, `usage:` and `executable by:`.
- Text can hold quotes inside `%...%`, as in `"%f("x")%"`.

- `/carve` admin command: `help`, `reload [all|<script>]`, `enable`/`disable`,
  `list`, `info`, `errors`, `vars list/get/set/delete`, `run`, `timings`.
  Permissions: `carve:admin`, plus `carve:reload`, `carve:enable`,
  `carve:vars` and `carve:run`. See [the /carve command](docs/admin.md).

### Fixed (v2)

- An option whose value ends with `:` was read as an empty block.


### Fixed

- `on quit`: `player` was `<none>` because Pumpkin removes the player before
  the event runs. The event's player now resolves until the first `wait`.

### Added

- In-game test runner (`tests/ingame/`) for a real Pumpkin server. See
  [in-game testing](docs/ingame-testing.md).

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
- Load errors with file, line and "did you mean" suggestions.
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

[Unreleased]: https://github.com/void1z2/carve/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/void1z2/carve/releases/tag/v0.1.0
