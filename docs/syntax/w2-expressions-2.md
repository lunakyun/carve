# Characters, differences, directions and event values

Small expressions added in wave 2.

## Character codes

```
send codepoint of "A"                   # 65
send unicode of "a"                     # 97
send character from codepoint 65        # A
set {_c} to char from code point 100 + 1
```

- `codepoint of x` uses the first character of the text. Empty text gives `<none>`.
- `character from codepoint n` gives `<none>` for negative numbers, fractions and
  invalid code points.

## Characters between

```
loop characters between "a" and "e":
    send loop-value
set {_digits::*} to alphanumeric characters between "0" and "z"
```

- Counts up or down, both ends included.
- `alphanumeric` keeps only letters and digits.
- Only the first character of each end is used. The list is capped at 65,536 items.

## Difference

```
send difference between 3 and 10        # 7
send difference between (1 minute) and (90 seconds)   # 30 seconds
```

Works on numbers (absolute value), dates and timespans. Put timespans in brackets,
because `1 minute and 90 seconds` reads as one timespan.

## Directions

```
set {_up} to 3 blocks above player's location
set {_n} to 2 blocks north of {_loc}
```

Words: `above`, `below`, `north` (-z), `south` (+z), `east` (+x), `west` (-x).
`block`, `blocks`, `meter` and `meters` all work. This gives a location. It is
not a full direction type.

## Chunks

```
send chunk x of player's location
send chunk z of player's location
```

Numbers only (block coordinate divided by 16, rounded down).

## Compass target

```
set compass target of player to {spawn}
send compass target of player
reset player's compass target           # points at 0, 0, 0
```

## Event values

| Expression | Where | Notes |
| --- | --- | --- |
| `death message` | `on death` | Settable: `set death message to "..."`. |
| `damage cause` | `on damage` | Read only. Text such as `fall` or `player_attack`. |
| `chat recipients` | `on chat` | A list of players. `size of chat recipients` works. |

## Not possible

Chat format, damage source values, display entity settings, loot contexts, queues
and command info need things the plugin API does not give us. See the status rows
in `docs/skript-parity.md`.
