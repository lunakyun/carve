# Health and food

Health is 0 to the player's max health (20 by default, 1 heart = 2).
Food level is 0 to 20. Saturation is 0 to 20.
Values outside the range are cut to the nearest limit.
Health that reaches 0 kills the player.

## Reading

| Expression | Type |
|---|---|
| `health of %player%`, `%player%'s health` | number |
| `max health of %player%`, `%player%'s max health` | number |
| `food level of %player%`, `hunger of %player%` (and `'s` forms) | number |
| `saturation of %player%`, `%player%'s saturation` | number |

```
if food level of player < 6:
    send "You are hungry."
```

## Changing

Every one of these works for health, max health, food level (or `hunger`)
and saturation. The `'s` form works too, like `set player's health to 5`.

```
set health of player to 10
add 4 to health of player
remove 2 from food level of player
set max health of player to 40
set saturation of player to 5
```

Lowering max health also lowers current health if needed.

## Effects

| Pattern | What it does |
|---|---|
| `heal %player%` | Full health. |
| `heal %player% by %number%` | Adds health, up to the max. |
| `damage %player% by %number%` | Hurts the player (generic damage). |
| `kill %player%` | Kills the player. |
| `feed %player%` | Food level and saturation to 20. |
| `feed %player% by %number%` | Adds to the food level. |

## Not supported

- Healing or damage reasons and causes.
- Changing the damage in `on damage`.
- Fractional food level (it is rounded).
