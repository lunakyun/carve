# Horns, trees, flight, fake blocks and more

Every example below runs on Pumpkin 0.2.2.

## Goat horns

```
spawn a goat at player
remove the left horn of last spawned entity
regrow both horns of last spawned entity
```

Takes the horns off a goat, or grows them back. Goats only.

Syntax:

- `remove [the] (left|right|both) (horn|horns) of %object%`
- `remove %object%'s (left|right|both) (horn|horns)`
- `(regrow|replace) [the] (left|right|both) (horn|horns) of %object%`
- `remove [the] horns of %object%`
- `(regrow|replace) [the] horns of %object%`

## Silence entity

```
spawn a cow at player
silence last spawned entity
```

Stops an entity from making sounds, or lets it again.

Syntax:

- `silence %object%`
- `unsilence %object%`
- `make %object% silent`
- `make %object% not silent`

## Make entity scream

```
spawn a goat at player
make last spawned entity scream
```

Switches the screaming goat voice. Goats only.

Syntax:

- `make %object% (start screaming|scream)`
- `force %object% to (start screaming|scream)`
- `make %object% stop screaming`
- `force %object% to stop screaming`

## Grow tree

```
grow oak at location(10, 64, 10)
```

Grows a tree at a location with Pumpkin's `place feature` command. Types: oak, tree, big tree, birch, jungle, acacia, dark oak, spruce, redwood, cherry, mangrove. The tree needs the right ground, like a sapling.

Syntax:

- `(grow|create|generate) [(a|an)] <type> [tree] (at|on|above) %location%`

## Zombify villager

```
spawn a villager at player
zombify last spawned entity
```

Replaces a villager with a zombie villager at the same spot, keeping its name and health (and back). Pumpkin has no conversion call, so this is a replacement.

Syntax:

- `zombify %object%`
- `unzombify %object%`

## Make allay duplicate

```
spawn an allay at player
make last spawned entity duplicate
```

An allay makes a copy of itself. Both allays then wait before they can do it again.

Syntax:

- `make %object% (duplicate|clone)`

## Enable, disable and reload script

```
reload script "homes"
```

Runs `/carve enable`, `/carve reload` or `/carve disable` for a script file. `unload` disables the script (the file is renamed with a `-`).

Syntax:

- `(enable|load) script [(file|named)] %text% [with errors]`
- `reload script [(file|named)] %text% [with errors]`
- `(disable|unload) script [(file|named)] %text% [with errors]`

## Reset and remove all

```
set {n} to 5
reset {n}
set {l::*} to 1, 2, 1 and 3
remove all 1 from {l::*}
```

`reset` sets a number to 0, a text to empty and a boolean to false; any other value, and a list, is deleted. `remove all x from {list::*}` removes every x.

Syntax:

- `reset %variable%`
- `remove all %object% from %variable%`

## Rotate vector

```
set {v} to vector(1, 0, 0)
rotate {v} around the z-axis by 90
rotate {v} around vector(0, 1, 0) by 45
```

Rotates the vector in a variable (every vector of a list variable) by degrees, around an axis or around another vector.

Syntax:

- `rotate %variable% around [the] (x|y|z) [-] axis by %number%`
- `rotate %variable% around %object% by %number%`

## Allow flight

```
allow flight for player
```

Lets a player fly (double-tap jump) or takes it away. Only the permission changes; `make player fly` also starts the flight.

Syntax:

- `(allow|enable) (fly|flight) (for|to) %object%`
- `(disallow|disable) (fly|flight) (for|to) %object%`

## Make player see a block

```
make player see location(5, 101, 5) as diamond_block
make player see location(5, 101, 5) as original block
```

Shows a player a block that is not there (only that player sees it), or puts the real block back.

Syntax:

- `make %object% see %location% as %object%`
- `make %object% see %location% as [(the|its|their)] (original|normal|actual) [(block|blocks)]`

## Make interaction responsive

```
spawn an interaction at player
make last spawned entity responsive
```

Makes an interaction entity answer clicks with the swing animation.

Syntax:

- `make %object% responsive`
- `make %object% (unresponsive|not responsive)`

## Text display drop shadow and see through

```
spawn text display at player with text "hi"
apply drop shadow to last spawned entity
make last spawned entity visible through blocks
```

Switches the drop shadow of a text display and whether it shows through blocks.

Syntax:

- `(apply|add) (drop|text) shadow to [[the] text of] %object%`
- `(remove|clear) (drop|text) shadow from [[the] text of] %object%`
- `make %object% visible through (blocks|walls)`
- `(prevent|block) %object% from being (visible|seen) through (blocks|walls)`

## Stop sound in a category

```
stop sound "entity.cow.ambient" in music for player
stop all sounds in master for player
```

Stops a named sound, or every sound, of one category: master, music, record, weather, block, hostile, neutral, player, ambient, voice. Without a player it goes to the one who started the trigger.

Syntax:

- `stop (sound|sounds) %text% (in|from) [the] <category> [(from playing to|for|to) %object%]`
- `stop all (sound|sounds) (in|from) [the] <category> [(from playing to|for|to) %object%]`

## Shear and unshear

```
spawn a sheep at player
shear last spawned entity
```

Shears a sheep (the wool grows back with `unshear`). Sheep only; the wool is not dropped.

Syntax:

- `shear %object%`
- `unshear %object%`

## Tame and untame

```
spawn a wolf at player
tame last spawned entity
```

Tames a wolf or a cat, with the player who started the trigger as the owner, or frees it. Other tameable animals are not supported.

Syntax:

- `tame %object%`
- `untame %object%`
- `domesticate %object%`

## Ban for a time

```
ban player because of "spam" for 2 hours
```

A ban that ends by itself. The player has to be online; Pumpkin's ban list needs a UUID, which a plain name does not give. A later `ban` without a time is permanent.

Syntax:

- `ban %object% [(because of|by reason of|due to|with reason) %text%] for %timespan%`
- `ban ip %object% [(because of|by reason of|due to|with reason) %text%] for %timespan%`

## Ride and dismount any entity

```
spawn a pig at player
make player ride last spawned entity
make player dismount
```

Puts any entity or player on another entity, or takes it off. A player can not be the vehicle.

Syntax:

- `(make|let|force) %object% [to] (ride|mount) [(in|on)] %object%`
- `(make|let|force) %object% [to] dismount`

## Swing hand

```
spawn a zombie at player
make last spawned entity swing main hand
```

Plays the arm swing of any living entity.

Syntax:

- `make %object% swing [their] [main] hand`
- `make %object% swing [their] off hand`

## Equip

```
spawn a zombie at player
equip last spawned entity with diamond helmet and iron boots
```

Puts up to four items on a player or a mob. Each item goes to its own slot: helmets on the head, shields in the off hand, everything else in the main hand.

Syntax:

- `equip %object% with %object% [and %object%]`
- `make %object% wear %object% [and %object%]`
- `equip with %object%`
