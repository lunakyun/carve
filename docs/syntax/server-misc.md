# Server info, whitelist and worlds

Server-wide values and a few effects: player counts, the MOTD, the version, the whitelist, worlds, spawn points and the world border.

## Expressions

| Syntax | Type | Notes |
| --- | --- | --- |
| `online players count`, `online player count`, `player count`, `amount of players`, `number of players`, `amount of online players` | number | Players on the server now. |
| `max players`, `maximum players`, `max player count`, `player limit` | number | The slot limit. Read only. |
| `motd`, `server motd`, `message of the day` | text | The server list text. Read only. |
| `server version`, `pumpkin version`, `version of the server` | text | The Pumpkin version. It is not the Minecraft version. |
| `all worlds`, `worlds` | list | Every world, main world first. |
| `spawn of <world>`, `spawn point of <world>`, `world spawn of <world>` | location | The shared spawn point, centered on its block. |
| `world border size [of <world>]`, `world border diameter`, `border size` | number | Current diameter. |

`<world>` is `world "name"`, `player's world`, or plain text with the world name. When you leave the world out, the main world is used.

```
on join:
    send "&7%online players count%/%max players% online"
    send "&7%motd%"
    send "&7Server %server version%"
    loop all worlds:
        send "world: %loop-value%"
    set {w::*} to all worlds
```

`amount of characters in x` and `max(a, b)` work as before.

## Whitelist

```
add player to the whitelist
remove player from the whitelist
whitelist "Alex"
unwhitelist "Alex"

if player is whitelisted:
    send "welcome back"
if "Alex" is not whitelisted:
    send "not on the list"
if player is on the whitelist:
    send "listed"
```

Targets can be a player or a name. Online players are added by UUID. An offline name runs the `whitelist` console command, which looks the profile up.

## Broadcast to one world

```
broadcast "&eEvent starts!" to world "arena"
broadcast "&7Only for this world" in player's world
```

A plain `broadcast "text"` still goes to everyone. The `to`/`in` part is only used when what follows it is a world.

## World border

```
set world border size to 500
set world border size of world "nether" to 200 over 30 seconds
set the world border diameter to {size}
send "%world border size%"
```

The size is clamped between 1 and 59,999,968. The time after `over` (also `in` or `during`) is at most one day. Without it the border changes at once.

## Teleport to spawn

```
teleport player to spawn
teleport player to the spawn of world "nether"
teleport player to spawn of world "nether"
```

Without a world, the player goes to the spawn of the world they are in. `teleport player to spawn of world "x"` also works with the normal location teleport.

## Event

```
on server list ping:
    set {pings} to {pings} + 1
```

Runs each time a client asks for the server list. There is no player, and the script cannot change the MOTD or the counts shown.

## What does not work

- Changing the MOTD, the max players or the spawn point.
- Whitelist on/off and the list of whitelisted players.
- The Minecraft version (the API only gives the Pumpkin version).
- Expanding or shrinking the border, its center, damage and warnings.
- On a host without the world API, `set world border size of world "x"` only works for the main world and warns for any other.
