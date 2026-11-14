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
- There is no `and` or `or` yet. Nest `if` blocks instead.

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
