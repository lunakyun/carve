# Numbers, locations, player list, data (wave 2, expressions 5)

## Number constants

| Syntax | Value |
| --- | --- |
| `pi` | 3.14159... |
| `nan` | not a number |
| `infinity`, `positive infinity`, `negative infinity` | infinite values |
| `maximum integer value`, `minimum integer value` | 2147483647 and -2147483648 |
| `maximum long value`, `minimum long value` | about 9.22e18. Numbers are doubles, so the last digits are lost. |
| `maximum float value`, `minimum float value` | largest 32 bit float, and its negative |
| `maximum double value`, `minimum double value` | largest double, and its negative |

`max` and `min` work in place of `maximum` and `minimum`.

## Numbers from a to b

```
set {_l::*} to numbers from 1 to 5
loop integers from 1 to 10:
    add loop-value to {_sum}
```

- `numbers from a to b`: steps of 1 starting at `a`. Counts down when `a` is bigger than `b`.
- `integers from a to b`: only whole numbers inside the range.
- `between a and b` also works. At most one million numbers.

## Percent of

```
set {_tax} to 20% of {_price}
set {_half} to 50 percent of 80
```

`x% of y` is `x / 100 * y`. Use it outside of `"%...%"` text, or put it in a variable first.

## Text counting

`number of uppercase letters in <text>`, also `lowercase letters`, `digits`, `spaces`.

## Locations

- `midpoint of <a> and <b>`: halfway between two locations (or players). Errors when the worlds differ.
- `middle of <location>`: the centre of the block that holds the location.

## Items

- `max stack size of <item>`: 64, 16 or 1. This uses a built-in table of vanilla items.
- `plain <item>`: the item without extra data. Items have no extra data yet, so it gives the same item.

## Moon phase

`moon phase of <world | location | player>` gives text: `full moon`, `waning gibbous`, `last quarter`, `waning crescent`, `new moon`, `waxing crescent`, `first quarter` or `waxing gibbous`. It counts days from the world time.

## Player list (tab list) name

```
set player list name of player to "&a[VIP] &f%player%"
set player's tab list name to "Steve"
reset player list name of player
send player's player list name
set player's player list priority to 5
```

Without a custom name, reading gives the player's name. The priority can only be set.

## Metadata and persistent data

```
set metadata value "mood" of player to "happy"
send metadata value "mood" of player
delete metadata value "mood" of player

set persistent data value "rank" of player to 7
send persistent data value "rank" of player
```

- Keys are not case sensitive. Targets are players, worlds and item types.
- Metadata lives in memory and is gone after a restart.
- Persistent data is saved with the script variables. It is not stored inside the entity.
- Values are text, numbers, booleans, players and so on, but not lists.

## Parse error

```
set {_n} to {_text} parsed as number
if {_n} is not set:
    send parse error
```

`parse error` is the message of the last failed `parsed as number`, like `'abc' is not a number`.
