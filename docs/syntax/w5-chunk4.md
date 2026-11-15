# Block data, books, borders, attributes and enchanting

## Block data

```
send "%block data of block at player%"
set block data of block at {_l} to "oak_stairs[facing=west,half=top]"
send "%{_l}'s block data%"
```

The text is the block id with its state properties, like
`minecraft:oak_stairs[facing=east,half=bottom,shape=straight,waterlogged=false]`.
Set it to a text in the same form; properties you leave out get their
default. An unknown block or property value is a runtime error.

## Lists of blocks

```
set {_ball::*} to blocks in radius 3 of player
loop all blocks around {_center} in radius 2:
set {_box::*} to blocks between {_a} and {_b}
set {_line::*} to blocks from player to {_target}
set {_up::*} to blocks above player
```

Every result is a list of block locations (whole coordinates), so `block at
loop-value`, `type of block at ...` and `set block at ...` work on them.

- `in radius`: every block whose center is within the radius. Up to radius 32.
- `between a and b`: every block of the box, both corners included. Up to 262144 blocks.
- `from a to b`: the blocks a straight line between the two block centers passes through, in order. Up to 1000.
- `above`, `below`, `north`, `south`, `east`, `west`: the 100 blocks next to the location in that direction.

## Written books

```
set book title of player's tool to "Rules"
set book author of player's tool to "Admin"
set book pages of player's tool to "Be nice"
set page 2 of player's tool to "No griefing"
send "%size of book pages of player's tool% pages by %author of player's tool%"
```

These work on the held item only (`player's tool`): item values in variables
do not keep book content. Pages are plain text.

## Boss bar from key

```
set {_b} to boss bar with id "raid"
if {_b} is set:
    show bossbar {_b} to player
```

Bars are named by their text id, so this gives the id back when a bar with it
exists, and none otherwise.

## Brewing stand slots

```
send "%first bottle slot of block at {_stand}%"
set ingredient slot of block at {_stand} to nether wart
set brewing stand's fuel slot of block at {_stand} to blaze powder
```

`first`, `second` and `third bottle [slot]`, `ingredient slot` and `fuel slot`.
They read and write the stand's slots; a stand placed by a script gets its slots on the first set.

## World border center and damage

```
set {_b} to world border of world "world"
set world border center of {_b} to location(100, 0, -50)
set world border damage amount of {_b} to 1
add 2 to world border damage buffer of {_b}
reset world border center of {_b}
```

The center is a location at y 0. The damage amount is the damage per block
outside the border per second, the buffer the distance past it that is still
safe. A world works in place of its border.

## Potion effect values

```
set {_e} to potion effect of speed of tier 2 for 30 seconds
set {_n} to a new ambient potion effect of night vision without particles for 5 minutes
apply {_e} to player
```

The value is the effect's NBT in the vanilla format (`id`, `amplifier`,
`duration`, `ambient`, `show_particles`, `show_icon`). Without `for` it lasts
15 seconds. `apply {_e} to player` keeps the ambient and particle settings.
Effects go on players only.

## Item damage and durability

```
give diamond sword with damage 500 to player
set {_s} to iron pickaxe damaged by 10
send "%damage of {_s}% used, %durability of {_s}% left"
set damage of player's tool to 0
remove 10 from durability of player's tool
```

`damage` is the used-up durability, `durability` what is left (from the
vanilla maximum of tools and armor). Item values keep their damage in
variables. Setting works on the held item.

## Ender chests

```
set slot 0 of ender chest of player to diamond
send "%slot 4 of player's ender chest%"
send "%size of ender chest of player% stacks"
clear ender chest of player
open player's ender chest to player
```

Slots are 0 to 26. Pumpkin can only open a player's own ender chest, so
`open ... to` another player is a runtime error.

## Entities

```
set ai of last spawned entity to false
send "%ai of {_mob}%"
set movement speed attribute of player to 0.15
send "%max health total attribute of player%"
reset scale attribute of player
set carried block of {_enderman} to "tnt"
```

- `ai of` is for mobs; players have none.
- An attribute is any of the `attribute type` names (`movement speed`,
  `max health`, `attack damage`, `scale`, `gravity`, ...), with or without
  `generic`. Without `total` it is the base value; set, add, remove and reset
  change the base.
- `carried block` is block data text and can be set to a block type or a
  block data text.

## Enchanting

```
on enchant prepare:
    send "%size of enchantment offers% offers, bonus %enchantment bonus%"
    send "%enchantment offer 1% costs %cost of enchantment offer 1%"
on enchant:
    send "%enchant item% for %exp cost% levels, hint %enchantment hint% %enchantment hint level%"
```

- An offer is a value with an enchantment, a level and a cost. Use
  `cost of <offer>` and `enchantment level of <offer>`.
- `enchantment hint` is the enchantment the table showed for the chosen
  offer. When no script listens to `on enchant prepare` it is the first
  enchantment added instead.
- `enchantment level of "sharpness 5"` also reads text (`sharpness IV` works,
  no number is 1).
- All of these are read only.

## Inventory action and cursor slot

```
on inventory click:
    if inventory action is "pickup all":
        send "picked up %clicked item%"
    send "cursor: %cursor slot of player%"
```

The action is worked out from the click type, the clicked item and the cursor
item, with Bukkit's names: `pickup all`, `pickup half`, `place all`,
`place some`, `place one`, `swap with cursor`, `drop all cursor`,
`drop one cursor`, `drop all slot`, `drop one slot`,
`move to other inventory`, `hotbar swap`, `collect to cursor`,
`clone stack`, `nothing`. `cursor slot` is the cursor item before the click
and only exists in `on inventory click`.

## Event values

```
on broadcast:
    send "%broadcast message%" to console
on consume:
    send "you ate %consumed item%"
on death:
    if the zombie is set:
        broadcast "a zombie died"
```

`broadcast message` and `consumed item` are read only. `the zombie`,
`the creature`, `the entity` and `event-<type>` work in `on damage` and
`on death` for the mob that was hurt or died; with a type they are none for
other mobs.
