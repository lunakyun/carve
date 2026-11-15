# Triggers

A trigger says when code runs. Every script file is a list of triggers. A
trigger line has no indent and ends with `:`. The code under it is indented.

```
on join:
    send "Welcome!"
```

## Events

| Trigger | Also written as | Runs when |
|---|---|---|
| `on join:` | `on player join:`, `on login:` | a player joins |
| `on quit:` | `on player quit:`, `on leave:`, `on disconnect:` | a player leaves |
| `on chat:` | `on player chat:` | a player sends a chat message |
| `on break:` | `on block break:`, `on mine:` | a block is broken |
| `on place:` | `on block place:` | a player places a block |
| `on death:` | `on death of player:`, `on death of zombie:`, `on player death:` | a player or mob dies |
| `on damage:` | `on damage of player:`, `on damage of zombie:`, `on player damage:` | a player or mob takes damage |
| `on respawn:` | `on player respawn:` | a player respawns |
| `on right click:` | `on rightclick:` | a player right-clicks (air or a block) |
| `on left click:` | `on leftclick:` | a player left-clicks (air or a block) |
| `on click:` | | either click |
| `on sneak toggle:` | `on toggle sneak:` | a player starts or stops sneaking |
| `on sprint toggle:` | `on toggle sprint:` | a player starts or stops sprinting |
| `on flight toggle:` | `on toggle flight:` | a player starts or stops flying |
| `on load:` | `on script load:`, `on server start:` | scripts were loaded or reloaded |
| `every 5 seconds:` | `every 20 ticks:`, `every 2 minutes:`, `every second:` | on a timer |

Notes:

- Every event has `player`. A block break that no player caused has no
  `player` (it is `<none>`).
- `on chat:` also has `message`, the chat text. Using `message` in any other
  trigger is a load error.
- `cancel event` works before the first `wait`. Chat, break, place, damage,
  death and clicks are stopped. What a cancelled join or quit does is up to
  Pumpkin. Cancelling `on respawn`, `on load` or `every` is a load error.
- `on load` and `every` have no `player`; using it there is a load error.
- You can have many triggers for the same event, in one file or many. They
  run in file order (files sorted by name). If any of them cancels the event,
  it is cancelled.

### `on join:`

```
on join:
    send title "Welcome, %player%!" to player
```

### `on quit:`

```
on quit:
    broadcast "%player% left the server"
```

Pumpkin fires this event after the player is already gone. Carve still knows
the player's name until the first `wait`, so `%player%` works. Effects that
target the player (like `send ... to player`) do nothing, because the player
can't receive them anymore.

`quit message` is the message everyone sees. Change it with
`set quit message to "&e%player% left"` (before the first `wait`). In
`on join`, `join message` works the same way.

### `on chat:`

```
on chat:
    if message is "help":
        send "Try /rules" to player
```

### `on break:`

```
on break:
    if player does not have permission "builder":
        cancel event
        send "You can't break blocks here." to player
```

### `on place:`

```
on place:
    add 1 to {placed::%player%}
```

## Commands

```
command /name <arg> [<arg>]:
    permission: node
    description: text
    trigger:
        code
```

```
command /greet <player> [<text>]:
    permission: greet
    description: Say hi to someone
    trigger:
        if arg-2 is set:
            send "%player% says: %arg-2%" to arg-1
        else:
            send "%player% says hi!" to arg-1
```

- `permission:` and `description:` are optional. `trigger:` is required.
- The name can use letters, digits, `_` and `-`. It is lowercased.
- Two commands with the same name are a load error.
- New commands need a server restart. See [commands](../commands.md).

Arguments, permissions and everything else about commands is on the
[commands](../commands.md) page.

## Damage and death

```
on damage of player:
    if attacker is a player:
        send "%attacker% hit you for %damage%" to victim
    if damage > 10:
        set damage to 10

on damage of zombie:
    if attacker is a player:
        add 2 to damage

on death:
    if attacker is a zombie:
        set death message to "&c%player% was eaten"
```

| Expression | Where | Meaning |
|---|---|---|
| `damage` | `on damage` | Damage of this hit, a number (2 = one heart). |
| `attacker` | `on damage`, `on death` | The attacking player or entity, `<none>` for falls, fire and the like. |
| `victim` | `on damage`, `on death` | The hurt player or entity. |

`on damage` runs for players and mobs. `on damage of <type>` (`of player`,
`of a zombie`, `of skeletons`) runs only for that victim type. With a mob
victim, `player` is `<none>`; use `victim`.

`attacker` and `victim` are entities when they aren't players: `attacker is a
zombie`, `type of attacker` and `name of attacker` (the custom name, else the
type) work.

| Effect | Where | Meaning |
|---|---|---|
| `set damage to 4` | `on damage` | Change the damage of this hit. |
| `add 2 to damage`, `remove 1 from damage` | `on damage` | Raise or lower it (not below 0). |
| `set death message to "..."` | `on death` | Replace the death message. Colors work. |

The damage and death message reach the server only when set before the first
`wait`, like `cancel event`.

`on death` runs for players and mobs, like Skript. `on death of <type>`
(`of player`, `of zombie`) runs only for that type. With a mob victim, `player`
is `<none>`; use `victim`. `death message` is for players only.

```
on death of zombie:
    if attacker is a player:
        send "You killed %name of victim%" to attacker
```

In `on death`, `attacker` is the last player or entity that hit the victim in
the 5 seconds before the death, else `<none>`.

Cancelling `on death` stops Pumpkin's death handling, but the player keeps 0
health. Heal them in the same trigger.

## Clicks

```
on right click with blaze rod:
    send "Whoosh!" to player

on right click on barrel:
    cancel event
    send "This barrel is locked." to player

on left click with stick on stone:
    send "You poked %clicked block%." to player
```

- `with <item>` checks the item in the player's main hand.
- `on <block>` checks the clicked block.
- Both can be used, in any order. They are checked before the first line runs.
- `clicked block` (also `event-block`) is the block type as an item value
  (`air` when clicking air). It compares with item names:
  `if clicked block is chest:`.

## `on load`

Runs after the server loads the scripts, and after every successful
`/carve reload`. Good for setting defaults:

```
on load:
    if {spawn} is not set:
        set {spawn} to location(0, 64, 0)
```

## `every`

```
every 5 minutes:
    broadcast "&eDon't forget to vote!"

every 1 second:
    loop all players:
        send action bar "&7Coins: %{coins::%uuid of loop-player%}%" to loop-player
```

The timer counts server ticks from when the plugin loaded (20 ticks = 1
second). A `wait` inside `every` doesn't delay the next round; rounds can
overlap if the wait is longer than the period.

More events (server list ping, sign change, vehicles, worlds, weather, smelting and
more) are in [w2-events-4.md](w2-events-4.md).
