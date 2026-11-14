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

Notes:

- Every event has `player`. A block break that no player caused has no
  `player` (it is `<none>`).
- `on chat:` also has `message`, the chat text. Using `message` in any other
  trigger is a load error.
- `cancel event` works in all five, before the first `wait`. Chat, break and
  place are stopped. What a cancelled join or quit does is up to Pumpkin.
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
