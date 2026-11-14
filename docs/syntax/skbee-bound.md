# Bounds (SkBee style)

A bound is a named box in a world. It has owners, members and flags. Carve
saves bounds to `bounds.json` in the plugin data folder, so they stay after a
restart.

A bound is held as its id (text). `set {_b} to bound at player's location`
stores the id, like `spawn`.

Ids are not case sensitive. Corners are block positions, and both corners are
inside the bound. The bound `(0,60,0)` to `(10,70,10)` holds the blocks from 0
to 10 on x and z, so x = 10.9 is still inside. A location with no world means
the world named `world`.

## Create and delete

```
create bound with id "spawn" within location(0, 60, 0) and location(10, 70, 10)
create a new bound with id "arena" between {_a} and {_b}
create temporary bound with id "tmp" within {_a} and {_b}
delete bound "spawn"
```

- The corners can come in any order.
- Creating a bound with an id that exists replaces it.
- A temporary bound is not saved.
- Deleting or changing a bound that does not exist writes a warning to the log.

## Find bounds

```
set {_b} to bound at player's location
set {_b} to bound at location(5, 64, 5)
set {_b} to bound "spawn"
set {_b} to bound with id "spawn"
loop bounds at player's location:
loop all bounds:
```

`bound "x"` and `bound at ...` give `<none>` when nothing matches. Use
`if {_b} is set:` to test. `bound at` gives the first bound made when several
overlap. `bounds at` gives all of them.

## Conditions

```
if player is within bound "spawn":
if location(1, 64, 1) is not within bound "spawn":
if bound "spawn" exists:
if bound at player's location does not exist:
if player is an owner of bound "spawn":
if player is not a member of bound "spawn":
if bound "spawn" has flag "pvp":
if bound "spawn" does not have flag "pvp":
```

`in` and `inside` work like `within`.

## Owners and members

```
add player to owners of bound "spawn"
add player to members of bound "spawn"
remove player from owners of bound "spawn"
set owner of bound "spawn" to player
loop owners of bound "spawn":
set {_m::*} to members of bound "spawn"
```

- Only online players can be added. They are saved with their UUID and name.
- `set owner ... to` replaces the owners with that one player.
- A player is added once, never twice.

## Flags

A flag is a yes/no value with a name. Carve does not give flags a meaning on
its own. Your scripts read them.

```
set flag "pvp" of bound "spawn" to true
delete flag "pvp" of bound "spawn"
if flag "pvp" of bound "spawn" is true:
```

A flag that is not set reads as `false`.

## Other values

```
lesser corner of bound "spawn"
greater corner of bound "spawn"
world of bound "spawn"
volume of bound "spawn"
set lesser corner of bound "spawn" to location(0, 50, 0)
set greater corner of bound "spawn" to {_loc}
```

## Events

```
on bound enter:
    send "You entered %event-bound%" to player

on bound exit:
    send "You left %event-bound%" to player
```

- `player` is the player who moved. `event-bound` is the id of the bound.
- Also accepted: `on player enter bound`, `on player exit of bound`, `on bound leave`.
- If a player stands inside a bound when Carve first sees them (for example
  when they join), that counts as entering.
- If a bound is deleted, players inside it get no exit event.
- You cannot cancel these events.
- To react to one bound only, test it: `if event-bound is "spawn":`.

### How the check works

Carve looks at every online player every 2 ticks, but only while the bound
list is not empty and some script has a bound event. A player is looked at
again only when the block under them changes. A bound lookup reads one cell of
a grid index per world, so it does not slow down when you have many bounds.

A host that has its own move event can call `Engine::bounds_moved` instead.

## Not included

- Blocks or entities inside a bound (`blocks in bound`, `entities in bound`).
- Custom bound values besides yes/no flags.
- Bound events for entities other than players.
- Filtering `on bound enter` by id in the header.
