# Wave 6, chunk 1: events that turned out to be possible

These events used to be marked blocked or partial. Each one was checked against
Pumpkin's plugin API and the server source. Most are verified in-game by
`tests/ingame/areas/w6-chunk1.js`.

Events that watch very busy server events (`on jump`, `on player chunk enter`,
`on step on <block>`, `on pressure plate`, `on experience spawn`) only attach
at startup when a script uses them, like `on move`. Adding the first script for
one of them needs a restart.

## New events

| Event | Values | Notes |
|---|---|---|
| `on jump` | `player` | A rising edge of the jump bit in the player input. Players only. |
| `on player chunk enter` (also `... chunk enters`, `on player enters chunk`) | `player`, `event-chunk-x`, `event-chunk-z`, `event-previous-chunk-x`, `event-previous-chunk-z` | From the old and new position of each move. |
| `on step on <block>` (also `walk on`, `move on`) | `player`, `event-block`, `event-location` | Runs when the player moves onto a different block that is `<block>` (`stone`, `oak planks`). |
| `on pressure plate` (also `on trip`) | `event-location`, `event-power` | Can be cancelled. Tripwires fire no event on Pumpkin. |
| `on experience spawn` | `event-location`, `event-entity` | Can be cancelled. |
| `on player pickup arrow` | `player`, `event-entity` | Can be cancelled. Not verified in-game. |
| `on riptide` | `player`, `event-item` | Can be cancelled. Not verified in-game. |
| `on projectile collide` | `event-entity`, `event-hit-entity`, `event-location` | Arrows that hit an entity. Not verified in-game. |
| `on armor change` | `player`, `event-slot`, `event-item`, `past event-item` | Found by checking the armor slots every 4 ticks. |
| `on spectate` | `player`, `event-entity` | Found by checking the camera every 2 ticks. |
| `on world border bounds change` | `event-world`, `event-old-size`, `event-new-size` | Found by checking the target size every tick. |
| `on world border center change` | `event-world`, `event-old-x`, `event-old-z`, `event-new-x`, `event-new-z` | Same. |
| `on unload` (also `on script unload`) | | Before `/carve reload` replaces the scripts, and when the server stops. |
| `on server stop` | | When the server stops. |
| `on entity break door` | `event-entity`, `event-block` | Any mob. `on zombie break door` now only runs for zombie, husk, drowned and zombie villager. |

```
on step on stone:
    send "stone under you" to player

on player chunk enter:
    send "chunk %event-chunk-x%, %event-chunk-z%" to player

on armor change:
    send "%event-slot%: %past event-item% -> %event-item%" to player
```

## Changed events

- `on tool change` has `event-item` (the new tool) and `past event-item`.
- `on spawn change` also runs for a player's spawn point (`/spawnpoint`, beds).
- `on shoot` also runs for players' arrows (`event-projectile` is the arrow), through the arrow launch event.
- `on bell ring` has `player` when a player rang the bell.
- `on block drop` has `event-items`, the whole list.
- `on bucket catch entity` and `on combust` have `event-entity`.
- `on can build check` can change the result: `set event-boolean to false`.
- `on leaves decay`, `on lightning strike`, `on piston extend`, `on piston retract`,
  `on portal create` and `on redstone` have `event-location`.
- `on leash`, `on unleash` and `on love mode enter` have `event-entity`.
- `on piglin barter` has `event-item` (the input) and `event-items` (the drops).

## Still blocked

- `on attempt attack`, `on stop using item`, `on stonecutter recipe select`,
  `on unknown command execution`: they would need Pumpkin's packet event, which
  panics inside Pumpkin 0.2.2 when it decodes a packet (the server crashes right
  after a player joins). Protocol 26.3 also sends attacks as their own packet
  without any plugin event.
- `on world border bounds finish change`: Pumpkin does not keep the length of
  a border transition (the client does the lerping), so the end is unknown.
- The server never fires the events for chunk load, bat toggle sleep, beacons,
  bell resonate, creeper power, pig zap, pathfinding, ready arrow, slime split,
  sheep wool regrow, vault display item, villager career change, fade, anvil
  damage and area effect clouds, and an experience cooldown change.

## Limits

- Events that happen inside a running script (`ignite`, `strike lightning`) are
  skipped when Pumpkin fires them while the script runs, because Carve is
  already busy. `on lightning strike` is the exception: Carve runs it right
  after the script.
- `event-entity` of `on experience spawn` is looked up by the entity id the
  server sends, so it can be `<none>` if the orb is not in the world yet; use
  `event-location` there.
