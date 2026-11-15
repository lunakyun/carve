# Changelog

All notable changes to Carve are listed here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.3] - 2026-11-16

### Added

- Core Skript syntax that popular public scripts use (page: `syntax/popular-syntax`):
  - Command arguments: `<offline player>`, `<boolean>`, `<integer>` (refuses `2.5`),
    `<world>`, `<item>`, `<timespan>`, defaults like `<number=5>` and
    `[<player=%player%>]`, `arg 1` and `argument 1`, `command //name`, and the
    `permission message:` entry.
  - Effects: `message "x" [to ...]`, `send/message ... to all players`,
    `player command "x"`, `execute player command "x"`, `console command "x"`,
    `make player run command "x"`, `command "x" by console`, `clear {list::*}`,
    `teleport player at <location>`, `delete player's held item`,
    `set 1st line of (player's held item)'s lore to "x"`,
    `remove all <item> from the player`, `reload skript "name"`,
    `exit trigger`, `exit N sections`, `exit N loops`,
    `set slots 0, 1 and 2 of {_gui} to ...` and `set slot (integers between 0 and 8) of ...`.
  - Items: `of <enchantment> [level]`, `with all flags hidden`, `with lore` on a
    loop value, and the names `dirt block`, `endstone`, `wheat seed`,
    `bottle o' enchanting`, `redstone dust`, `gold <tool>`, `player's head`, `grass`, `bed`.
  - Conditions: `{x} was less than a minute ago`, `difference between a and b is less than 20 seconds`,
    `player doesn't have permission "x"`, `a and b are set`, `arg-1 not set`,
    `{list::*} do not contain x`, `x contains "a" or "b"`, `loop-value is a location`,
    `player is holding anything`, `world is "world"`, `victim is not a or b or c`,
    `{x} mod 3 = 0`, and a section header that is only a condition works like `if`.
  - Expressions: `random integer from 1 to 10`, `rounded x`, `player's x-coordinate`,
    `"world"'s time`, `loop-entity's type`, `name of current inventory of player`,
    `event-player`, `loop-block`/`loop-item`/`loop-offline player` and the other typed loop values.
  - Loops: `loop all offline players`, `loop all entities in player's world` (and
    `in world "x"`), `amount of entities in radius 5 of player`.
  - Events: `on damage of a or b`, `on rightclick on a or b or a bed`,
    `on walking on <block or list>`, `on food bar change`; `on tab complete of "/cmd"`
    loads with a warning and never runs.
  - `draw 100 cloud at ... with offset vector(...) with extra 0`.

- Events without a player: `event-world` and `world is "x"` in `on explode`,
  `on redstone change`, `on spawn` and the like, `event-block's world`,
  `the entity` / `creature` and `the entity's vehicle` in `on spawn`,
  `on rightclick on a creature` / `on a zombie or a villager` (new
  `entity click` event), `event-block's type`, and `set Nth line of the item's
  lore` in `on drop` (read with a warning; Pumpkin gives no item to change).

### Changed

- `console` works inside functions.
- `name of <player>` gives the name of an offline player too.
- The host trait has `offline_player`, `offline_players` and `clear_held_item`
  (all have defaults). `CommandOutcome::NoPermission` carries the
  `permission message:` text.

## [0.3.2] - 2026-11-15

### Added

- Syntax from real Skript servers: `play sound ... at <location> to all players`
  (also Bukkit sound names like `ENTITY_TNT_PRIMED`), `on place of <block>`,
  `event-block's location`, `damage cause is a or b or primed tnt`, `is greater
  than` and the other comparison words, `subtract`, `make console execute`,
  `set message to`, `set chat format to`, `replace all x in message with y`,
  `formatted`, `idle time of`, `player's offhand tool`, `x meters in front of`,
  `shoot <kind> from <entity>`, `last shot <kind>`, `push x in direction of y`,
  `set size of world border of <player>`, `force teleport`, `hunger meter`,
  `remove <effect> from <player>`, `random element out of a and b`,
  `set bossbar progress of <id> to`, `a`/`an` before an item, item kinds like
  `sword` and `armor`, colored item names like `light blue wool`, and
  `location(x, y, z, world, yaw, pitch)`.
- `on <event> with priority <p>` on every event.
- Lines that need another plugin or something Pumpkin can't do now load with a
  load-time warning instead of an error: `region at`, `player's prefix`,
  `explosive yield`, `on tab complete` and a command's `suggestions:`.
- An empty section (only a comment) is a warning, not an error.

### Changed

- Indentation works like Skript: each trigger, command or function can use its
  own width or tabs. Before, the first indented line set one step for the whole
  file, so a file mixing 2- and 4-space blocks failed to load.
- `/carve reload` and `/carve errors` print every error, not the first ten.
- A call of a function that does not exist now says `unknown function 'name'`.
- `on break of <block>:` (also `on break of stone or oak log:`) runs only for those blocks.
  `event-block` is now the broken block in every `on break`.
- Source files, tests and docs pages are named by topic. Old docs page URLs redirect to the new ones.

## [0.3.1] - 2026-11-15

### Fixed

- Scripts could freeze or crash the server. They are now stopped with an error
  that names the script and line (see "Limits that keep the server safe" in
  `docs/limits.md`):
  - A command that ran itself (`make player execute command "/same"`, or
    `execute console command` of a Carve command) nested until the plugin's
    stack ran out. Queued commands now nest 6 levels deep at most, a run may
    queue 256, and the events a run starts are capped at 64.
  - Text that doubled in a loop (`set {_t} to "%{_t}%%{_t}%"`) ran the server
    out of memory. Text built by interpolation, `replace`, `join` and
    `repeated` is limited to 1 MiB.
  - A regex over a long text (`"a" repeated 1000000 times matches "(a|aa)*b"`)
    overflowed the stack. Repeated groups are limited to 300 rounds, patterns
    to 2,000 characters.
  - Huge lists (`numbers from 1 to 1000000` with `join`, `sorted` or
    `shuffled`) held the server for seconds. Lists now take part of the
    instruction budget by size.
  - Loops that spawn entities, create explosions or run commands used 100,000
    instructions of real server work. Those effects now cost more of the
    budget, and a run slice that spends more than 200 ms in them stops.
  - A run that waits can no longer be started without limit: 5,000 waiting
    runs per script file, 20,000 in all, and a wait is at most a year.
  - Brackets, `-` signs or calls nested thousands deep, very long lines and
    sections nested hundreds deep overflowed the stack while the script
    loaded. They are refused with a load error.

## [0.3.0] - 2026-11-15

### Changed (infrastructure)

- Actions that ran vanilla commands as the console now use the plugin API, so
  ops no longer see `[Server: Played sound ...]` in chat and the console log has
  no command echo: `play sound`, `stop sound`, `kick`, `ban` (also `ban ... for
  <time>`, offline names the server already knows), `unban`, `op`, `deop`,
  `give` (with names, lore and enchantments, so colour codes are no longer
  left in item names), experience, `make ... ride`, `dismount`, `clear title`,
  `poison`, `cure`, `launch a firework` and the scoreboard and team effects.
  An action that can't be done (unknown sound, player not online, item that
  doesn't exist) logs a warning with the script file and line. Console
  commands remain where the plugin API v0.1 has no call (swing, the recipe
  book, `stop`, difficulty, weather length, entity NBT such as `baby` and
  `leash`, `drop`, `place tree`, `set spawn`); each has a comment saying which.
- Carve keeps variables, bounds and the players that joined before in one file,
  `carve.redb` (a redb database), instead of `variables.json`, `bounds.json`
  and `first_join.txt`. Only changed keys are written, in one transaction per
  write, and also on `/carve reload` and when the server stops. On the first
  start with the new file, the old files are imported and renamed to `.old`.
  See [variables](docs/variables.md#saving).

### Added (new events and event values)

- New events: `on jump`, `on player chunk enter`, `on step on <block>`,
  `on pressure plate`, `on experience spawn`, `on armor change`, `on spectate`,
  `on world border bounds change`, `on world border center change`,
  `on unload`, `on server stop`, `on entity break door`, and (not yet verified
  in-game) `on player pickup arrow`, `on riptide` and `on projectile collide`.
  See [the step and jump events page](docs/syntax/events-step-jump.md).
- Event values: `event-location` for leaves decay, lightning, pistons, portal
  create and redstone; `event-entity` for leash, unleash, love mode, combust and
  bucket catch; `event-items` for block drop and piglin barter; `event-item`
  and `past event-item` for tool change; `player` for bell ring.
- `set event-boolean to false` in `on can build check`.
- `on spawn change` also runs for player spawn points; `on shoot` also runs for
  players' arrows; `on zombie break door` only runs for zombies.

### Added (more effects, conditions and expressions)

- Entity effects: flight, fake blocks, interaction and text display flags, sound
  categories, shearing, taming, timed bans, riding any entity, swing and equip,
  goat horns, handedness, silence, scream, trees, zombify, allay clone, script
  switching, reset, remove all, vector rotation, world calls.
- Conditions (adult, baby, charged, climbing, conditional, holding,
  pressing key, plugin, feature, version and more) and game-data tables for
  edible, fuel, fire resistant and flammable.
- Block state and lists: honey level, brushing stage, block age, element ranges, dequeue,
  banner pattern items, persistent data keys.
- Parsing and loot: `parsed as` integer, boolean and player, human-readable timespans,
  loot values, item components, server max stack size.
- Protocol and rotation: portal cooldown, protocol version, vehicle, villager, whitelist,
  rotation maths, block entity reads.
- Functions and aliases: variadic `min`/`max`, round places, `world()` and `entity()` calls,
  `axisAngle`, `for each`, alias patterns, team death message visibility,
  server icon, auto reload for scripts with an auto reload line.
- New in-game areas for these topics; the parity checklist now counts
  about 70% of core Skript (about 90% of what Pumpkin allows).

### Fixed

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
- About 200 more entries: command `cooldown:`, `damage was caused
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

[Unreleased]: https://github.com/lunakyun/carve/compare/v0.3.3...HEAD
[0.3.3]: https://github.com/lunakyun/carve/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/lunakyun/carve/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/lunakyun/carve/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/lunakyun/carve/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/lunakyun/carve/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/lunakyun/carve/releases/tag/v0.1.0
