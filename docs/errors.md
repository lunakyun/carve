# Reading errors

Carve finds most mistakes when it loads your scripts, before anything runs.
Some can only be found while a script runs.

## Load errors

Load errors look like this:

```
welcome.carve:3: can't understand this effect: 'sned "hi" to player' (did you mean: send %text% [to %player%] | set %variable% to %object% | add %number% to %variable%)
```

Read it in parts:

| Part | Meaning |
|---|---|
| `welcome.carve:3` | the file and the line number |
| `can't understand this effect: '...'` | what went wrong, and the line as Carve saw it |
| `(did you mean: ...)` | the three patterns closest to your line |

Carve reports every error in a file, not only the first one.

At server start, files with errors are skipped and the rest load. With
`/carve reload` it is all or nothing: if any file has an error, the old scripts
stay active. The errors go to the server log, and the first 10 are also sent
to whoever ran the reload.

## Did you mean

When a line matches no pattern, Carve compares it with every pattern and shows
the three closest ones. Values in your line (text, numbers, variables) are
ignored for this, so only the words count. A typo like `sned` or `broadcats`
usually puts the right pattern first.

The patterns use the same notation as the [effects](syntax/effects.md) and
[conditions](syntax/conditions.md) pages, so you can look them up there.

## Common load errors

**A value has the wrong type.** The words matched, but a value doesn't fit:

```
demo.carve:2: can't understand this effect: 'kill "Steve"' (expected a player here but got a text) (did you mean: kill %player% | heal %player% | delete %variable%)
```

`"Steve"` is text, not a player. Use `player`, a `<player>` argument, or a
variable.

**Broken text.**

```
demo.carve:2: can't understand this effect: 'send "100%"' (unclosed '%' in text (use %% for a percent sign)) (did you mean: ...)
```

A single `%` starts an inserted value. Write `%%` for a percent sign. A
missing closing `"` gives `unclosed text: missing closing "`.

**A value that doesn't exist here.**

- `'message' only exists in 'on chat'` comes up when `message` is used in
  another trigger.
- `send arg-2` in a command with one argument gives `can't understand this
  effect`. The command has no second argument.

**Indentation.**

```
demo.carve:3: unexpected indentation (the line above does not end with ':')
demo.carve:1: empty block: nothing is indented under this line
```

A file must use tabs or spaces, not both. The first indented line sets the
step: if it uses 4 spaces, every level is 4 more spaces. Mixing gives errors
like `this file is indented with spaces, but this line uses tabs`.

**Triggers and commands.**

```
demo.carve:1: unknown event 'jump', known: join, quit, chat, break, place
demo.carve:1: expected a trigger like 'on join:' or 'command /name:', got 'send "hi"'
demo.carve:1: command has no 'trigger:' section
demo.carve:1: unknown argument '<thing>', use <text>, <number>, <player> or [<text>] for optional
demo.carve:3: commands have no event to cancel
```

## Runtime errors

Runtime errors happen while a script runs. They go to the server log with the
file and line, and that run stops. Other runs and other triggers keep going.

```
demo.carve:2: expected a number, got <none> (<none>)
demo.carve:3: division by zero
demo.carve:2: can't wait -1
demo.carve:3: can't add to a variable holding text (abc)
```

The first one is the most common: math on a variable that was never set. Set
it first, or check it with `if {x} is set:`.

### The runaway guard

```
demo.carve:4: script stopped: ran more than 100000 instructions without a wait
```

A run can do at most 100,000 steps before it reaches a `wait`. Each `wait`
starts a fresh count. Carve has no loops yet, so normal scripts never get
close. The guard is there so a broken script can't freeze the server.

## Warnings

```
demo.carve:3: 'cancel event' after a wait does nothing (the event already happened)
```

The script keeps running. Move `cancel event` above the first `wait`.
