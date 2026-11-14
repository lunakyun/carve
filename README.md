# Carve

Simple, readable scripts for the Pumpkin Minecraft server.

**[Documentation](https://void1z2.github.io/carve/)** · [Syntax reference](https://void1z2.github.io/carve/reference/)

[![CI](https://github.com/void1z2/carve/actions/workflows/ci.yml/badge.svg)](https://github.com/void1z2/carve/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Pumpkin plugin API v0.1](https://img.shields.io/badge/Pumpkin%20API-v0.1-orange.svg)](https://github.com/Pumpkin-MC/Pumpkin)

Carve is a scripting plugin for [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin),
in the spirit of Skript. You write what should happen in almost-plain English,
save the file, and reload. No Rust, no compiling.

```
on join:
    add 1 to {joins::%player%}
    if {joins::%player%} is 1:
        broadcast "&eSay hi to %player%, they are new!"
    else:
        send title "Welcome back!" to player

command /heal [<player>]:
    permission: heal
    trigger:
        heal player
```

## Features

- **Events:** join, quit, chat, break, place, death, damage (with `attacker`),
  respawn, right and left clicks (`on right click with blaze rod on barrel:`),
  `on load` and `every 5 seconds`. Most can be cancelled.
- **Commands** with typed arguments (`<text>`, `<number>`, `<player>`),
  optional arguments, permissions, aliases and usage messages.
- **Loops:** `loop N times`, `loop {list::*}`, `loop all players`, `while`,
  with `exit loop` and `continue`.
- **Conditions** with `and`, `or` and brackets, `is "a" or "b"`, text checks
  (`contains`, `starts with`), ranges and inventory checks.
- **Functions** with typed parameters, defaults and return values, callable
  from every script. **Options** with `{@name}`.
- **Types:** numbers, text, players, items (`2 diamonds`), locations and
  worlds, all saved in `variables.json`.
- **`wait`** in ticks or seconds, without blocking the server, also inside
  loops and functions.
- **Clear errors** with file, line and "did you mean" suggestions.
- **Skript-style loading:** a broken line or trigger is reported and skipped; the rest of the script still loads.
- **Fast:** players are looked up once per run, and constants are worked out
  at load time. See [performance](docs/performance.md).

## Quick install

1. Download `carve.wasm` from a release, or build it (below).
2. Put it in your Pumpkin server's `plugins/` folder and start the server.
   Allow the `fs.read.data` and `fs.write.data` permissions when asked.
   On a hosted server without an interactive console, allow them in
   `pumpkin.toml` instead (see below).
3. Put `*.carve` files in `plugins/data/carve/scripts/`.
4. Run `/carve reload` (ops can by default). `/carve help` lists everything
   else: enable/disable, info, errors, variables, run and timings.

On hosted servers (panels without a console you can type in), Pumpkin can't
ask and skips Carve with "Console readline is not available". Allow the
permissions in `pumpkin.toml`:

```toml
[plugins.overrides.carve]
allowed_permissions = ["fs.read.data", "fs.write.data"]
```

Write them with dots exactly like this. Pumpkin compares the text exactly,
and the `fs:read:data` form shown in Pumpkin's own config comment doesn't
match.

The full walkthrough is in [getting started](docs/getting-started.md).

### Build from source

```
rustup target add wasm32-wasip2
cargo build -p carve-wasm --release --target wasm32-wasip2
# -> target/wasm32-wasip2/release/carve.wasm
```

## Documentation

- [Getting started](docs/getting-started.md)
- Syntax: [triggers](docs/syntax/triggers.md), [effects](docs/syntax/effects.md),
  [conditions](docs/syntax/conditions.md), [expressions](docs/syntax/expressions.md),
  [loops](docs/syntax/loops.md), [functions and options](docs/syntax/functions.md),
  [types](docs/syntax/types.md)
- [Placeholders](docs/placeholders.md)
- [Variables](docs/variables.md)
- [Commands](docs/commands.md)
- [The /carve command](docs/admin.md)
- [Reading errors](docs/errors.md)
- [Known limits](docs/limits.md)
- [Architecture](docs/architecture.md)
- [Performance](docs/performance.md)
- [In-game testing](docs/ingame-testing.md)
- [Examples](examples/): economy, starter kit, chat filter, join counter, heal, welcome

## FAQ

**Is it Skript compatible?**
No. Carve looks a lot like Skript, but it is its own language with far fewer
features. Most Skript scripts will not load without changes.

**Why does my new command need a restart?**
Pumpkin lets a plugin register commands only while the plugin loads, and it
has no way to remove them. `/carve reload` updates the code of commands that
already exist. New commands show up after a restart. See
[commands](docs/commands.md#reloading-and-restarting).

**My permission doesn't work, even for ops.**
Pumpkin denies permission nodes that were never registered. Carve registers
the `carve:` nodes used in commands when it loads, so a node added later needs
a restart. See [permissions](docs/commands.md#permissions).

**Does a broken script crash the server?**
No. Load errors skip only the broken line, section or trigger, like Skript.
Runtime errors stop only that run and are logged with the file and line.

## Roadmap

Planned, not available yet:

- dates and time spans (`now`, `time since`, `wait 1 minute` is fine, but
  there are no date values yet)
- sounds, game modes, blocks in the world (`set block at`), item names and lore
- inventory and GUI events
- adding new commands on reload, once Pumpkin's API allows it

The full list of every core Skript feature and where Carve stands is in the
[Skript parity checklist](docs/skript-parity.md).

Ideas and votes are welcome in the issues.

## Contributing

Bug reports, ideas and pull requests are welcome. See
[CONTRIBUTING.md](CONTRIBUTING.md) for how to build, test and add new syntax.
Changes are listed in [CHANGELOG.md](CHANGELOG.md).

## License

Carve is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE),
at your option.
