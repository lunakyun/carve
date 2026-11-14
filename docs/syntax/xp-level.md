# Experience and levels

## Reading

| Expression | Meaning |
|---|---|
| `level of %player%`, `%player%'s level` | Whole levels. |
| `xp of %player%`, `exp of`, `experience of`, `total experience of` | Total experience points. |
| `level progress of %player%`, `%player%'s level progress` | Progress to the next level, from 0 to 1. |

```
send "You are level %level of player%"
if level of player >= 30:
    send "nice"
```

## Changing

Each form works with `level` or `levels`, and with `xp`, `exp` or `experience`.

```
set level of player to 10
set player's level to 10
set xp of player to 500
set level progress of player to 0.5

add 3 to level of player
add 3 levels to player
give player 3 levels
give 2 levels to player
remove 1 from player's level
remove 1 level from player

give player 10 xp
give 10 xp to player
add 10 exp points to player
remove 10 xp from player
```

Rules:

- Levels, points and progress never go below 0.
- Progress never goes above 1.
- Fractions in levels and points are rounded down.
- Add and remove read the current value first, then set the result.

## In the plugin

The plugin uses the Pumpkin player API for all of this. A host without that API
falls back to the `experience` console command. That fallback cannot read
values (they are `<none>`) and cannot change level progress.

`on level change` and `on experience change` events are not done yet.
