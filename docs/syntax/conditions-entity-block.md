# Rare conditions

All of these work in `if`, `else if`, `while`, `and` and `or`, and have a negated form
(`is not` or `isn't`).

## Is valid

```
if player is valid:
if {_x} is not valid:
```

A player is valid while online. An unset variable is not valid. Anything else that has a value is valid.

## Script loaded

```
if script "shop" is loaded:
if script "shop.carve" isn't loaded:
```

Matches the file name of a loaded script. Case does not matter, and `.carve` is optional.

## Regex

```
if {_name} matches "[a-z]+\d*":
if {_name} doesn't match "\d+":
if {_text} contains regex "b[a-z]r":
```

`matches` needs the whole text to match. `contains regex` needs a match anywhere.
Write `\d` with a single backslash. Carve has no text escapes.

Supported: literals, `.`, classes (`[a-z]`, `[^0-9]`), `\d \w \s \D \W \S \b \B`, `^`, `$`,
groups `(...)` and `(?:...)`, `|`, and `* + ? {n} {n,} {n,m}` with a lazy `?` suffix.
Start the pattern with `(?i)` to ignore case. Not supported: captures, lookaround, back references.
A bad pattern is a runtime error. A pattern that takes too many steps is stopped with an error.

## Within

```
if player is within location(0, 0, 0) and location(10, 10, 10):
if player is within 5 blocks of location(0, 64, 0):
if player is within world "nether":
```

Corners can be given in any order. Both ends are included. A player stands for their position.
If both locations name a world and the worlds differ, the answer is no.
`blocks`, `meters` and `metres` all work, and `of`, `around` and `from` all work.

## Player states

```
if player is poisoned:
if player is silent:
if player is riding:
```

Players only for now. `riding` is true on any vehicle.

## PvP

```
if pvp is enabled:
if pvp is disabled in world "nether":
```

Reads the `pvp` game rule. Without a world, the first world is used.

## Potion effect type is instant

```
if "instant health" is instant:
```

Effect names are text. Instant effects are instant health, instant damage and saturation.

## Blocks

```
if player's tool is solid:
if player's tool is transparent:
if player's tool is passable:
```

Works on a block or item type. If the name is not a block, every form answers no.

## Not possible on Pumpkin

Is Slime Chunk (no world seed), Running Minecraft (no version API), Is Using Experimental Feature,
Is Pressing Key, Left Handed, Is Tagged and Is Ticking: the plugin API does not give this information.
