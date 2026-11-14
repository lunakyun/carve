# Conditions

Conditions go after `if` and `else if`. A block of lines under them runs only
when the condition is true.

```
if {coins::%player%} >= 10:
    send "You can buy this."
else if {coins::%player%} is set:
    send "You need 10 coins."
else:
    send "You have no coins yet."
```

Rules:

- `if`, `else if` and `else` must end with `:`, and the lines under them must
  be indented one more level.
- `else if` and `else` must come right after the `if` block (or another
  `else if` block) at the same indent.
- `if` blocks can be nested.
- Conditions can be combined with `and`, `or`, commas and brackets. See
  [combining conditions](#combining-conditions).
- `while` loops use the same conditions (see [loops](loops.md)).

Each heading below is the exact pattern Carve matches. See
[effects](effects.md) for how to read a pattern.

## Set or not set

### `%object% is set`

### `%object% is not set`

```
if {home::%player%} is set:
    send "You have a home."
if arg is not set:
    send "Usage: /greet <player>"
```

A value is "not set" when it is `<none>`: a variable that was never set, an
optional command argument that was left out, or `player` in an event that
has no player.

## Equal and not equal

### `%object% (is|=) %object%`

### `%object% (is not|!=) %object%`

```
if message is "hello":
    send "Hi!"
if {team::%player%} != "red":
    send "You are not on the red team."
if arg-1 is player:
    send "That's you!"
```

How values are compared:

- Text ignores case: `"Hello"` is `"hello"`.
- Numbers compare by value. Text that holds a number equals that number, so
  `"5"` is `5`.
- Two players are equal when they are the same player (same UUID).
- A player equals text that matches their name, ignoring case.
- Two values that are both not set are equal.
- Anything else is not equal.

## Comparing numbers

### `%number% > %number%`

### `%number% < %number%`

### `%number% >= %number%`

### `%number% <= %number%`

```
if {joins::%player%} > 10:
    send "You are a regular!"
if arg-1 <= 0:
    send "Pick a number above 0."
```

Text that holds a number works too. If either side is not a number (or not
set), the condition is false.

## Permissions

### `%player% has permission %text%`

### `%player% does not have permission %text%`

```
if player has permission "vip":
    send "Thanks for supporting the server!"
if player does not have permission "carve:builder":
    cancel event
```

A permission without a namespace gets Carve's namespace: `"vip"` checks
`carve:vip`. If the player is offline, both conditions are false. See
[commands](../commands.md#permissions) for how permissions are registered.

## Text

All text checks ignore upper and lower case, like Skript.

### `%text% contains %text%`

### `%text% (does not|doesn't) contain %text%`

```
if message contains "discord":
    send "Our Discord is on the website."
if {banned::*} contains name of player:
    kick player
```

With a list on the left (`{list::*}`), this checks whether one of the items
equals the text exactly (ignoring case), not whether it appears inside one.

### `%text% starts with %text%`

### `%text% (does not|doesn't) start with %text%`

### `%text% ends with %text%`

### `%text% (does not|doesn't) end with %text%`

```
if message starts with "!":
    cancel event
```

## Ranges

### `%number% is between %number% and %number%`

### `%number% is not between %number% and %number%`

```
if {level::%player%} is between 10 and 20:
    send "You can enter the mid-level arena."
```

Both ends count as inside. The two bounds can come in any order. Text that is
not a number is never between anything.

## Combining conditions

```
if {coins::%player%} >= 10 and player has permission "shop":
if {team::%player%} is "red" or {team::%player%} is "blue":
if {a} is set, {b} is set and {c} is set:
if ({x} > 1 or {y} > 1) and {z} is 0:
```

- `and` (or a comma list ending in `and`) needs every part to be true.
- `or` needs at least one part to be true.
- `and` binds tighter than `or`: `a or b and c` means `a or (b and c)`.
  Use brackets to say something else.
- Parts are checked left to right, and checking stops as soon as the answer is
  known.

### One value, several options

```
if {class::%player%} is "sniper" or "scout":
if {_n} is 1, 2 or 3:
if {_c} is not "red" or "blue":
```

`x is a or b` is true when x equals any option. `x is not a or b` is true when
x equals none of them. This is the Skript meaning.

