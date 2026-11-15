# More conditions

All of these work in `if`, `else if`, `while`, `and` and `or`, and have a negated form.
A check the game can't answer (a hive that isn't there, an unknown boss bar) is false in
both forms.

## Damage cause

```
on damage:
    if damage was caused by fall:
    if the damage wasn't caused by "lava":
    if damage is caused by drowning:
```

Only in `on damage`. Takes the game's damage type names (`fall`, `lava`, `in_wall`,
`player_attack`, `out_of_world`) and the Skript names for them: `drowning`, `suffocation`,
`starvation`, `void`, `fire`, `fire tick`, `lightning`, `contact`, `projectile` (arrows,
tridents, fireballs, ...), `entity attack` (mob and player hits), `block explosion` and
`entity explosion`.

## Egg will hatch

```
on throwing of an egg:
    if the egg will hatch:
    if egg won't hatch:
```

Only in `on throwing of an egg`. The same as `event-hatching`.

## Entity storage

```
if entity storage of block at {_loc} is full:
if {_loc}'s entity storage isn't full:
```

A beehive or bee nest is full with 3 bees.

## Equippable items

```
if diamond helmet can be dispensed:
if saddle can be sheared off:
if saddle can be equipped on entities:
if carved pumpkin can't swap equipment:
if elytra won't lose durability when the wearer is hurt:
```

These read the item's vanilla equippable component. An item that can't be worn (stone)
is false. Custom components set on one item stack are not read.

## Boss bar flags

```
if bossbar "raid" has darken sky flag:
if bossbar "raid" doesn't have the fog flag:
if bossbar "raid" has the flag boss music:
```

The flags are `darken sky`, `fog` (or `create fog`) and `boss music` (or `dragon bar`).

## Tooltips

```
if tooltip of player's tool is shown:
if the additional tooltip of {_item} is hidden:
if {_item} has an entire tooltip:
```

Carve item values carry no tooltip settings, so the tooltip always counts as shown.

## Loot tables

```
if block at {_loc} has a loot table:
if block at {_loc} doesn't have a loottable:
```

True for a chest, barrel or other container that hasn't been filled from its loot table
yet.

## Metadata and persistent data

```
if player has metadata "frozen":
if player doesn't have metadata value "frozen":
if player has persistent data tag "kit":
if player does not have data tag "kit":
```

The same targets as `set metadata value` and `set persistent data value`: players,
worlds and item types. Keys don't care about case.

## Played before

```
on join:
    if player has played before:
    if player hasn't played on this server before:
```

Carve remembers who joined in `first_join.txt` in its data folder. A player who joins for
the first time hasn't played before for the whole session; after they leave and come back,
they have.

## Preferred tool

```
if diamond pickaxe is the preferred tool for block at {_loc}:
if player's tool is {_loc}'s preferred tool:
if stick isn't the preferred tool for stone:
```

True when breaking the block with the item gives its drops: any item for blocks like
dirt, otherwise the vanilla tool rules (pickaxe for stone, a diamond pickaxe or better
for obsidian, shears or a sword for cobwebs).

## Responsive

```
if {_interaction} is responsive:
if last spawned entity isn't responsive:
```

For interaction entities. Any other entity is not responsive.

## Already covered

These Skript conditions were already in Carve (see [conditions](conditions.md)) and are
now checked in game: `can pick up items`, `can see`, `can see chat colors`,
`can see messages`, `has chat filtering`, `has client weather`, `respawn anchors work`,
`has been stared at`, `is in liquid`, `is sheared`, `is wet`, `has ai`, `has glowing text`,
`has item cooldown`, `has line of sight` and `contains`.
