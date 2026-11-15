# Potion effects, particles, queues and more

## Potion effect values

```
set {_e} to potion effect of speed of tier 2 for 30 seconds
set {_e} to an ambient potion effect of regeneration 3 without particles for 10 seconds
set {_e} to infinite potion effect of night_vision
apply {_e} to player
```

In a potion effect value, write a name with several words with an underscore
(`night_vision`). The `apply` effect also takes spaces.

- The tier is `of tier N`, `tier N` or a plain number after the effect (1 is the lowest). Without one it is 1.
- Without `for`, the effect lasts 15 seconds, like `apply speed to player`.
- `ambient` and `without particles` are sent to the player when the effect is applied.
- A potion effect prints like `speed 2 for 30 seconds`. It is not saved in global variables.

Read or change its parts:

```
send "%potion tier of {_e}%"          # 2
send "%potion amplifier of {_e}%"     # 1 (tier - 1)
send "%potion duration of {_e}%"      # 30 seconds
set potion amplifier of {_e} to 3
set potion duration of {_e} to 1 minute
```

## Potion effects of a player

```
send "%potion effects of player%"
set {_all::*} to active potion effects of player
send "%speed potion effect of player%"
send "%tier of speed of player%"
```

Players only: the plugin API has no potion effects for other entities, and item values carry no potion contents.

## Potion effect categories

```
send "%potion category of speed%"     # beneficial
if potion effect type category of poison is harmful:
```

## Potion and skull items

```
give potion of speed to player
give splash potion of poison to player
give lingering potion of haste to player
give potion item of {_e} to player
give skull of player to player
give skull of "Notch" to player
```

- `potion of <effect>` uses the vanilla potion for that effect (`swiftness` for speed). Effects without one get a custom effect of 3 minutes.
- `potion item of <potion effect value>` keeps its tier, length, ambient and particles.
- These items are given with item components through the `give` command. Inside chest menus the components are not set yet.

## Particle values

```
set {_p} to flame particle with offset vector(1, 2, 3) with particle speed 0.25
set {_v} to end rod particle with a velocity of vector(0, 1, 0)
set particle count of {_p} to 7
set extra value of {_p} to 0.1
draw {_p} at player
play {_v} at location(0, 70, 0)
```

- `with offset` and `with distribution` set the random spread on each axis. `with velocity` sets the count to 0, so the vector is the direction and the speed scales it.
- Read the parts with `particle count of`, `particle offset of`, `particle distribution of`, `particle speed of` and `extra value of`.
- Particles that need data (dust colors, blocks, items) can't be sent by this server; drawing one logs a warning.

## Client weather

```
set client weather of player to rain
send "%client weather of player%"     # rain, clear or <none>
reset client weather of player
```

Only the player sees it. `weather of player` and `set weather of player` still mean the weather of the player's world.

## Blocks

```
send "%redstone power of block at {_l}%"
set charges of block at {_l} to 3
send "%charges of block at {_l}%/%max charges of block at {_l}%"
```

- Redstone power is read from the block state: redstone wire, weighted pressure plates, targets and daylight detectors give their power, redstone blocks 15, and levers, buttons, torches and repeaters 15 when on. Other blocks give 0, including blocks that are only powered by a neighbour.
- Charges are 0 to 4, on respawn anchors only.

## Boss bars and inventories

```
send "%bossbar key of bossbar "raid"%"     # raid, or <none> when there is no such bar
send "%progress of bossbar "raid"%"        # 0 to 100
send "%rows of {_gui}%, %amount of slots of {_gui}%"
send "%viewers of {_gui}%"
```

`holder of` a chest inventory made by a script is always `<none>`, as in Skript.

## Queues

```
set {_q} to a new queue of 1, 2 and 3
set {_q} to queue with {_list::*}
send "%start of {_q}% %end of {_q}%"
```

## Smaller expressions

```
send "%raw name of player's tool%"                 # minecraft:diamond_sword
send "%max enchantment level of sharpness%"        # 5
send "%minimum enchantment level of mending%"      # 1
send "%result of function "double" with arguments 21%"
send "%last caught runtime errors%"
```

- `result of` calls a script function by its name. Function values (`function named ...`) don't exist yet.
- In `on respawn`, `respawn reason` is `death` (Pumpkin fires the event only after a death) or `end_portal`.
- In `on kick`, `on-screen kick message` is the reason the kicked player sees. Changing it has no effect on Pumpkin.
- In `on entity shoot bow`, `projectile force` is the force of the shot. Pumpkin fires the event only for mobs.
