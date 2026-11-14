# Senders

A sender is whoever started the trigger. It is either a player or the console.

## Expressions

| Syntax | Meaning |
|---|---|
| `sender`, `executor`, `command sender` | The player or the console that ran the trigger. |
| `console` | The server console. |
| `me`, `myself` | The player, same as `player`. |

They work in any trigger, but not inside functions. Pass them in as
parameters instead. `me` also needs a player, so it is not allowed in
`on load` or `every`.

A variable can hold the console: `set {who} to console`.

## Conditions

```
if sender is a player:
if sender is not a player:
if sender is console:
if sender is not the console:
```

## Effects

```
send "hello" to sender
send "hello" to console
make player execute command "/spawn"
make me execute command "home base"
make console execute command "say hi"
force player to execute command "spawn"
execute command "/spawn" by player
execute command "say hi"
execute console command "say hi"
```

- A leading `/` in the command is ignored.
- `execute command "x"` with no `by` runs as the console.
- `send ... to console` from a player's trigger writes to the server log. It
  does not reply to the player.
- Running as a player uses the real player as the command sender, so their
  permissions apply.
