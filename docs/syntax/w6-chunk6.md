# Portal, protocol, rotation, villagers, whitelist and block entity reads

## Entities

```
set portal cooldown of player to 7 seconds
send "%portal cooldown of player%" to player

send "client %protocol version of player%, server %protocol version%" to player

set yaw of player to 90
add 45 to yaw of player
set pitch of last spawned entity to 30

set {_ride} to vehicle of player
set {_riders::*} to passengers of last spawned entity

send "%villager level of {_v}% %villager profession of {_v}%" to player
```

- `portal cooldown of` is a timespan (ticks under the hood) and can be set.
- `protocol version of <player>` is the number of the version the Java client joined with. Without a player it is the
  server's own (777 for Minecraft 26.3). Bedrock players have no number.
- `yaw` / `pitch` of a player or entity can be set, added to and removed from. Pitch is clamped to -90..90.
- `vehicle of` and `passengers of` read what an entity rides and what rides it. To change that, use the `ride`
  command (`execute console command "ride <name> mount <uuid>"`).
- `villager level`, `villager experience` and `villager profession` are read only. The profession is text such as
  `librarian` or `none`.

## Vectors and quaternions

```
set {_q} to quaternion(0.5, 0.5, 0.5, 0.5)
send "%w of {_q}% %rotation angle of {_q}% %rotation axis of {_q}%" to player

set {_v} to vector(1, 0, 0) rotated around z by 90
set x of {_v} to 5
add 2 to y of {_v}
set yaw of {_v} to 90
```

- `rotation angle of` gives degrees; `rotation axis of` a unit vector (`0, 0, 1` when there is no rotation).
- `<vector or quaternion> rotated around x|y|z|<vector> by <degrees>`.
- `x`, `y`, `z` and `w` of a quaternion read like those of a vector. `set`, `add` and `remove` on `x|y|z|w of {var}`
  rewrite the variable, so they only work on variables.
- `yaw` and `pitch` of a vector variable turn it and keep its length.

## Lists and items

```
set {_l::*} to 5 times
set {_p} to plain (diamond sword named "x")
```

- `N times` is the list 1..N.
- `plain <item>` keeps the type and amount and drops name, lore and enchantments.

## World and server

```
set whitelist to true
set {_w::*} to whitelisted players
add 10 to world border size
remove 30 from world border size
set spawn of world "world" to location(7, 100, 7)
```

- `whitelist enabled` reads the switch; `set whitelist to true|false|on|off` changes it.
- `add` and `remove` on `world border size [of <world>]` change the diameter at once.
- `set spawn of <world>` runs `setworldspawn`, so it only works for the main world.

## Block entity reads

```
set {_lines::*} to sign text of block at player
send "%skull owner of {_b}% %skull texture of {_b}%" to player
send "%spawner type of {_b}% %loot table seed of {_b}%" to player
send "%ringing time of {_b}%" to player
```

- `sign text of` (also `lines of`) lists the four front lines.
- `skull owner of` and `skull texture of` read the skull's profile; none when it has none.
- `spawner type of` reads the entity a spawner spawns. `loot table seed of` reads `LootTableSeed` of chests, barrels
  and brushable blocks.
- `ringing time of` is a bell's ring time as a timespan.
- Pumpkin's `data merge block` cannot fill these block entities, so the in-game test only checks the empty case.
