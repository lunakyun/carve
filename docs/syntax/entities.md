# Entities

Entities are mobs, items, projectiles and everything else in the world that is not a player.
Players also work in most of these lines, because a player is an entity too.

Entity names are the vanilla ids with spaces: `zombie`, `iron golem`, `armor stand`.
Plurals work (`pigs`, `wolves`, `zombies`).
Unknown names are a script error, so typos show up at load time.

## Spawning

```
spawn a zombie at player
spawn 3 pigs
spawn 2 iron golems at location(10, 70, 10)
summon a creeper at {_spot}
spawn a zombie named "&cBob" at player
```

- `at <place>` is a location, a player or an entity.
  Without it, the mob spawns at the event's player.
  Triggers with no player (`on load`, `every`) need `at`.
- The count is cut to 100 per line.
- `named "x"` (before or after `at`) sets the name shown above each mob.
- You cannot spawn a player.
- `spawn lightning at ...`, explosions and particles are world effects, see [effects](effects.md).

## Last spawned entity

```
spawn a pig at player
set name of last spawned entity to "&aBob"
push last spawned entity upwards with speed 1
```

`last spawned entity` is the newest entity this run spawned.
It is `<none>` before the first spawn.

It needs the Pumpkin plugin. A server without the entity API only runs `/summon`,
which cannot say what it made, so `last spawned entity` stays `<none>` there.

## Finding entities

| Expression | Gives |
|---|---|
| `last spawned entity` | one entity |
| `nearest entity to player` | one entity, or `<none>` |
| `nearest zombie to {_loc} within 20 blocks` | one entity, or `<none>` |
| `all zombies in radius 10 of player` | a list |
| `all entities in radius 10 of player` | a list (players too) |
| `all players in radius 10 of {_loc}` | a list |
| `all pigs` | a list, from every world |
| `size of all pigs in radius 20 of player` | a number |
| `type of {_e}`, `entity type of {_e}` | text like `zombie` |
| `name of {_e}`, `{_e}'s name` | the custom name, or `<none>` |
| `health of {_e}` | a number |
| `location of {_e}`, `x of {_e}`, `y of {_e}`, `z of {_e}`, `world of {_e}` | as for players |
| `x velocity of {_e}` (also `y`, `z`) | a number |

`nearest` never gives back the thing you measured from.
The radius is in blocks. `radius`, `blocks`, `a` and `of` after `in` are optional words.

Lists work with `loop`. Inside, use `loop-value` or `loop-entity`:

```
loop all entities in radius 10 of player:
    if loop-entity is a zombie:
        kill loop-entity
```

## Conditions

```
if last spawned entity is a pig:
if {_e} is not a creeper:
if {_e} isn't an enderman:
if player is a player:
```

A player is of type `player`. An unset value is never of any type.

## Killing and removing

```
kill last spawned entity
kill all zombies in radius 10 of player
kill {_e}
despawn {_e}
delete last spawned entity
```

`kill` deals fatal damage, so drops and death events happen.
`despawn`, `remove` and `delete` take the entity away with no death.
`delete {x}` on a variable is not changed. Only entity values use the removal.
A player cannot be despawned. Use `kick` or `kill`.

## Moving

```
push player upwards with speed 2
push last spawned entity north at speed 0.5
push all pigs in radius 5 of player east
push player forward with speed 2
push player backwards
push last spawned entity away from player with speed 3
push all zombies in radius 10 of player towards player
teleport last spawned entity to location(0, 70, 0)
set y velocity of player to 1.5
set velocity of player to vector(0, 1, 0)
```

- Directions: `up`, `upwards`, `down`, `downwards`, `north` (-z), `south` (+z), `east` (+x), `west` (-x).
- `push` adds to the current velocity. The default speed is 1. Speed is cut to 20.
- `forward` is the way the player or entity looks (pitch included),
  `backward` the other way.
- `away from` / `towards` take a location, player or entity. Nothing happens
  when both are at the same spot.
- `set velocity of ... to <vector>` replaces the whole velocity. Each part is
  cut to -20 to 20.
- Pushing a player depends on Pumpkin sending the new velocity to the client.
  Check it in game.

## Naming

```
set name of last spawned entity to "&cAngry Zombie"
```

The name shows above the entity. `&` color codes work.
On a player, this sets the display name.

## What does not work

- Mob-specific data (profession, sheep color, ...), NBT on spawn.
- `all entities in chunk`, `entity from uuid`.
- On a server with only console commands, `last spawned entity`, `nearest`,
  `all ... in radius`, names and velocity do nothing. Spawn, kill and teleport still run
  as `/summon`, `/kill` and `/teleport`.
