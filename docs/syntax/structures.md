# Structures

Structures are the top-level parts of a script file, plus a few helpers inside
triggers: `variables:`, `aliases:`, inline conditions,
`catch runtime errors:`, `parse if:`, `else if`, `local function` and `using`.

## variables:

`variables:` gives variables a default value. A default is set only when the
variable has no value yet, so saved values are never overwritten.

```
variables:
    {motd} = "Welcome!"
    {max-homes} = 3
    {coins::%player%} = 100
```

- Each line is `{name} = value`. The value can be any expression.
- Only global variables can have defaults. `{_name}` is an error.
- A name that holds the player (`%player%`) is a per-player default. It is set
  when the player joins, and for players who are online when scripts load.
- Other defaults are set once, before the first event after scripts load or
  reload.
- `{@option}` values work in names and values.
- A section can appear more than once, in any file.

## aliases:

`aliases:` gives a short name to any text. The name is replaced before the
script is read, but never inside quotes or `{variables}`.

```
aliases:
    magic wand = stick
    gem = diamond

on right click with magic wand:
    if player has gem:
        send "You are rich."
```

- The format is `name = replacement`. Names ignore case.
- Whole words only: `gem` does not change `gems`, but every name also works
  in the plural (`gems`) unless you define that name yourself.
- Longer names win, so `iron sword` beats `iron`.
- Names can hold variants: `(fast|quick) blade` makes `fast blade` and
  `quick blade`, and `the [big] key` makes `the big key` and `the key`.
- A name can't hold `:`, `%`, `{` or quotes.

## Inline conditions

Add `if` or `unless` to the end of a line to run it only when a condition
holds. It works for effects, `stop`, `exit loop`, `continue` and `return`.

```
on join:
    send "Welcome back!" if {joined::%player%} is set
    send "Say hi to the staff." unless player is op
    stop if {maintenance} is true
    do send "Hello" to player if player is op
```

- `do` in front of the effect is optional.
- Conditions can use `and` / `or`, like in `if` sections.
- The `if` is found outside text and `{variables}`, so
  `send "go if you can"` is not split.
- An error in the condition or in the effect is reported with the line number.

## catch runtime errors:

Runs a block. If a runtime error happens inside it (division by zero, a wrong
type, a failed function call), the block stops, the error is not logged, and the
script goes on after the block.

```
on join:
    catch runtime errors:
        send "%{share} / {people}%"
        send "never reached when people is 0"
    if last caught runtime error is set:
        send "Math failed: %last caught runtime error%" to console
    send "Still running."
```

- The message is in `last caught runtime error`. These short forms work too:
  `last caught error`, `last runtime error`, `caught error`, `runtime error`.
  It is also the local variable `{_runtime error}`.
- The message is `<none>` when nothing went wrong. It is cleared when the
  section starts.
- Errors inside functions called from the block are caught too. The function
  stops where the error happened.
- Sections can be nested. The innermost one takes the error.
- `wait` inside a section is fine.
- `exit loop`, `continue`, `return` and `stop` leave the section normally.
- Not caught: errors that stop a script before it runs (load errors) and the
  "ran more than N instructions" limit.
- Also written `catch runtime error:`, `catch errors:` or `catch error:`.

## parse if:

`parse if` keeps or drops a block while the file loads. The condition may only
use fixed values: text, numbers and `true` / `false`. This is meant for
options.

```
options:
    mode: debug

on join:
    parse if "{@mode}" is "debug":
        send "debug on"
    parse if "{@mode}" is not "debug" and 1 = 1:
        send "debug off"
```

- Supported: `is`, `is not`, `=`, `!=`, `>`, `<`, `>=`, `<=`, `and`, `or`,
  `not` and brackets.
- A dropped block is not checked at all, so it may hold syntax you don't want to
  load.
- `parse if` can't test variables or players. Use a normal `if` for that.

## else if

`if`, `else if` and `else` chains work as in Skript, and can be nested.

```
if {n} is 1:
    send "one"
else if {n} is 2:
    send "two"
else:
    send "many"
```

An `else` or `else if` without an `if` before it is a load error.

## local function

`local function` works like `function`, but only the file that holds it can call
it. Two files can have local functions with the same name. A local function
wins over a global one with the same name in its own file.

```
local function greeting(p: player) :: text:
    return "Hello %{_p}%"

on join:
    send greeting(player)
```

A name can't be used for two functions in one file. Two global functions with
the same name still clash across files.

## using

`using <feature>` lines are accepted and ignored. Carve has no experimental
features, but scripts written for Skript still load.
