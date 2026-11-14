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

## What it covers

join, quit, chat (cancel and `message`), break and place, titles, subtitles,
action bars, `wait`, `cancel event` after a wait, commands with arguments,
permissions (op allowed, non-op denied), health and heal, teleport, console
commands, global variables across `/carve reload` and across a restart, a
broken reload keeping the old scripts, the restart notice for new commands,
and the runaway guard.
