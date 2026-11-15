# Loops

A loop runs its indented block more than once. Every loop counts towards the
runaway guard (100,000 instructions without a `wait`), so a loop that never
ends is stopped with an error instead of freezing the server. A `wait` inside
a loop pauses the whole run, loop included, and the loop carries on after it.

## `loop %number% times:`

```
loop 3 times:
    send "Round %loop-number%"
```

`loop-number` is 1, 2, 3, and so on. Zero or a negative count runs nothing.

## `loop {list::*}:`

```
loop {warps::*}:
    send "%loop-index%: %loop-value%"
```

This goes over the direct items of a list. Number indexes come in number order
(1, 2, 10), then text indexes in alphabetical order. Nested items like
`{warps::spawn::x}` are skipped. The loop works on a copy, so changing the list
inside the loop doesn't confuse it.

- `loop-value` is the item.
- `loop-index` is its index, as text.

`for each {_x} in {warps::*}:` is the same loop, and it also sets `{_x}` to the
value on every round. `for each value in {list::*}:` (a type word instead of a
variable) is a plain `loop`.

## `loop all players:`

```
loop all players:
    send "Server restarts in 1 minute" to loop-player
```

`loop-player` is each player online when the loop starts. If a player leaves
while the loop waits, effects on them are skipped.

## `loop %object%, %object% and %object%:`

```
loop "red", "green" and "blue":
    broadcast loop-value
```

## `while %condition%:`

```
set {_i} to 0
while {_i} < 5:
    add 1 to {_i}
```

The condition is checked before every round. A `while` loop has no
`loop-value`.

## Loop values

| Expression | Meaning |
|---|---|
| `loop-value` | Item of the innermost loop |
| `loop-value-1`, `loop-value-2` | Item of the 1st (outermost), 2nd loop |
| `loop-player` | Player of the innermost `loop all players` |
| `loop-index` | Index of the innermost list loop |
| `loop-number` | Count of the innermost `loop N times` |
| `loop-iteration` | Round number (1, 2, ...) of the innermost loop |

## Leaving a loop

- `exit loop` (also `stop loop` or `break`) leaves the innermost loop.
- `continue` skips to its next round.
- `stop` ends the whole trigger.

## Lists

These go with loops:

- `add %object% to {list::*}` and `remove %object% from {list::*}` add and
  remove items (see [effects](effects.md)).
- `size of {list::*}` (also `amount of` or `count of`) is the number of items.
- `{list::*}` inside text prints as `a, b and c`.
- `{list::*} is set` is true when the list has at least one item.
