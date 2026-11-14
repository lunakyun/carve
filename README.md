# Carve

A small scripting language for the [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) Minecraft server, in the spirit of Skript. It looks like Skript but it is not compatible with it.

```
on join:
    send title "Welcome, %player%!" to player
    add 1 to {joins::%player%}

command /heal [<player>]:
    permission: heal
    trigger:
        if arg is set:
            heal arg
        else:
            heal player
```

## Install

1. Build it (see below) or take `carve.wasm` from a release.
2. Copy `carve.wasm` into the Pumpkin server's `plugins/` folder.
3. Start the server. It asks once to allow the `fs.read.data` and `fs.write.data` permissions. Carve needs them to read scripts and save variables.
4. Put your `*.carve` files in the plugin's data folder under `scripts/`. Pumpkin creates that folder next to the plugin.
5. Run `/carve reload` (or `/cv reload`) after editing scripts. You need the `carve:admin` permission, which op level 2 has by default.

The plugin is not signed. Pumpkin loads unsigned plugins by default (`allow_unsigned = true`) and shows a warning.

## Build

```
rustup target add wasm32-wasip2
cargo build -p carve-wasm --release --target wasm32-wasip2
# -> target/wasm32-wasip2/release/carve.wasm
cargo test -p carve-core
```

## Files

- `scripts/*.carve`: your scripts. Indent with tabs or spaces, but use one style per file. `#` starts a comment.
- `variables.json`: global variables. It is saved about 5 seconds after a change and again when the server stops.

## Syntax

### Triggers

| Trigger | Notes |
|---|---|
| `on join:` | also `on login:` |
| `on quit:` | also `on leave:`, `on disconnect:` |
| `on chat:` | `message` is the chat text |
| `on break:` | also `on mine:`, `on block break:` |
| `on place:` | also `on block place:` |
| `command /name <text> [<number>]:` | Arguments can be `<text>`, `<number>` or `<player>`. `[...]` makes an argument optional, and optional arguments must come last. The last text argument takes the rest of the line. The command has the entries `permission: node`, `description: text` and `trigger:` (the code). |

### Effects

```
send %text% [to %player%]          # no "to": goes to the player or command sender
broadcast %text%
send title %text% [with subtitle %text%] to %player%
send action bar %text% to %player%
heal %player%
set health of %player% to %number%
kill %player%
teleport %player% to %number%, %number%, %number%
execute console command %text%
cancel [the] event
wait %number% (tick|ticks|second|seconds)
set {var} to %object%
add %number% to {var}
remove %number% from {var}
delete {var}                       # {list::*} deletes the whole list
```

### Conditions

```
if %object% (is|=) %object%:
else if %object% (is not|!=) %object%:
else:
%number% > %number%   (also <, >=, <=)
%player% has permission %text%
%player% does not have permission %text%
%object% is set / %object% is not set
```

Text comparison ignores case. A player also equals text that matches their name.

### Expressions

- `player`, `name of %player%`
- `message` (only in `on chat`)
- `arg` / `arg-1`, `arg-2`, ... (commands)
- numbers `5`, `-2.5`, math `+ - * /` with brackets
- text `"hi %player%, you have %{coins::%player%}% coins"`. `%expr%` inserts a value, `%%` is a literal `%` and `""` is a quote. `&a`-style color codes work.
- variables: `{global}`, `{_local}` (lives for one run), `{list::%player%}`

### Errors

Load errors show `file:line` and the three closest patterns, for example:

```
bad.carve:3: can't understand this effect: 'sned "hi" to player' (did you mean: send %text% [to %player%] | ...)
```

## How it works

- `crates/carve-core` is pure Rust and has no Pumpkin dependency. Each trigger compiles to a flat instruction list with jumps. The game is reached only through the `Host` trait.
- `crates/carve-wasm` is the Pumpkin plugin, built on `pumpkin-plugin-api` (WIT v0.1, pinned to Pumpkin commit `058f7ec`). It registers one handler per event type and dispatches to the current script table.
- `wait` saves the run state (position, locals, event data) and continues it from the plugin's tick task. After a run's first `wait`, `cancel event` does nothing and logs a warning, because the event already happened.
- Players are stored as handles (UUID). If the player is offline when the handle is used, it resolves to `<none>` and the effect is skipped.
- Each run slice is limited to 100,000 instructions. A run that hits the limit stops with an error.
- If a reload has an error, all the old scripts stay active and the errors go to the sender. Runs that are waiting finish on the code they started with.

## Known limits

- **New commands need a restart.** Pumpkin's plugin API registers commands only while the plugin loads, and it can't unregister them. A reload updates the code of existing script commands. A new command needs a restart, and a removed command answers "no longer in any script" until the next restart. Permission nodes for script commands are also registered only at load.
- Permission nodes without a namespace are put in the plugin's namespace (`heal` becomes `carve:heal`). Pumpkin denies any node that was never registered, even for ops, unless it is granted to the player directly. A node from another plugin (`other:x`) works only if that plugin registered it.
- Five events in v1: join, quit, chat, break, place. A block break without a player has no `player`.
- If the server calls back into Carve while a script is running (a nested event), that event is skipped with a warning. Console commands from scripts run after the script finishes, which avoids this problem.
- When a console command run reaches a `wait`, messages sent to the console after the wait go to the server log.
- No loops, functions, options, item, location or world types yet. `teleport` uses the player's current world.
- Variable names are lowercased. A player in a variable name is written as the player's name, not UUID.
- If `variables.json` is broken, the plugin moves it to `variables.json.broken` and starts with no variables.
