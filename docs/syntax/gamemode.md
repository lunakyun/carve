# Game modes

A game mode is plain text: `survival`, `creative`, `adventure` or
`spectator`. You can write them without quotes. Case does not matter.

## Read

```
send "%gamemode of player%"
send "%player's gamemode%"
send "%game mode of player%"
```

## Set

```
set gamemode of player to creative
set player's gamemode to adventure
set game mode of player to spectator mode
```

The mode can be a word, a quoted text, a number (`0` to `3`) or a short form
(`s`, `c`, `a`, `sp`). Anything else is a runtime error that names the
script line. In the plugin this uses the player API directly. A host without
that API falls back to the console `gamemode` command.

## Conditions

```
if gamemode of player is creative:
if player's gamemode is not survival:
if player is in creative:
if player is in adventure mode:
if player is not in survival:
if player isn't in survival mode:
```

## Not done yet

- `on gamemode change` event.
- Gamemode of an offline player.
