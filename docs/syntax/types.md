# Types: items, locations, worlds

Values in Carve have a type. Most of the time you don't need to think about
it: text accepts anything, and variables can hold any type. Effects and
function parameters check the type when the script loads.

| Type | Examples | Prints as |
|---|---|---|
| number | `5`, `-2.5`, `{_n} * 2` | `5`, `-2.5` |
| text | `"hi %player%"` | the text |
| boolean | `true`, `false`, `yes`, `no` | `true` |
| player | `player`, `loop-player`, `arg-1` | the name |
| item | `diamond`, `2 diamonds`, `diamond sword` | `2 diamond` |
| location | `location(0, 64, 0)`, `location of player` | `x: 0, y: 64, z: 0 in 'world'` |
| world | `world "world_nether"`, `world of player` | the world name |

All of them can be stored in variables and are saved with the variables.

## Items

Write the Minecraft id with spaces instead of `_`: `diamond sword`,
`oak log`, `golden apple`. Plurals work too (`diamonds`, `torches`). A number
in front is the amount: `16 oak logs`.

- **Checked at load.** Carve knows every item and block id of the pinned
  Pumpkin version, so a typo like `diamondz` is a load error.
- **Comparing.** Items compare by type, not amount. `tool of player is diamond`
  is also true when the player holds 5 diamonds.
- **Text.** An item equals text with its id: `diamond sword`,
  `"diamond_sword"` and `"minecraft:diamond_sword"` all match.
- **Not yet supported:** names, lore, enchantments and other item data. See
  [limits](../limits.md).

| Expression | Type |
|---|---|
| `tool of %player%`, `%player%'s tool`, `held item of %player%` | item in the main hand (`air` when empty) |

## Locations

| Expression | Notes |
|---|---|
| `location(x, y, z)` | No world: when used for a teleport, the player stays in their world. |
| `location(x, y, z, world "name")` | With a world. |
| `location of %player%`, `%player%'s location` | Includes the world. |
| `x of %location%`, `y of ...`, `z of ...` | Also `x-coordinate of`. Works on a player too: `y of player`. |
| `world of %location%` | `<none>` when the location has no world. |
| `distance between %location% and %location%` | A number. A player works as a location. |

## Worlds

| Expression | Notes |
|---|---|
| `world "name"`, `world("name")` | A world by name, like `world "world_nether"`. |
| `entity("uuid")`, `entity from uuid "..."` | A player or any live entity with that UUID, or `<none>`. |
| `world of %player%`, `%player%'s world` | The world the player is in. |

A world equals text with its name: `if world of player is "world":`.

## Player properties

| Expression | Type |
|---|---|
| `name of %player%`, `%player%'s name` | text |
| `uuid of %player%`, `%player%'s uuid` | text. Good for variable names: `{coins::%uuid of player%}`. |
| `health of %player%`, `%player%'s health` | number |
| `level of %player%`, `%player%'s level` | number |
| `xp of %player%`, `experience of %player%`, `total experience of %player%` | number (total points) |
| `level progress of %player%`, `%player%'s level progress` | number from 0 to 1 |
| `max health`, `food level`, `saturation` of a player | number. See [health and food](health-food.md). |

## In functions

Parameters and return types can be `item`, `location` or `world`:

```
function home(p: player) :: location:
    return {home::%uuid of {_p}%}
```
