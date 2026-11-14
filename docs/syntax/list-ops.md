# List operations

Lists are variables like `{x::*}`. These forms work on whole lists.

## Making a list

```
set {_n::*} to 10, 2, 33 and 4
set {_all::*} to {a::*}, {b::*} and 7
```

`set {x::*} to ...` replaces the list. Items are numbered from 1.

## Sorted, reversed, shuffled

```
set {_s::*} to sorted {_n::*}
send "%reversed {_n::*}%"
loop shuffled {_n::*}:
    send loop-value
```

- Numbers sort by value, before text. Text sorts ignoring case.
- None of these change the original list.
- `sort {x::*}` sorts the list itself. Add `in descending order` to flip it.

## Filter with `where`

```
set {_big::*} to {_n::*} where [input > 2]
send "%size of ({_n::*} where [input is not 3])%"
loop {_words::*} where [input starts with "a"]:
    send loop-value
```

`input` is the item being tested. You can use:

- `> < >= <= = !=`, `is`, `is not`, `is greater than`, `is less than`, `is at least`, `is at most`
- `contains`, `does not contain`, `starts with`, `ends with`
- `is set`
- `and` and `or` to join tests (`and` binds first)

Limits: `input is 1 or 2` does not work, write `input is 1 or input is 2`.
`input` is not available inside quoted text.

## Random elements

```
set {_pick::*} to random 3 elements of {_n::*}
```

The elements are different items. Asking for more than the list has gives the
whole list, in random order. `random element of {x::*}` still gives one item.

## Numbers from a list

| Syntax | Result |
|---|---|
| `sum of {x::*}` | total (0 when empty) |
| `average of {x::*}` | mean, `<none>` when empty |
| `min of {x::*}` | smallest, `<none>` when empty |
| `max of {x::*}` | largest, `<none>` when empty |
| `size of ({x::*} where [input > 2])` | how many match |

Items that are not numbers are skipped. `sum of {x::*} where [input > 5]` works
too, and `sum of {x::*} + 1` adds 1 to the sum.

## Lists in functions

Use a plural type for a list parameter or result:
`numbers`, `texts`, `players`, `items`, `locations`, `worlds`, `booleans` or `objects`.
The element type is not checked.

```
function evens(nums: numbers) :: numbers:
    return {_nums::*} where [mod(input, 2) = 0]

on join:
    set {_x::*} to 5, 8, 2, 9
    set {_e::*} to evens({_x::*})
    send "%sum of evens({_x::*})%"
```

Inside the function the list is `{_nums::*}`. It is a copy, so changing it does
not change the caller's list. Results of list functions work in `set`, `loop`,
`sum of` and the other forms above.

Not supported: `sorted ... by <expression>`, `first N elements of`, and
list values stored in a single variable.
