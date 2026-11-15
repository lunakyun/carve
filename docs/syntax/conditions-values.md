# More conditions

Every condition here also has a negated form (`is not`, `isn't`).

## Dates

```
if {deadline} is in the past:
if {deadline} is in the future:
```

## Numbers

```
if 10 is evenly divisible by 5:
if {n} isn't divisible by 3:
```

Dividing by zero is never true.

## Empty values

```
if {list::*} is empty:
if "" is empty:
```

An unset variable counts as empty.

## Players

```
if player is online:          # also: "Steve" is online / is offline
if player is alive:           # also: is dead
if player is in world "world_nether":
if player is burning:         # also: is on fire
if player is invisible:
if player is invulnerable:
if player is gliding:
if player is frozen:          # fully frozen (140 freeze ticks)
if player's custom name is visible:
if player has a resource pack:
```

`has a resource pack` is true after the client reports the pack as loaded.

## Items

```
if "bread" is edible:
if "coal" is fuel:
if "netherite_sword" is fire resistant:
if "oak_planks" is flammable:
if "diamond" is of type "diamond":
if player is of type "player":
```

Edible, fuel, fire resistant and flammable use fixed lists of vanilla ids.
An id may have a `minecraft:` prefix, and spaces count as underscores.

## Script commands

```
if command "hello" is a script command:
```

True when a loaded script defines the command or one of its aliases.
A leading `/` is ignored.
