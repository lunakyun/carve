# Op, kick, ban and names

Admin effects, conditions and expressions. Targets can be a player or a
text with a name. Names work for players who are offline, except for kick and
the name effects, which do nothing when the player is not online.

## Effects

```
op player
deop "Alex"
kick player
kick player due to "bad vibes"
ban player
ban "Alex" because of "cheating"
ban player by ip
ban ip "Alex" for "bots"
unban "Alex"
pardon player
unban ip "Alex"
set display name of player to "&6Boss"
set player's nickname to "Bo"
set tab list name of player to "&a[VIP] Steve"
reset display name of player
reset player's tab list name
```

The reason can start with `due to`, `because`, `because of`, `by reason of`,
`on account of`, `for` or `with reason`.

Op, kick and ban run the server's own `op`, `deop`, `kick`, `ban`, `pardon`,
`ban-ip` and `pardon-ip` commands as the console. They are queued and run
right after the script, like `execute console command`. Display and tab list
names are set directly. `&` color codes work in names.

## Conditions

```
player is op
player is not op
player isn't an operator
"Alex" is banned
"Alex" is not banned
```

`is op` is true when the player's permission level is above zero.

`is banned` is blocked for now: Pumpkin gives plugins no way to read the ban
list, so the condition parses but is always false.

## Expressions

```
display name of player      (also: nickname of player, player's display name)
tab list name of player     (also: player's tab list name)
ping of player              (a number, in milliseconds)
ip of player                (also: ip address of player, player's ip)
```

Both name expressions fall back to the player's account name. They give no
value when the player is offline.
