# Movement and command events

| Event | Names that work | Event values | Can cancel |
|---|---|---|---|
| Command | `on command`, `on command "home"`, `on player command` | `command`, `full command`, `player` | yes |
| First join | `on first join`, `on first login` | `player` | no |
| Move | `on move`, `on player move`, `on walk` | `event-location`, `past event-location`, `player` | yes |
| Swap hand items | `on swap hand items`, `on swap hands`, `on hand swap` | `player` | yes |
| Teleport | `on teleport`, `on player teleport` | `event-location`, `past event-location`, `player` | yes |
| World change | `on world change`, `on world switch` | `event-world`, `past event-world`, `player` | no |

## Command

`command` is the command name in lowercase without the slash. `full command`
is the whole text without the slash. `cancel event` stops the command.

```
on command:
    if command is "op":
        cancel event
        send "No." to player
```

`on command "home":` only runs for that command. A name with a space, such as
`on command "gamemode creative":`, compares the full command instead.

## First join

Carve stores the uuids of players it has seen in `carve.redb` in its data
folder. A player who joined before Carve was installed counts as new once.

## Move

`on move` runs only when the player enters another block. Turning the head or
moving inside a block does not count. `event-location` is where to and
`past event-location` is where from. Pumpkin calls the plugin for every move
packet, so Carve only listens when a script uses `on move` at startup. Adding
the first `on move` script needs a restart.

## Swap hand items

Runs when the player presses the swap key (F). `cancel event` keeps both
hands as they were.

## World change

`event-world` is the new world, `past event-world` the old one. The overworld
is `world`; the nether and the end are `minecraft:the_nether` and
`minecraft:the_end`.
