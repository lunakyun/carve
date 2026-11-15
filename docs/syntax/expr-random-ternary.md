# Random values, ternary, reduce and a few host reads

Small expressions that did not fit the other pages.

## Numbers

```
set {_a} to pi
set {_b} to infinity
set {_c} to negative infinity
set {_d} to NaN
```

## Random character and UUID

```
set {_c} to a random character                          # any printable ASCII, no space
set {_c} to random alphanumeric character               # a-z, A-Z, 0-9
set {_c} to a random character between "a" and "f"      # a to f
set {_id} to random uuid                                # like 3f2b8c1e-...-4...-a...
```

## Raw string

`raw` keeps colour codes and tags from turning into colours.

```
send raw "&aHello"        # shows &aHello
send raw string of "<red>x"
```

It works by adding an invisible character after `&`, `§` and `<`. The result
does not compare equal to the plain text.

## Repeat

```
send "ab" repeated 3 times       # ababab
send "-" repeated {_n} + 1 times
```

The result can be at most 1,000,000 characters. A longer one is a runtime error.

## Ternary

```
set {_t} to "big" if {_x} > 3 else "small"
set {_y} to 1 if {_x} < 3 else 2 if {_x} < 6 else 3
```

`otherwise` works like `else`. The condition uses the same forms as a `where`
filter: comparisons (`>`, `<`, `=`, `is`, `is not`, `at least`...), `contains`,
`starts with`, `ends with`, `is set`, joined with `and` / `or`.
An `if` with no `else` after it is not a ternary.

## Reduce

```
set {_sum} to reduce {_l::*} with [reduced value + input]
set {_product} to {_l::*} reduced with [reduced value * input]
set {_text} to reduce {_names::*} with ["%reduced value%, %input%"]
```

`reduced value` starts as the first element. Then each other element is `input`
in turn, and the result of the bracket becomes the new `reduced value`.
An empty list gives `<none>`. A one-item list gives that item.

## Recursive lists

```
send recursive size of {t::*}        # counts values at every depth
loop recursive {t::*}:               # visits values at every depth
set {all::*} to recursive {t::*}
```

`size of {t::*}` counts only the direct items.

## Reading the server and players

| Syntax | Result |
|---|---|
| `remaining air of player`, `player's remaining air` | timespan (300 ticks is 15 seconds) |
| `respawn location of player` | location |
| `sea level of world "x"`, `sea level of player's world` | number |
| `simulation distance` | number of chunks |
| `tablisted players`, `tablisted players of player` | list of online players |
| `quit message` (only in `on quit`) | text |

Effects:

```
set remaining air of player to 5 seconds
add 2 seconds to player's remaining air
remove 10 ticks from remaining air of player
set respawn location of player to location(0, 70, 0)
```

Remaining air is clamped to the entity's maximum air.
`tablisted players` is every online player: Pumpkin cannot tell who is hidden
from a player's tab list.
`quit message` is read only, and is the plain text of the leave message.
