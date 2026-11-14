# Lists and list literals

A list is a variable with `::*`, like `{warps::*}`. A list literal is several
values written in a row, separated by `,`, `and` or `or`.

## Set, add and remove many values

```
set {colors::*} to "red", "green" and "blue"
add "pink" and "gold" to {colors::*}
remove "red", "gold" from {colors::*}
```

- `set` clears the list first, then stores the values at 1, 2, 3, and so on.
- `add` appends at the next free number index.
- `remove` removes every item equal to one of the values.
- A value can also be a whole list: `set {copy::*} to {colors::*}` and
  `add {a::*} and {b::*} to {all::*}`.
- The values are worked out before the list changes, so
  `set {x::*} to {x::*}, 5` keeps the old items.
- An item that is not set is skipped.
- An expression with its own `and` stays in one piece:
  `set {x::*} to 1, random integer between 1 and 6`.

## Size

```
send "%size of {x::*}%"
send "%amount of {x::*}%"
send "%number of {x::*}%"
```

All three give the number of direct items.

## Contains

```
if {x::*} contains "red":
if {x::*} does not contain "red":
```

Text is compared without caring about case.

## Indices

```
set {i::*} to indices of {x::*}
loop indices of {x::*}:
    send "%loop-value%"
send "%indices of {x::*}%"
```

The indices are text, in list order (numbers first, then names).

## First, last and Nth element

```
send "%first element of {x::*}%"
send "%last element of {x::*}%"
send "%element 3 of {x::*}%"
send "%{x::%{_n}%}%"
```

`element N` counts from 1 and goes by position in the list. The result is
`<none>` when the list is empty or N is too big. `{x::3}` and `{x::%n%}`
read the item with that exact index instead.

## Loop

```
loop 1, 2 and 3:
loop {a::*} and {b::*}:
loop indices of {x::*}:
```

## Delete

`delete {x::*}` removes the whole list, including nested items.

## Not supported yet

- `first N elements of`, `last N elements of`, and `%n%th element of`.
- Lists as function parameters or return values.
