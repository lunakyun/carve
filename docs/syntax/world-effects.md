# World effects

Lightning, explosions, particles, potion effects and fire. These lines take
free words (a particle name, a potion name), so Carve checks the name when the
script loads. A wrong name is a load error that says which name it did not know.

A place is a location, like `location(1, 64, 2)`, or a player. A player means
the spot where they stand.

Effects and conditions on a player skip silently when the player is offline.

## Lightning

```
strike lightning at player
strike lightning at location(0, 70, 0)
strike lightning effect at player
show a lightning effect at location(0, 70, 0)
```

`strike lightning` is a real bolt. With the word `effect` it is only the
flash and sound: no fire and no damage.

## Explosion

```
create explosion at player
create explosion of force 3 at player
create an explosion with power 2 with fire at location(0, 70, 0)
```

The default force is 3. The most you can ask for is 100. The explosion breaks
blocks like TNT does.

## Particles

```
play 5 flame particles at player
show 20 happy villager particles at location(1, 2, 3) with spread 0.5 with speed 0.1
show crit particles at player
```

- The count is optional and defaults to 1. The most is 1000.
- `with spread N` sets how far the particles scatter on each axis.
- `with speed N` sets how fast they move.
- You can write the verb as `play`, `show`, `draw`, `spawn`, `create` or `make`.
- The name is the Minecraft particle id with spaces: `happy villager`,
  `dragon breath`, `flame`.

## Potion effects

```
apply potion of speed 2 to player for 10 seconds
apply speed to player
apply fire resistance of tier 3 to player for 2 minutes
remove all potion effects from player
remove the speed potion effect from player
```

- The number after the name is the tier. `speed 2` is Speed II. The default
  tier is 1.
- The time can use `ticks`, `seconds`, `minutes` or `hours`. The default is 15
  seconds.
- Names are the Minecraft ids with spaces, like `jump boost` or `night vision`.
  `regen`, `jump`, `slow`, `healing` and `harming` also work.

Condition:

```
if player has potion effect speed:
if player doesn't have potion effect poison:
```

## Fire

```
ignite player for 5 seconds
set player on fire for 3 seconds
extinguish player
```

The default is 5 seconds.

## What the server needs

Carve uses the plugin API where it can. On any other host it falls back to
console commands:

| Effect | Fallback |
|---|---|
| Lightning | `/summon minecraft:lightning_bolt` |
| Particles | `/particle` |
| Apply and remove potion effects | `/effect give` and `/effect clear` |
| Explosion, fire, `has potion effect` | none: a warning is logged, or the check is false |
