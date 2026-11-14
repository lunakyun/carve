# Functions and sections (wave 4)

## Built-in functions

Call them like script functions: `clamp(x, 0, 10)`. A script function with the
same name wins. Wrong argument counts or types are load errors with `file:line`.

| Function | Notes |
| --- | --- |
| `sin cos tan asin acos atan atan2` | degrees in and out |
| `exp ln log(x[, base]) root(x[, n])` | `log` is base 10 by default, `root` is square root by default |
| `factorial combinations(n, k) permutations(n, k)` | whole numbers |
| `clamp(x, min, max)` | runtime error when `min > max` |
| `median product concat` | take numbers or a list like `{x::*}` |
| `formatNumber(n, pattern)` | pattern like `#,##0.00` |
| `toBase(n, base) fromBase(text, base)` | base 2 to 36 |
| `isNaN(x)` | |
| `calcExperience(level)` | total xp points for a level |
| `rgb(r, g, b[, a])` | hex colour text |
| `uuid(text)` | parses a UUID |
| `player(name or uuid)`, `offlinePlayer(...)` | looks up a player |

Not available: `entity()` (no entity lookup by UUID in the plugin API),
`quaternion()` and `axisAngle()` (no quaternion type). `vector(x, y, z)` already existed.

## Sections

```
set {_l::*} to 1, 2, 3, 4, 5
filter {_l::*} to match:
    input > 2
    input < 5

spawn a zombie at player:
    set name of event-entity to "Bob"

suppress type hints:
    set {_x} to 1
```

- `filter` keeps elements where every condition holds. `input` is the element, `input index` its index. Conditions are single lines.
- `spawn ...:` spawns, then runs the body; `event-entity` is the new entity.
- `suppress type hints:` just runs its body.
- `shoot ...:` is refused with an error: Pumpkin cannot launch projectiles from plugins.

## Structures

- `auto reload:` is accepted and ignored; Carve reloads only on request.
