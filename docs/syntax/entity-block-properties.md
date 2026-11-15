# Scale, targets, displays, block data, borders and more

## Entities

```
set scale of player to 2
add 0.5 to scale of last spawned entity
reset scale of player
set scale of {_display} to vector(2, 1, 2)

set target of {_zombie} to player
delete target of {_zombie}
send "you look at %target of player%"

set spectator target of player to {_pig}
reset spectator target of player
```

- `scale of` a living entity or player is its `scale` attribute (0.0625 to 16). Of a display entity it is the
  transformation scale, a vector; setting a number scales all three axes.
- `target of` a mob is what it attacks. `target of` a player is the entity they look at (up to 100 blocks). Only
  mobs can be given a target. `target` alone means the target of `player`; in `on target` it is the event's target.
  `targeted entity of player` and `player's target` work too.
- `spectator target of player` is the entity whose view the player sees. Setting it works in any game mode;
  `reset` gives the player their own view back. It reads as nothing when the player sees through their own eyes.

## Text displays

```
spawn text display at player with text "Shop"
set {_d} to last spawned entity
set text alignment of {_d} to left
set line width of {_d} to 80
remove 50 from text opacity of {_d}
reset line width of {_d}
```

`spawn text display`, `spawn block display` and `spawn item display` now set `last spawned entity`.
Alignment is `left`, `center` or `right`. Line width is in pixels (default 200), opacity is 0 to 255 (default 255).
`display opacity` and `opacity of` are other names for the text opacity.

## Blocks

```
send "%sea pickles of targeted block of player%"
add 1 to sea pickles of targeted block of player
send "%block data of block at player%"          # oak_log[axis=y]
set block data of {_b} to "sea_pickle[pickles=3,waterlogged=false]"
send "%temperature of block below player%"
```

- `[min|max] [sea] pickles of <block>` (also `pickle count`, `pickle amount`): the pickle count of a sea pickle
  block, 1 to 4. Changes are clamped; other blocks read as nothing and are left alone.
- `block data of <block>` is a text: the block id and its state properties. Setting it to another block id places
  that block first; properties not named keep their current (or default) value. A wrong property or value is an
  error that names the allowed ones. The property table is the vanilla 26.3 one.
- `temperature of <block>` is the biome's base temperature, lowered above y 80 the way vanilla does it, without
  vanilla's small per-column noise.

## World border

```
send "%world border%"                      # world border of 'world'
set {_wb} to world border of player
set world border warning distance to 10
add 30 seconds to world border warning time of {_wb}
reset world border warning time            # 15 seconds
```

`world border [of <world|player|location>]` and `<x>'s world border` give the border value. Warning distance is in
blocks, warning time is a timespan (whole seconds). `reset` goes back to the vanilla 5 blocks and 15 seconds.

## Boss bars and inventories

```
send "%title of bossbar "raid"% %style of bossbar "raid"%"
set {_who::*} to viewers of bossbar "raid"
add player to viewers of bossbar "raid"

set {_i} to chest inventory with 3 rows named "Shop"
set title of {_i} to "Bank"
add player to viewers of {_i}
send "%viewers of {_i}%"
```

- Boss bar `title`, `style` and `viewers` read the live bar; change the title and style with
  `set title of bossbar ...` and `set style of bossbar ...` from [scoreboards and boss bars](scoreboards-bossbars.md).
- The title of an inventory value can be changed while it is in a variable; it shows the next time it is opened.
  `add <player> to viewers of {_i}` opens it, `remove <player> from viewers of {_i}` closes it.
- `viewers of {_i}` lists the players who have a menu open that was built from that value, or the player of
  `event-inventory`.

## Values of a type, slots, spawn eggs, stored enchantments

```
loop all difficulties:
loop all boss bar styles:
send "%all of the moon phases%"
send "%index of clicked slot%"
send "%spawn egg entity of player's tool%"
set spawn egg entity of {_egg} to cow
set stored enchantments of player's tool to "sharpness 5"
loop stored enchantments of player's tool:
```

- `all <type>s` / `every <type>` lists every value of a named-value type (see [types](named-types.md)).
- `[raw|unique] index of <slot>` is the slot number of `clicked slot` or `slot N of <inventory>`.
- `spawn egg entity of <item>` is the entity type a spawn egg makes. Setting it changes an item kept in a variable
  into another spawn egg.
- `stored enchantments of player's tool` are the enchantments kept in an enchanted book, as texts like
  `sharpness 5`. Set them to a text or a list of texts, or `delete` them. Main-hand item only.

## Event values

| Name | In | Same as |
|---|---|---|
| `source block` | `on spread` | `event-source-block` |
| `tamer` | `on tame` | `event-owner` |
| `shooter` | `on shoot` | `event-entity` |
| `spawning reason` | `on spawn` | `spawn reason` |
| `target` | `on target` | `event-target` |

## Shoot

```
shoot an arrow from player at speed 2:
    send "%type of event-entity%"
shoot a snowball:
    set {_s} to shot entity
```

Spawns the projectile one block in front of the shooter's eyes and gives it a velocity along their view (default
speed 5). In the body, `event-entity`, `event-projectile`, `shot entity` and `shot projectile` are the projectile.
The shooter defaults to `player`. The plugin API cannot set a projectile's owner, so hits are not credited to the
shooter and `on shoot` does not fire.

## Types

`block data`, `enchantment type`, `entity type with amount`, `executable`, `potion effect` and `type` are accepted
as function parameter types. They have no value of their own: block data and stored enchantments are texts, an
entity type with amount is written in `spawn 3 zombies`, and type names are used in `all <type>s` and `parsed as`.

`suppress type hints:` runs its body; Carve has no type hints to hide.
