# Effects

An effect is one line that does something: send a message, heal a player,
change a variable. Effects go inside a trigger (`on join:`, a command's
`trigger:`) or inside an `if` block.

Each heading below is the exact pattern Carve matches. In a pattern:

- `%text%`, `%number%`, `%player%` and `%object%` are values. See
  [expressions](expressions.md) for what you can put there.
- `%variable%` must be a variable, like `{coins}` or `{_x}`.
- `[...]` is optional.
- `(a|b)` means one of the words.

Words are not case sensitive. `Send "hi"` works the same as `send "hi"`.

A `%text%` slot takes any value and prints it. A `%number%` slot needs a
number. A `%player%` slot needs a player. A variable fits any slot, and its
value is checked when the line runs.

When an effect needs a player who is offline, the effect is skipped. There is
no error.

## Messages

### `send %text% [to %player%]`

```
send "Hello!"
send "Welcome, %player%!" to player
```

Without `to`, the message goes to the player of the event, or to whoever ran
the command. When the console runs a command, the message goes to the
console. In a block break event with no player, it goes to the server log.

Color codes like `&a` and `&c` work in all text.

### `broadcast %text%`

```
broadcast "&e%player% joined the server"
```

Sends the text to every online player. It is also written to the server log.

### `send title %text% [with subtitle %text%] to %player%`

```
send title "Welcome!" to player
send title "Level up" with subtitle "You are now level 2" to player
```

Shows a big title in the middle of the screen.

### `send action bar %text% to %player%`

```
send action bar "Type /help for help" to player
```

Shows text just above the hotbar.

## Players

### `heal %player%`

```
heal player
```

Sets the player's health to their maximum health.

### `set health of %player% to %number%`

```
set health of player to 10
```

Health below 0 becomes 0. In the plugin, health above the player's maximum
health becomes the maximum. A full health bar is 20.

### `kill %player%`

```
kill player
```

### `teleport %player% to %number%, %number%, %number%`

```
teleport player to 0, 64, 0
teleport player to 100.5, 70, -20
```

Moves the player to x, y, z in the world they are in now. There is no way to
pick another world yet.

## Server

### `execute console command %text%`

```
execute console command "say %player% found a secret"
execute console command "/give %player% minecraft:bread 4"
```

Runs a command as the console. A leading `/` is removed. In the plugin the
command runs after the script finishes (or reaches a `wait`), not right away.
This keeps the command from calling back into Carve while a script is running.

### `cancel [the] event`

```
on chat:
    if message is "badword":
        cancel event
```

Stops the event from happening: the chat message is not sent, the block is
not broken, and so on. Notes:

- It works only before the first `wait`. After a `wait` the event is already
  over, so `cancel event` does nothing and logs a warning.
- Commands have no event, so `cancel event` in a command is a load error.

## Waiting

### `wait %number% (tick|ticks)`

```
wait 1 tick
wait 10 ticks
```

### `wait %number% (second|seconds)`

```
wait 3 seconds
wait 0.5 seconds
```

Pauses this run and continues it later. The server runs 20 ticks per second.
Notes:

- The wait is rounded to whole ticks. A wait of 0 ticks does nothing.
- A negative wait is a runtime error.
- Local variables and the event's values (`player`, `message`, `arg`) are
  kept across the wait.
- A player can go offline while the run waits. Effects on them are then
  skipped.
- If you reload while a run is waiting, it finishes on the old code.

## Variables

See [variables](../variables.md) for how names, lists and saving work.

### `set %variable% to %object%`

```
set {spawn-message} to "Welcome to the server"
set {_count} to 0
set {last-joined} to player
set {coins::%player%} to {coins::%player%} * 2
```

Setting a variable to a value that is not set (`<none>`) deletes it.

### `add %object% to %variable%`

```
add 1 to {joins::%player%}
add player to {team::*}
```

On a single variable this adds a number. A variable that is not set counts
as 0, and adding to a variable that holds text that is not a number is a
runtime error.

On a list (`{name::*}`) this appends the value at the next free number index,
like Skript. Any value works there.

### `remove %object% from %variable%`

```
remove 5 from {coins::%player%}
remove player from {team::*}
```

On a single variable this subtracts a number, with the same rules as `add`.
The result can go below 0. On a list it removes every item equal to the value
(text compares without case).

### `delete %variable%`

```
delete {coins::%player%}
delete {coins::*}
```

A name ending in `::*` deletes the whole list: every variable whose name
starts with the part before the `*`.

## Control flow

These lines change which line runs next. They are not patterns, so they are
not in the list above.

| Line | What it does |
|---|---|
| `stop` | Ends the trigger (or the function) right here. |
| `exit loop` / `stop loop` / `break` | Leaves the innermost loop. |
| `continue` | Skips to the next round of the innermost loop. |

Loops themselves (`loop ...:` and `while ...:`) are explained in
[loops](loops.md).

