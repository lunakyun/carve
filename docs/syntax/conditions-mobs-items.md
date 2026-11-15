# Mob checks, lids, effect flags, server lists and event values

## Mob and player checks

```
if {_wolf} is tamed:
if {_wolf} is tameable:
if {_pig} is saddled:
if {_enderman} is screaming:
if player is riptiding:
if player is wearing diamond helmet:
if {_zombie} is not wearing iron boots:
```

All of them have a negated form (`is not`, `isn't`, `aren't`).

- `tamed` reads the wolf and cat data. Other tameable mobs (horses, parrots, ...)
  have no data in Pumpkin's plugin API, so the check is false either way.
- `tameable` is true for wolves, cats, parrots, horses, donkeys, mules, llamas,
  trader llamas, skeleton and zombie horses and camels.
- `saddled` means something sits in the saddle slot.
- `screaming` is the enderman scream. Goats have no data, so it is false.
- `riptiding` is the trident spin pose.
- `wearing` looks at the head, chest, legs, feet, body and saddle slots of a
  player or mob. With several items (`wearing {_set::*}`) every one must be worn.

A check the game can't answer is false, negated or not.

## Item and type checks

```
if diamond is stackable:
if "zombie" is spawnable:
if {_type} is spawnable in world "world":
if player's tool has enchantment glint overridden:
if tool of player doesn't have an enchantment glint override:
```

- `stackable`: the item stacks to more than 1.
- `spawnable`: a known entity type other than `player` and `fishing_bobber`.
  Type names are text (`"zombie"`, `type of {_e}`) or `"zombie" parsed as entity type`.
- The glint check works on the held item only: item values in variables do not
  keep data components. Set it with `set glint override of player's tool to true`.

## Lids

```
if lid of {_chest} is open:
if the lid of location(10, 64, 10, world "world") is closed:
if {_barrel}'s lid is opened:
```

Chests and trapped chests are open while someone looks inside. Barrels use their
`open` block state. Pumpkin reports no viewers for ender chests and shulker
boxes, so they (and blocks without a lid) are neither open nor closed.

## Potion effect flags

```
if player's speed effect has an icon:
if player's speed effect doesn't have particles:
if potion effect of haste on player is ambient:
```

Carve has no potion effect values yet, so these name the effect type and the
player. They work for players (the plugin API has no effects for mobs). A
player without that effect makes the check false either way.

## Text displays

```
if {_display} has a drop shadow:
if text of {_display} is visible through blocks:
if {_display} is not visible through walls:
```

## Resource pack response

```
on resource pack response:
    if the resource pack was successfully loaded:
        send "Thanks!"
    if resource pack was declined:
        kick player due to "Please accept the pack"
```

States: `accepted`, `declined`, `downloaded`, `successfully loaded`,
`failed download`, `invalid url`, `failed reload`, `discarded`.

## Any of

```
if {_n} is any of {_allowed::*}:
if player is not any of all operators:
if any of {_list::*} is "admin":
send "%any of {_l::*}%"
```

`x is any of <list>` is true when x equals one of the items. `is not any of` is
true when it equals none of them. Outside these checks `any of` is just the list.

## Operators and bans

```
send "%all operators%"
loop all banned players:
    send "%loop-value%"
set {_ips::*} to banned ips
```

`all operators` and `all banned players` are players (online or not),
`banned ips` is text.

## Beacons

```
send "%primary beacon effect of {_beacon}%"
send "%secondary effect of {_beacon}%"
```

The effect name (`speed`), or `<none>`. Read only: the plugin API has no call to
change a beacon, and Pumpkin 0.2.2 has no beacon screen on Java and does not
apply `/data merge` to a loaded beacon, so in game a beacon always reads `<none>`
today.

## Bed location

```
send "%bed location of player%"
set player's bed location to location(0, 70, 0)
```

The same as `respawn location`.

## Event values

| Event | Value |
|---|---|
| `on anvil prepare` | `anvil text input` (the rename text), `anvil repair cost` (Pumpkin always sends 1) |
| `on enchant` | `applied enchantments`: a list like `sharpness 3` (read only) |
| `on piglin barter` | `barter input` (the item), `barter drops` (a list of items) |

```
on piglin barter:
    set barter drops to 3 diamonds
    add 1 gold nugget to barter drops
    remove gravel from barter drops
    clear barter drops
```

The changed drops stay in the script (`barter drops` reads them back), but
Pumpkin 0.2.2 copies only `cancelled` back from the plugin's barter event, so
the piglin still throws its own drops. Cancel the event to stop them.
