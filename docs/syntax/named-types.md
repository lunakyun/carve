# Types: named values, colors and more

Named value types, colors and more. Every value can be stored in a variable,
compared with `is`, printed in text and used as a function parameter or return type.

## Named values

About sixty types are a fixed list of names. A value prints as its name with spaces (`shift left`) and
is saved with its type and name.

```
on inventory click:
    if click type is shift left:
        cancel event
```

Three ways to get a value:

- **Plain words.** `left`, `full moon`, `waxing crescent`, `bite`, `sharpness`. The words are tried last,
  after every other meaning (items, variables, event values). When two types share a word (`left` is a
  click type and a text alignment) the first type in the table below wins.
- **From text.** `"shift_right" parsed as click type` gives the value, or `<none>` when the name does not
  exist. Use this for the types whose words are common elsewhere (see the notes column) and when you need a
  specific type.
- **From an event.** Values such as `click type` come as text; text equal to a name compares equal to the
  value, so `click type is left` works either way.

A value equals text with the same name, ignoring case, spaces and `_`, and an optional `minecraft:` prefix.
Two values of different types are equal when their names match.

```
function describe(c: click type) :: text:
    return "click: %{_c}%"

on join:
    send describe(shift left)
    send describe("right")          # text is accepted and compared by name
```

A parameter type in plural (`click types`) is a list.

| Type | Values |
|---|---|
| attribute type | the 40 Pumpkin attributes (`armor`, `attack_damage`, ...) |
| banner pattern type | `base`, `stripe_top`, `creeper`, ... |
| boss bar style | `solid`, `segmented 6`, `segmented 10`, `segmented 12`, `segmented 20` |
| cat type, chicken variant, cow variant, frog variant, pig variant, wolf variant, zombie nautilus variant | the variants |
| click type | `left`, `right`, `shift left`, `shift right`, `middle`, `drop`, `control drop`, `double click`, `number key`, ... |
| damage cause | the Bukkit style causes (`fall`, `fire tick`, `entity attack`, ...) |
| damage type | the 51 Pumpkin damage types |
| difficulty, weather type, world environment, direction | no plain words (they stay text elsewhere); use `parsed as` |
| display billboard, display text alignment, item display transforms | `fixed`, `center`, `left`, `gui`, ... |
| enchantment, potion effect type, villager profession, entity effect, item component | from the Pumpkin API lists |
| entity type, particle, visual effect | no plain words (zombie, flame, ... are entities or particle names); use `parsed as` |
| converging, directional and scalable particle effect | subsets of the particles |
| equipment slot, firework type, fishing state, game effect, gene, input key | fixed names |
| inventory action, inventory close reason, inventory type | fixed names |
| item flag, moon phase, quit reason, resource pack state, respawn reason, sound category | fixed names |
| potion effect event action, potion effect event cause, potion effect type category | fixed names |
| spawn reason, teleport cause, teleport flag, transform reason, unleash reason | fixed names |
| tree type, villager career change reason, villager type | fixed names |

The word `none`, `unknown`, `default`, `custom`, `plugin`, `command`, `normal` and `player` are never read as
a value on their own; use `parsed as`.

## Colors

```
rgb(255, 128, 0)                 # 0 to 255 per channel, clamped
red   light blue   lime          # the 16 dye colors, as plain words
color from hex "#ff8000"        
"#ff8000" parsed as color
hex of {_c}                      # "#ff8000" as text
```

A color prints as `#rrggbb`, so `red value of {_c}`, `green value of`, `blue value of` work on it.

## Quaternion, timeperiod, chunk, queue, world border

```
quaternion(1, 0, 0, 0)           # w, x, y, z       also: identity quaternion
timeperiod(0, 12000)             # 6:00 to 18:00    also: time period(a, b)
chunk(2, -3)                     # chunk x, z       also: chunk(2, -3, world "w")
chunk of player's location       # also: chunk at {loc}
queue(1, 2, 3)                   # not saved
world border of world "world"
```

Not here: operations on these values (rotating by a quaternion, `de-queue`, border center and damage)
belong to the expression pages and are listed in the parity checklist.
