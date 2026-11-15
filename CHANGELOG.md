# Changelog

All notable changes to Carve are listed here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (Skript parity, wave 6 chunk 1)

- New events: `on jump`, `on player chunk enter`, `on step on <block>`,
  `on pressure plate`, `on experience spawn`, `on armor change`, `on spectate`,
  `on world border bounds change`, `on world border center change`,
  `on unload`, `on server stop`, `on entity break door`, and (not yet verified
  in-game) `on player pickup arrow`, `on riptide` and `on projectile collide`.
  See [the wave 6 page](docs/syntax/w6-chunk1.md).
- Event values: `event-location` for leaves decay, lightning, pistons, portal
  create and redstone; `event-entity` for leash, unleash, love mode, combust and
  bucket catch; `event-items` for block drop and piglin barter; `event-item`
  and `past event-item` for tool change; `player` for bell ring.
- `set event-boolean to false` in `on can build check`.
- `on spawn change` also runs for player spawn points; `on shoot` also runs for
  players' arrows; `on zombie break door` only runs for zombies.

### Added (Skript parity, wave 6 chunks 2 to 7)

- Chunk 2: flight, fake blocks, interaction and text display flags, sound
  categories, shearing, taming, timed bans, riding any entity, swing and equip,
  goat horns, handedness, silence, scream, trees, zombify, allay clone, script
  switching, reset, remove all, vector rotation, world calls.
- Chunk 3: new conditions (adult, baby, charged, climbing, conditional, holding,
  pressing key, plugin, feature, version and more) and game-data tables for
  edible, fuel, fire resistant and flammable.
- Chunk 4: honey level, brushing stage, block age, element ranges, dequeue,
  banner pattern items, persistent data keys.
- Chunk 5: `parsed as` integer, boolean and player, human-readable timespans,
  loot values, item components, server max stack size.
- Chunk 6: portal cooldown, protocol version, vehicle, villager, whitelist,
  rotation maths, block entity reads.
- Chunk 7: variadic `min`/`max`, round places, `world()` and `entity()` calls,
  `axisAngle`, `for each`, alias patterns, team death message visibility,
  server icon, auto reload for scripts with an auto reload line.
- In-game areas `w6-chunk1` to `w6-chunk7`; the parity checklist now counts
  about 70% of core Skript (about 90% of what Pumpkin allows).

### Fixed (wave 6)

- A `N times` read only applies where a list can start.

### Fixed

- Several events were marked blocked in the parity checklist for reasons that
  turned out to be wrong; each was re-checked against the plugin API.

## [0.2.0] - 2026-11-15

### Changed

- Faster: `add x to {list::*}` no longer slows down as the list grows (10,000
  adds went from 4.7 s to 4 ms), function calls are about 30% cheaper, and
  comparing a loop value with a number takes a fast path. In a side-by-side
  benchmark against Skript on Paper (100,000 runs of each case), Carve is
  faster on all seven cases, from about 1.3x (text) to 7x (global variables).
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

### Added (Skript parity)

- `on damage` runs for mobs too, with `on damage of <type>` filters.
  `victim` and `attacker` can be any entity (`attacker is a zombie`,
  `type of attacker`). `set damage to`, `add/remove ... damage` and
  `set death message to` change the hit and the death message on the server.
  `on death` has an `attacker`: the last hit within 5 seconds. Hits of
  fireballs, thorns, tridents and wither skulls now run `on damage`.
- About 120 more events: inventories, items, blocks, movement, vehicles,
  portals, pistons, weather, worlds, chunks, server list ping, `at 18:00`
  triggers and more, each wired to Pumpkin where its API fires them.
- Time: dates, `now`, time spans with math, `time since`, `ago`/`later`, and
  `wait` with any span.
- Text and lists: case, substrings, join/split, hashes, sorted/reversed/
  shuffled lists, `where` filters, sum/min/max, list functions.
- World: blocks, entities (spawn, kill, push, nearest), particles, potion
  effects, explosions, lightning, weather, time, game rules, difficulty,
  world borders, worlds and chunks.
- Players: game modes, op/kick/ban, tab list names, XP and levels, health
  and food, flight, visibility, titles, boss bars, scoreboards, chest menus,
  slots, item names, lore and enchantments.
- Many more conditions and expressions, built-in math and text functions,
  `filter` and spawn sections, variables defaults, aliases, inline `if`,
  `parse if` and local functions.
- Types: named-value types, colors, vectors, quaternions, time periods,
  chunks, queues and world borders.
- SkBee-style syntax: bounds with enter/exit events, text components, NBT,
  particles and display entities, recipes, advancements, statistics,
  scoreboard teams, ray traces, Minecraft tags and utilities.
- Placeholders for other plugins.
- About 200 more entries in wave 5: command `cooldown:`, `damage was caused
  by`, `blocks in radius`, book pages, tamed and saddled checks, `all
  operators` and more. 645 core Skript entries now fully work and 223 work
  with a noted limit, all checked on a real server (1035 in-game checks).
- Every registry event, effect and condition now has a description in the
  generated syntax reference.

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

[Unreleased]: https://github.com/lunakyun/carve/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/lunakyun/carve/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/lunakyun/carve/releases/tag/v0.1.0
