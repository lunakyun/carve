# Player effects (effects-2)

These effects act on a player. `player` can be any player expression, like `{_p}` or `attacker`. If the player is offline, the effect does nothing.

## Flight, damage and looks

```
make player fly
make player stop flying
make player invulnerable
make player vulnerable
make player invisible
make player visible
```

`make player fly` allows flight and starts flying. `stop flying` turns both off. Invisible uses the entity flag. If the server cannot set the flag, the default is the vanilla invisibility effect.

## Fire

```
ignite player
ignite player for 10 seconds
set player on fire for 3 seconds
extinguish player
put out player
```

`ignite` with no time burns for 5 seconds. The time must be a timespan.

## Knockback

```
knock player away from location(10, 64, 0)
knock player toward {_spot} with strength 2
```

The default strength is 0.4. It is kept between 0 and 20. Only the flat (x and z) direction counts. The game adds the small upward push itself.

## Looking and talking

```
make player look at location(0, 70, 0)
make player say "hello"
open book to player
open book in off hand to player
```

- `look at` turns the player's view toward the point. It uses the player's eye height.
- `say` runs the vanilla `say` command as the player.
- `open book` opens the written book in the main hand (or off hand). It does nothing if no book is held.

## Logging

```
log "something happened"
log "bought a rank" to file "shop"
```

Lines go to `logs/<name>` with a time stamp. With no file, the name is `server.log`. A file name may only use letters, digits, `-`, `_`, `.` and spaces. A name with a folder or `..` is an error.

## Warnings

```
locally suppress missing conjunction warnings
```

Accepted and ignored. Carve has no such warnings. It exists so scripts ported from Skript still load.
