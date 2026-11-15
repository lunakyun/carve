# Game events (heal, grow, explode, fishing, furnace)

These events all work like `on join:`. Each has event values you can read
inside the trigger. Values are plain numbers, texts or true/false.

```
on heal:
    if heal amount > 5:
        send "big heal"      # only runs when a player healed

on grow:
    if new block is "wheat":
        broadcast "wheat grew at %block x%, %block y%, %block z%"
```

## Rules

- `cancel event` works except where the table says "no cancel".
- Entities are numbers: the entity id. `event-entity` gives it. Pumpkin's
  plugin API has no entity object yet.
- "Player" column: **yes** means `player` always works. **maybe** means the
  trigger runs for every entity and `player` is set only when it is a
  player. **no** means `player` is a compile error there.
- Blocks are written as texts without `minecraft:`, like `wheat`.
- Short forms: `event-number` is the first number value, `event-text` the
  first text, `event-boolean` the first true/false. `event-item` is the item
  text where there is one. `event-player` is `player`.
- Value names are only known inside their own event. Anywhere else they are
  a compile error.

## Player events

| Event (aliases) | Player | Values |
|---|---|---|
| `on enchant` (`enchant item`) | yes | `enchanted item`, `enchantment cost`, `enchant option`, `enchantment count` |
| `on enchant prepare` | yes | `enchanted item`, `bookshelf count`, `enchant offer count` |
| `on experience change` (`xp change`) | yes, no cancel | `experience amount` |
| `on experience cooldown change` | yes | `new cooldown` (Pumpkin does not fire it yet) |
| `on fishing` (`fish`) | yes | `fishing state` (`bite`, `caught fish`, ...), `caught type`, `fishing xp` |
| `on harvest block` | yes | `harvested count`, `block x`, `block y`, `block z` |
| `on furnace item extract` | yes, no cancel | `extracted item`, `extracted amount`, `extracted xp`, `block x/y/z` |
| `on inventory open` | yes | none |
| `on inventory close` | yes, no cancel | none |
| `on inventory drag` | yes | none |
| `on item break` | yes, no cancel | `broken item` |
| `on item damage` | yes | `damaged item`, `item damage amount` |
| `on item mend` | yes | `mended item`, `repair amount`, `mending xp` |

## Entity events

| Event (aliases) | Player | Values |
|---|---|---|
| `on heal` (`entity heal`) | maybe | `heal amount`, `event-entity` |
| `on hunger meter change` (`food level change`) | maybe | `new food level`, `event-entity` |
| `on entity mount` | maybe | `event-entity`, `vehicle id` |
| `on entity dismount` | maybe | `event-entity`, `vehicle id` |
| `on entity potion effect` | maybe | `effect name`, `effect duration`, `effect amplifier`, `event-entity` |
| `on entity shoot bow` | maybe | `bow item`, `bow force`, `event-entity` (mobs only in Pumpkin) |
| `on entity transform` | maybe | `transform reason`, `new entity id`, `event-entity` |
| `on gliding state change` | maybe | `gliding state` (true when it starts), `event-entity` |
| `on entity breed` | no | `father id`, `mother id`, `child id` |
| `on enderman/sheep/silverfish/falling block` | no | `new block`, `block x/y/z`, `event-entity` |
| `on horse jump` | no | `jump power`, `event-entity` |
| `on firework explode` | no | `event-entity` |
| `on explosion prime` | no | `explosion radius`, `explosion fire`, `event-entity` |
| `on explode` | no | `explosion yield`, `explosion x/y/z`, `event-entity` (-1 for block explosions) |
| `on item spawn` | no | `spawned item`, `event-entity`, `block x/y/z` |
| `on item despawn` | no | `event-entity` |
| `on item merge` | no | `event-entity`, `merge target id` |

## Block events

All of these have no player.

| Event (aliases) | Values |
|---|---|
| `on grow` | `old block`, `new block`, `block x/y/z` |
| `on form` | `block x/y/z` |
| `on fade` | `block x/y/z` (Pumpkin does not fire it yet) |
| `on ignition` | `block x/y/z` |
| `on flow` (`liquid flow`) | `flow source x/y/z`, `flow target x/y/z` |
| `on fuel burn` | `fuel item`, `burn time`, `block x/y/z` |
| `on inventory item move` | `moved item`, `moved amount`, `move source x/y/z`, `move target x/y/z` |
| `on inventory pickup` | `picked up item`, `item entity id`, `block x/y/z` |

`on inventory item move` and `on inventory pickup` are hopper events.

## Not possible on Pumpkin

- `on enderman enrage`: no event.
- `on entity jump`: no event.
- `on experience spawn`: no event.
- `on inventory slot change`: no event.
