# In-game testing

`tests/ingame/run.js` checks Carve on a real Pumpkin server. It starts a local
server, connects two test players (`Tester` is op, `Pleb` is not), runs every
case, restarts the server once to check that variables persist, and stops it.

## Why not mineflayer?

Pumpkin 0.2.2 only speaks Minecraft 26.3 (protocol 777). mineflayer and
minecraft-data stop at 26.1, and the login and configuration packets changed
between those versions, so mineflayer can't join. `tests/ingame/mc.js` is a
small client written from Pumpkin's own packet definitions. It speaks only the
packets these tests need and has no npm dependencies.

## Setup (once)

1. Build the plugin: `cargo build -p carve-wasm --release --target wasm32-wasip2`.
2. Download `pumpkin-X64-Windows.exe` (or the Linux build) from the
   [Pumpkin releases](https://github.com/Pumpkin-MC/Pumpkin/releases). It was
   tested with `0.2.2+26.3-26.60`. Put it in `.test/server/` (gitignored).
3. Run it once to create `pumpkin.toml`, stop it, then change these settings:

   | Section | Setting |
   |---|---|
   | `[networking.java]` | `address = "127.0.0.1:25565"`, `online_mode = false`, `encryption = false` |
   | `[networking.java.authentication]` | `enabled = false` |
   | `[networking.bedrock]` | `enabled = false` |
   | `[plugins]` | `ask_permission_confirmation = false`, `allowed_permissions = ["fs.read.data", "fs.write.data"]` |
   | `[commands]` | `use_tty = false` (the runner types console commands on stdin) |
   | `[telemetry]`, `[update_checker]` | `enabled = false` |

   Only ever use these settings on a local test server. They turn off login
   checks.

## Run

```
node tests/ingame/run.js
```

The runner copies `carve.wasm`, the examples and `tests/ingame/carve-test.carve`
into the server. It deletes `variables.json` first. It prints one line per
case and exits with code 1 if anything fails. The server log is saved to
`.test/server/last-run.log`. Set `CARVE_SERVER_DIR` or `PUMPKIN_EXE` to use
another folder or binary.

## Areas

An area is a small, self-contained test file in `tests/ingame/areas/`. Each
one starts its own server, so areas can run side by side and a broken area
doesn't take the others down.

```
node tests/ingame/areas/smoke.js
```

**Ports.** Area number `i` uses Java port `25570 + i` (smoke is 0, so
25570). The harness turns off Bedrock, query, management and LAN broadcast
and moves their addresses off the Java port, so several servers can run at
once.

**Server folders.** Each area gets `.test/areas/<name>/` (ignored by git
through `/.test/`). On first use the harness copies the Pumpkin binary (a
hard link where possible), `pumpkin.toml` and `data/*.json` from
`CARVE_TEMPLATE_DIR` (default `../pumpscript/.test/server`, the server from
the setup above). It never copies the world. On every start it rewrites the
ports, `ops.json`, `carve.wasm` and the scripts, and deletes `variables.json`
and the world unless you pass `keepVariables` or `keepWorld`. The server log
is saved to `.test/areas/<name>/last-run.log`.

A minimal area:

```js
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const t = area('items', { port: 25571 })
t.run(async () => {
  const srv = await startServer({
    dir: t.dir, port: t.port, ops: ['Tester'],
    scripts: { 'items.carve': ['command /gem:', '    trigger:', '        give 1 diamond to player'].join('\n') },
  })
  t.cleanup(() => srv.stop())
  const bot = await joinBot('Tester', { port: t.port })
  t.cleanup(() => bot.quit())

  const since = Date.now()
  bot.command('gem')
  const s = await bot.waitPacket('container_set_slot', (d) => d.item && d.item.name === 'diamond', 5000, since)
  t.check('/gem gives a diamond', !!s)
})
```

`t.run` counts a thrown error as a FAIL and always runs the cleanups, so the
server is stopped. It prints `PASS`/`FAIL`/`SKIP` lines and a summary and
exits with code 1 on any FAIL. Use `t.skip(name, reason)` for checks Pumpkin
can't support yet.

### `lib/server.js`

`startServer({ dir, port, wasm, scripts, ops, keepVariables, keepWorld })`
resolves once the log says "Server is now running". `scripts` is either
`{ 'name.carve': source }` or a list of `.carve` paths. `ops` names get
level 4 in `ops.json` (Pumpkin's offline UUID is the first 16 bytes of
`sha256(name)`). It returns `{ console(cmd), waitLog(re, ms, from), mark(),
lines, stop() }`. `stop()` sends `stop` and kills the process after 20 s.

### `lib/bot.js`

`joinBot(name, { port })` connects a `TestBot` (it extends the `mc.js`
client) and resolves on spawn. Every play packet is emitted as
`packet` `{ id, name, buffer, data }`, with names from Pumpkin's
`packets.json`. `bot.packets` keeps the decoded ones, and
`waitPacket(name, pred, ms, since)` and `waitText(parts, { kind, since })`
wait for one.

Decoded: chat (system, player, disguised), action bar, title, subtitle,
title times, clear title, boss bar (add, remove, progress, name, style,
flags), objectives, display slot, scores, reset score, teams, sound, entity
sound, stop sound, particles, block and section updates, entity spawn
(type name from `entities.json`), remove, motion, entity events, damage
events, hurt animation, health, experience, game events (game mode), player
info (name, game mode, listed, latency, display name), tab list, open,
content, slot and close of containers, equipment, held slot, explosions,
time, world border and disconnect. Text components come back as
`{ text, color, bold, click, hover, translate }`, with translations filled
in from `en_us_java.json`.

Not fully decoded: item stacks with component patches (custom name, lore,
enchantments) show their id and count, but the data that follows them is
skipped because it has no length prefix. A `container_set_content` with such
an item is marked `partial`. Explosions stop after the particle id.
Particle options (dust color, block type) are skipped.

Actions: `chat`, `command`, `attack(entityId)`, `interact(entityId)`,
`useItem()`, `useOn(x, y, z, face)`, `place(x, y, z)`, `breakBlock(x, y, z)`,
`look(yaw, pitch)`, `moveTo`, `walkTo` (small steps), `setHotbar(slot)`,
`swapHands()`, `sneak(on)`, `sprint(on)`, `clickSlot(slot, { button, mode })`,
`closeContainer()` and `respawn()`.

## What `run.js` covers

**v1:** join, quit, chat (cancel and `message`), break and place, titles,
subtitles, action bars, `wait`, `cancel event` after a wait, commands with
arguments, permissions (op allowed, non-op denied), health and heal,
teleport, console commands, global variables across `/carve reload` and
across a restart, a broken reload loading everything but the broken lines, the restart notice
for new commands, and the runaway guard.

**v2:**
- loops (`loop all players`, `loop N times`, list size)
- combined conditions
- functions (value and waiting statement calls) and options
- `on load` and `every`
- `give`, `has`, `tool of player`
- location, x/y/z and world of a player, teleport to a location
- `on damage` (from the console `damage` command), `on death` (`kill`) and
  `on respawn`
- right and left click with filters

**`/carve`:**
- every subcommand, including single-file reload with and without errors and
  disable/enable with the file rename
- `vars` set/get/list/delete, `run` (also with a typo) and `timings`
- the permission check for a non-op
- tab completion of subcommands, and that asking for argument completion
  doesn't crash the server

Not covered: `attacker` with a real hit. That needs the test client to attack
another player's entity, which it can't do yet.

## Client notes

- **Teleport confirms.** Pumpkin kicks a client that confirms a teleport
  after a newer one was already sent ("Wrong teleport id"). The test client
  waits 60 ms and confirms only the latest one.
- **After a respawn**, the client sends "player loaded" again. Without it,
  Pumpkin ignores right clicks.
