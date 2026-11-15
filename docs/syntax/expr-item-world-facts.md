# Item, world and color expressions

## Time and numbers

```
set {t} to an eternity          # also: eternity. A very long time span.
set {a} to 90 degrees           # angles are numbers in degrees
set {b} to 3.14159 radians      # converted to degrees (about 180)
send "%altitude of player%"     # the Y coordinate of a location or player
```

## Colors

Works on color text: `#rrggbb`, `&#rrggbb`, or a name like `gold` or `dark_blue`.

```
set {c} to "#ff8000"
send "%red value of {c}% %green value of {c}% %blue component of {c}%"
send "%alpha value of {c}%"    # always 255
```

Returns `<none>` when the text is not a color.

## Items and armor

```
send "%amount of diamonds in player's inventory%"
send "%number of diamond in inventory of player%"
if amount of iron ingots in player's inventory > 5:
    send "enough"

send "%helmet of player%"       # also chestplate, leggings, boots
if boots of player is not set:
    send "no boots"
```

Armor is read only for now.

## Lists

```
set {s::*} to alphabetically sorted {list::*}   # compares as text, ignores case
loop alphabetically sorted {list::*}:
    send loop-value
set {files::*} to all scripts                   # file names of loaded scripts
```

## World data

```
send "%biome at player%"                        # for example birch_forest
send "%biome of location(10, 70, 5, world "nether")%"
```

## Block entities

These read the block entity at a location. They give `<none>` when there is
no such block entity. Times are time spans.

| Syntax | Result |
|---|---|
| `beacon tier of {loc}` (or `beacon level of`) | pyramid levels, 0 to 4 |
| `beacon range of {loc}` | range in blocks |
| `brewing time of {loc}` (or `brewing stand time of`) | time span |
| `brewing stand fuel level of {loc}` | number |
| `burn time of {loc}` | time left on the fuel |
| `cook time of {loc}` | time spent on the current item |
| `total cook time of {loc}` | time the item needs |

## Not possible on Pumpkin

See the ⛔ rows in `docs/skript-parity.md` (arrows, banner patterns, block
hardness, active item, attack cooldown, and others) for the reason for each.
