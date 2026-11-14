# Getting started

This guide takes you from nothing to your first working script.

## What you need

- A [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) server with plugin
  support (plugin API v0.1).
- `carve.wasm`, from a release or built yourself (see below).

## Install

1. Copy `carve.wasm` into the server's `plugins/` folder.
2. Start the server. Pumpkin asks once to allow `fs.read.data` and
   `fs.write.data`. Carve needs them to read your scripts and save variables.
3. Carve creates a `scripts/` folder in its data folder. Your scripts go
   there.

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
Reloaded 1 script(s), 1 trigger(s).
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
1 error(s), old scripts kept:
hello.carve:2: can't understand this effect: 'sned "Hi"' (did you mean: send %text% [to %player%] | ...)
```

Your old scripts keep running until the new ones load without errors. See
[errors](errors.md) for more.

## Next steps

- Look at the scripts in [`examples/`](../examples) and copy what you need.
- Read about [variables](variables.md) to store things like coins or homes.
- Check the [known limits](limits.md) before you plan something big.
