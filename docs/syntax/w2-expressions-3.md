# Player stats, hashes, memory and more

These expressions work on online players. If the player is offline, the result is `<none>`.

## Player stats

| Syntax | Type | Notes |
| --- | --- | --- |
| `exhaustion of player` | number | Food exhaustion. Set, add and remove. Limited to 0 to 40. |
| `fall distance of player` | number | Blocks fallen. Set, add and remove. Never below 0. |
| `burn duration of player` | timespan | Also `burning time`, `fire duration`. Set, add and remove. |
| `freeze time of player` | timespan | Powder snow freeze time. Set, add and remove. |
| `gliding state of player` | boolean | Elytra flight. Read only. |
| `glowing state of player` | boolean | Set it with `set glowing state of player to true`. |
| `gravity of player` | boolean | Set it with `set gravity of player to false`. |
| `hotbar slot of player` | number | 0 to 8. Read only. |
| `facing of player` | text | `north`, `south`, `east`, `west`, `up` or `down`. |
| `head location of player` | location | The eye position. Also `eye location`. |
| `hidden players of player` | list | Online players this player cannot see. |
| `first empty slot of player` | number | 0 to 35. `<none>` when the inventory is full. |

The possessive form works too: `player's exhaustion`, `player's head location`.

```
set exhaustion of player to 0
add 2 seconds to player's burn duration
set {_y} to y of head location of player
send "you face %facing of player%"
loop hidden players of player:
    send "%loop-value% is hidden from you"
```

New conditions: `player is gliding`, `player is glowing`, and their `is not` forms.

## Numbers and memory

- `infinity` and `negative infinity` are numbers: `if {_x} < infinity:`.
- `free memory`, `max memory` and `total memory` are in megabytes. They report the machine's memory, not a Java heap.

## Text

- `"text" hashed with MD5` gives lowercase hex. Supported: `MD5`, `SHA-1`, `SHA-256`, `SHA-512`.
- `<timespan> formatted` gives `1:05:03`, `5:03`, or `2:01:05:03` when days are present.
- `hex code of "red"` or `hex code of "#00ff7f"` gives `FF5555` or `00FF7F`. Named chat colors only.
- `id of diamond sword` gives `minecraft:diamond_sword`.
- `exact item of <item>` returns the item. Items have no metadata in Carve yet, so nothing changes.

## Lookups

- `player from uuid "..."` finds an online player. Offline gives `<none>`. Worlds cannot be looked up by UUID.
- `highest solid block at <location>` gives the location of the highest non-air block in that column.
- `final damage` in `on damage` is the same value as `damage`.

## Lists

- `{a::*} except {b::*}` and `{a::*} except "x"` remove matching items.
- `first index of value "a" in {l::*}` and `last index of value ...` give one index.
- `indices of value "a" in {l::*}` gives all indices, so you can loop over it.
