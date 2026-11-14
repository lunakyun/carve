# Player states

Conditions that ask what a player is doing right now. Each has a "not" form.

```
on right click:
    if player is sneaking:
        send "You are sneaking."
    if player is not on the ground:
        send "You are in the air."
```

| Condition | Not form | Notes |
|---|---|---|
| `player is flying` | `player is not flying` | tracked from flight toggle events |
| `player is sneaking` | `player is not sneaking` | |
| `player is sprinting` | `player is not sprinting` | |
| `player is swimming` | `player is not swimming` | |
| `player is on ground` | `player is not on ground` | `on the ground` also works |
| `player is sleeping` | `player is not sleeping` | |
| `player is blocking` | `player is not blocking` | always false, see below |
| `player can fly` | `player can't fly` | `cannot` also works |

`isn't` works in place of `is not`.

## Toggle events

`on sneak toggle:`, `on sprint toggle:` and `on flight toggle:` run when the
player changes that state. They have `player`. `cancel event` stops the change.
Inside them the check conditions above show the state from before the change.

## Not available

Pumpkin's plugin API has no way to do these yet:

- `is blocking` is always false (no shield state).
- `can fly` follows the game mode (creative and spectator can fly). A survival
  player with flight allowed by a plugin is not seen.
- Setting flight (`set flight mode`, `allow flight`, `toggle flight`) and
  `walk speed` / `fly speed`.
