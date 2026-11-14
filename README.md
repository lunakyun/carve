# Carve

Simple, readable scripts for the Pumpkin Minecraft server.

[![CI](https://github.com/OWNER/carve/actions/workflows/ci.yml/badge.svg)](https://github.com/OWNER/carve/actions/workflows/ci.yml)
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

- **Events:** join, quit, chat, block break and block place, with `cancel event`.
- **Commands** with typed arguments (`<text>`, `<number>`, `<player>`),
  optional arguments, permissions and usage messages.
- **Effects** for messages, titles, action bars, broadcasts, health,
  teleporting and console commands.
- **Variables**, global and local, with `{list::%player%}` names. Globals are
  saved to `variables.json`.
- **`wait`** in ticks or seconds, without blocking the server.
- **Clear errors** with file, line and "did you mean" suggestions.
- **Safe reloads:** if a script has an error, the old scripts keep running.
- **Fast:** players are looked up once per run, and constants are worked out
  at load time. See [performance](docs/performance.md).

## Quick install

1. Download `carve.wasm` from a release, or build it (below).
2. Put it in your Pumpkin server's `plugins/` folder and start the server.
   Allow the `fs.read.data` and `fs.write.data` permissions when asked.
3. Put `*.carve` files in the plugin's `scripts/` folder.
4. Run `/carve reload` (`carve:admin`, ops have it by default).

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
  [conditions](docs/syntax/conditions.md), [expressions](docs/syntax/expressions.md)
- [Variables](docs/variables.md)
- [Commands](docs/commands.md)
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
No. Load errors skip the broken file (or, on reload, keep the old scripts).
Runtime errors stop only that run and are logged with the file and line.

## Roadmap

Planned, not available yet:

- loops over lists and numbers
- `and` / `or` in conditions, and text checks like "contains"
- more events (death, damage, respawn, interact)
- items, locations and worlds
- functions and options
- adding new commands on reload, once Pumpkin's API allows it

Ideas and votes are welcome in the issues.

## Contributing

Bug reports, ideas and pull requests are welcome. See
[CONTRIBUTING.md](CONTRIBUTING.md) for how to build, test and add new syntax.
Changes are listed in [CHANGELOG.md](CHANGELOG.md).

## License

Carve is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE),
at your option.
