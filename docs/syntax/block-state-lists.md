# Block properties, list parts, queues and more

## Block properties

```
send "%honey level of {_hive}% of %max honey level of {_hive}%"
set honey level of {_hive} to 5
add 1 to brushing stage of {_sand}
reset age of {_crop}
set age of {_crop} to max age of {_crop}
```

- `[max] [beehive|bee nest] honey level of <block>`: 0 to 5.
- `[max] brushing stage|progress of <block>`: the `dusted` property, 0 to 3.
- `[max] age of <block>`: the `age` property of crops and similar blocks. Blocks only, no entities yet.

All can be set, added to, removed from and reset (to 0). Results are clamped to
the block's range. A block that doesn't have the property gives `<none>` and is
left alone.

## Parts of a list

```
set {top::*} to 5, 4, 3, 2, 1
send "%first 2 elements of {top::*}%"
send "%elements 2 to 4 of {top::*}%"
loop last 3 elements of {top::*}:
```

`first N elements of`, `last N elements of` and `elements N to M of` work on
list variables. Asking for more than the list holds gives what there is.

## Queues

```
set {_q} to a new queue of "a", "b", "c"
set {_first} to dequeue {_q}
set {_two::*} to de-queue first 2 elements of {_q}
```

`dequeue` removes from the queue variable and gives the value (or a list for
`N elements`). An empty queue gives `<none>`.

## Banner pattern items and persistent data keys

```
give 1 of banner pattern item of "creeper" to player
send "%persistent data keys of player%"
```

The banner pattern item exists for the ten patterns that have one. The keys
are those set with `set persistent data value`.
