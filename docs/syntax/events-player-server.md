# Player and server events (kick, leash, piston, portal, redstone)

These triggers work like `on join:`. Each one can be cancelled with `cancel event`.

Two values are new in these events:

- `event-text` (or `message`) is the text value of the event.
- `event-number` (or `damage`) is the number value of the event.

Only events that have the value allow it. Using the wrong one is a load error.

```
on kick:
    send "%player% was kicked for %event-text%" to console
    cancel event

on redstone:
    if event-number > 10:
        broadcast "strong signal"
```

## Events

| Event (aliases) | `player` | `event-text` | `event-number` |
| --- | --- | --- | --- |
| `on kick` (`player kick`) | yes | reason | |
| `on language change` (`locale change`) | yes | new locale | |
| `on leash` | yes | | |
| `on unleash` | | reason | |
| `on love mode enter` | the feeder, if a player | | ticks in love |
| `on piglin barter` | | | |
| `on portal` | if the entity is a player | | |
| `on portal enter` | if the entity is a player | | |
| `on portal create` | | portal type | |
| `on resurrect attempt` (`resurrect`) | if the entity is a player | | |
| `on resource pack response` | yes | status | |
| `on send command list` | yes | command | |
| `on lightning strike` (`lightning`) | | | |
| `on leaves decay` | | | |
| `on piston extend` / `on piston retract` | | direction | |
| `on redstone` (`redstone change`) | | | new power |
| `on loot generate` | | loot table | |
| `on player input` (`input`) | yes | input bits | input as a number |

`player input` bits: 1 forward, 2 back, 4 left, 8 right, 16 jump, 32 sneak, 64 sprint.

## Notes

- Events with no player (`piston`, `redstone`, `lightning`, ...) do not allow `me`.
- Block positions are not exposed yet for the block events.
- `redstone`, `piston`, `leaves decay` and `player input` happen very often. The plugin only listens to them if a script uses them when the server starts. If you add the first script for one of them later, restart the server.
- `event-block` and the older click values are unchanged.
