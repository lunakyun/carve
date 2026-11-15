# Getting started

From nothing to a working script.

## What you need

- A [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) server with plugin
  support (plugin API v0.1).
- `carve.wasm`, from a release or built yourself (see below).

## Install

1. Copy `carve.wasm` into the server's `plugins/` folder.
2. Start the server. Pumpkin asks once to allow `fs.read.data` and
   `fs.write.data`. Carve needs them to read your scripts and save variables.
3. Carve creates `plugins/data/carve/scripts/`. Your scripts go there.

### Hosted servers

On a host without an interactive console, Pumpkin can't ask, and the log says
Carve was skipped because "Console readline is not available". Add this to
`pumpkin.toml` and restart:

```toml
[plugins.overrides.carve]
allowed_permissions = ["fs.read.data", "fs.write.data"]
```

Use dots exactly as shown. Pumpkin compares the text exactly; the
`fs:read:data` form in Pumpkin's config comment doesn't work.

The plugin is not signed. Pumpkin loads unsigned plugins by default
(`allow_unsigned = true`) and shows a warning.

## Your first script

Create `scripts/hello.carve`:

```
on join:
    send "Hello, %player%! Welcome to the server."
```

Run `/carve reload` (or `/cv reload`) in the console or in game. You need the
`carve:admin` permission, which ops (level 2) have by default. You should see:

```
[Carve] Reloading all scripts...
[Carve] Successfully reloaded all scripts. (3ms)
```

Rejoin the server, and the message shows up.

## How a script is built

```
on join:                              # a trigger: when this runs
    add 1 to {joins::%player%}        # an effect
    if {joins::%player%} is 1:        # a condition
        broadcast "%player% is new!"  # runs only if the condition is true
```

- A **trigger** has no indent and ends with `:`. See [triggers](syntax/triggers.md).
- An **effect** does something. See [effects](syntax/effects.md).
- A **condition** after `if` decides whether the indented lines run. See
  [conditions](syntax/conditions.md).
- **Values** like `player`, `"text"`, `5` and `{variables}` are
  [expressions](syntax/expressions.md).
- `#` starts a comment.

Indent with tabs or with spaces, but use one style per file.

## Your first command

Add this to `hello.carve`:

```
command /hello [<player>]:
    description: Say hello
    trigger:
        if arg is set:
            send "%player% says hello!" to arg
        else:
            send "Hello, %player%!"
```

New commands need a **server restart**, not just a reload. After the restart,
try `/hello` and `/hello <name>`. From then on, changes to the code under
`trigger:` work with `/carve reload`. See [commands](commands.md) for why.

## When something is wrong

If a script has a mistake, the reload tells you where:

```
[Carve] Line 2: (hello.carve)
    Can't understand this effect: 'sned "Hi"'
    Line: sned "Hi"
    Did you mean: send %text% [to %player%] | ...
[Carve] Encountered 1 error while reloading all scripts! (3ms)
```

Like Skript, the broken line is skipped and everything else loads. Fix it and
reload again. See [errors](errors.md) for more.

### Troubleshooting

| Symptom | Fix |
|---|---|
| The log says Carve was skipped, "Console readline is not available" | Allow the permissions in `pumpkin.toml` (see [hosted servers](#hosted-servers)). |
| `/carve` is an unknown command | Carve didn't load. Check the log for the reason above, or for "unsigned" if your server has `allow_unsigned = false`. |
| A new `command /x` doesn't work after `/carve reload` | Restart the server. Pumpkin only lets plugins add commands while they load. |
| Scripts aren't found | They must end in `.carve` and be in `plugins/data/carve/scripts/`. |

## Next steps

- Look at the scripts in [`examples/`](../examples) and copy what you need.
- Read about [variables](variables.md) to store things like coins or homes.
- Check the [known limits](limits.md) before you plan something big.
