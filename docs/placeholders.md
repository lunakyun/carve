# Placeholders

Carve has PlaceholderAPI-style placeholders: `%player_name%`-style names that
turn into values. Scripts can read them, add their own, and other Pumpkin
plugins can ask Carve for them.

In a script string `%` starts an expression, so a literal percent sign is
`%%`. That is why placeholder text inside a script is written `%%name%%`.

## Read one

```
send placeholder "player_health" for player
set {_n} to placeholder "server_online"
```

The name has no percent signs. The player defaults to the event's player.
An unknown placeholder, or a player placeholder with no player, is `<none>`.

## Parse text

```
send parse placeholders in "Hi %%player_name%%, %%server_online%% online" for player
```

Every `%name%` that Carve knows is replaced. Anything else stays as written,
so text with a normal percent sign is safe. Text over 64 KiB is left alone.

## Built-in placeholders

| Placeholder | Value |
|---|---|
| `player_name`, `player_displayname`, `player_uuid` | name, display name, UUID |
| `player_health`, `player_level`, `player_gamemode` | health, XP level, game mode |
| `player_world`, `player_x`, `player_y`, `player_z` | world and block position |
| `player_ping` | latency in ms (when the host knows it) |
| `server_online`, `server_max_players` | players now, player limit |
| `server_time` | time of day on the server clock, `HH:MM:SS` UTC |
| `server_motd`, `server_version` | when the host knows them |
| `carve_var_<name>` | the global variable `{name}` |

Names are not case sensitive. Placeholders the host cannot answer stay as written.

## Your own placeholders

```
on placeholder request for prefix "shop":
    if the identifier is "coins":
        set the result to "%{coins::%player%} ? 0%"
```

`%shop_coins%` now works everywhere, including in `parse placeholders`.
`the identifier` is the text after `shop_`, `the prefix` is `shop`, and
`set the result to ...` answers. If nothing sets the result, the placeholder
stays as written. Without `for prefix`, the trigger sees every placeholder
that is not built in. Scripts may parse placeholders inside a request, up to
8 levels deep.

## Other plugins

Pumpkin's plugin API (WIT v0.1) has `ipc.send-ipc-message(recipient, bytes)` and
a plugin export `handle-ipc-message(sender, bytes)`. Carve implements the export, so
any plugin can call Carve (recipient id `carve`). There is no shared service registry,
so this is the only route, and the other plugin must send the message itself.
Carve cannot push placeholders into another plugin's chat or scoreboard.

The message is UTF-8 text with three lines:

```
carve:parse            (or carve:get)
<player uuid, or empty>
<text with %placeholders%>   (or the placeholder name for carve:get)
```

`carve:parse` answers with the parsed text. `carve:get` answers with the value,
or an error when the placeholder is unknown. The player must be online.
Carve answers on the server thread when it is not busy running a script;
if it is, the call fails with "Carve is busy, try again".

## Errors

A bad `placeholder` or `parse placeholders` line is reported like any other
Carve error and only that line is skipped.
