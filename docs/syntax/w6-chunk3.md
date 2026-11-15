# Conditions the game can answer

Wave 6 checked every row marked as impossible against Pumpkin 0.2.2's real plugin
API (v0.1). These conditions all work now. Every one has a negated form
(`is not`, `isn't`, `aren't`). A check the game can't answer is false, negated
or not.

## Mobs and players

```
if {_cow} is a baby:
if {_pig} is an adult:
if {_creeper} is charged:
if {_creeper} is igniting:
if {_cow} is pathfinding:
if player is left handed:
if player is right handed:
if player is riding a horse:
if {_mob} is silent:
if {_mob} is from a spawner:
if player is holding diamond sword in the off hand:
```

- `baby` and `adult` read the mob's age data (zombies have a baby flag). Every
  other living mob is an adult.
- `charged` is a creeper struck by lightning, or a player holding a loaded
  crossbow in the main hand.
- `igniting` is a creeper with a lit fuse. Primed TNT is not in the plugin API.
- `pathfinding` is true while the mob's navigator follows a route.
- `left handed` and `right handed` read the player's client setting. Mobs have
  none.
- `riding` works for any entity. `riding a pig` compares the vehicle's type.
- `silent` works for players and other entities.
- `from a spawner` (also `spawned by a spawner`, `from a trial spawner`):
  Carve remembers the entities the spawner events report. A mob that spawned
  before Carve loaded counts as not from a spawner. The first script that uses
  it needs a server restart so the listener is registered.
- `holding` looks at both hands. Add `in the main hand` or `in the off hand` to
  pick one.
- `climbing` is an approximation: off the ground, standing in a ladder, vine,
  scaffolding or cave, twisting or weeping vine.

## Blocks

```
if {_block} is conditional:
if {_block} is occluding:
if {_block} is powered:
if {_leaves} is persistent:
if {_chest} is lootable:
if {_block} is solid:
if {_block} is passable:
```

These read the block at a location. `solid`, `transparent` and `passable` also
work on a placed block now (stairs and slabs by their own state), not just on a
block type.

- `conditional` is the command block's `conditional` property.
- `occluding` is a full cube with opacity 15.
- `powered` (also `redstone powered`, `indirectly powered`) reads the `powered`,
  `power` or `lit` property: levers, buttons, doors, lamps, repeaters, redstone
  dust. A solid block that is powered only through a neighbour has no such
  property, so it reads as not powered.
- `persistent` is the leaves property. Entities have no persistence flag in the
  plugin API.
- `lootable` is a container with a `LootTable`. Entities are not readable.

## Server

```
if plugin "Vault" is enabled:
if feature "bundle" is enabled:
if running minecraft "26.3":
if running above minecraft "1.21.4":
if not running minecraft "1.8":
if player is pressing the jump key:
```

- `plugin "x" is enabled`: a plugin with that name is loaded. Carve probes it
  with the plugin IPC call (an empty message), so its message handler runs.
- `feature` reads the enabled datapacks: `trade_rebalance`,
  `minecart_improvements`, `redstone_experiments` and `bundle`. `vanilla` is
  always on. `<world> is using the feature "x"` asks the same.
- `running minecraft` compares versions part by part. `"26"` matches every 26.x.
  The v0.1 plugin API reports only Pumpkin's version, so Carve maps the
  releases it knows (0.2.x is 26.3). An unknown release answers false.
- `pressing` takes `forward`, `backward`, `left`, `right`, `jump`, `sneak` and
  `sprint`, with or without `key`. It follows the input packets the client
  sends. Like the spawner check, the first script that uses it needs a restart.

## Flight, bans, tables

- `can fly` reads the player's `allow flying` ability (survival players can't,
  creative and spectator players can). `is flying` reads the live flag.
- `is banned` reads Pumpkin's ban list by name, by UUID, or for an IP address.
- `is edible`, `is fuel`, `is fire resistant` and `is flammable` use tables made
  from Pumpkin's own item and block data (food or consumable, fuel, damage
  resistant to fire, and blocks fire can burn).
- `is of type`: entities (`{_e} is of type pig`), blocks (`{_block} is of type
  stone`) and items.
- `is within <chunk>` and `is within the world border of <world>` for
  locations, entities and players.
- `is normalized` for vectors, `is infinite` for potion effects, and
  `player's tool is unbreakable`.
