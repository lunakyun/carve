# Syntax from popular scripts

These are the core Skript lines that public scripts use a lot. Command
argument types are on the [commands page](../commands.md).

## Messages and commands

```
message "&aHello %player%"
message "log line" to console
send "Server restart soon" to all players
message "Server restart soon" to all players

player command "/home"
execute player command "warp spawn"
make player run command "back"
console command "/say hello"
execute console command "say hello"
command "/list" by console
```

- `message` is `send`. `to all players` sends it to everyone online and also
  writes it to the server log.
- `player command "x"` runs as the player who started the trigger, so their
  permissions apply. A leading `/` is ignored everywhere.

## Leaving a section

```
exit trigger          # same as stop
exit loop
exit 2 loops
exit 2 sections       # the innermost two if or loop sections
exit all sections     # the whole trigger
```

`exit N sections` counts `if`, `else if`, `else` and loop sections. `exit N
loops` counts loops only. Other sections (such as `catch`) are not counted.

A line that is only a condition stops the trigger when it is false, and a
section header that is only a condition works like `if`:

```
{cooldown} was less than a minute ago:
    message "Wait a bit"
    stop
```

## Conditions

```
{date} was less than 5 minutes ago
{date} was more than a day ago
difference between now and {date} is less than 20 seconds
player doesn't have permission "x.y"
player have permission "x.y"
{a} and {b} are set
{a} and {b} are not set
arg-1 not set
{list::*} do not contain arg-1
{text} contains "a" or "b"
loop-value is a location        # number, text, boolean, entity, location
world is "world"                # the world of the player
player's world is "world" or "world_nether"
player is holding anything
victim is not warden or wither or ender dragon
event-block is not grass or tall grass or sugar cane
```

- `was less than N ago` and the `is less than` word forms work on dates and
  timespans (`a minute`, `an hour` count as one).
- `world` is the player's world. In block events `event-world` is too.
- `x is not a or b or c` checks each name, so every one must fail.

## Expressions

```
random integer from 1 to 10
{n} mod 3
rounded player's x-coordinate
player's y-coordinate
"world"'s time
loop-entity's type
name of current inventory of player   # in 'on inventory click': the menu title
location at (1, 64, 2) in world player's world
```

## Loops

```
loop all offline players:
loop all entities in player's world:
loop all zombies in world "nether":
loop-block / loop-item / loop-location / loop-world / loop-offlineplayer
loop-player        # also works in "loop all players within 25 meters of x"
```

- `loop all offline players` goes through the players the server told Carve
  about: everyone who joined since the scripts loaded, plus anyone online. The
  plugin API cannot list players who never joined since then. Each loop value
  has a name and a UUID.
- Offline players you type as an argument get the offline-mode UUID of their
  name when Carve does not know them, so keys like `{x::%uuid of arg-1%}` stay
  the same between runs (but are not the same as the account's online UUID).

## Menus and items

```
set slot 11 of {_gui} to redstone dust named "&cRedstone" with lore "a", "b"
set slots 0, 1 and 2 of {_gui} to gray stained glass pane named " "
set slot (integers between 0 and 8) of {_gui} to black stained glass pane
set slot {_i} of {_gui} to loop-value with lore "&7Click"
set {_i} to diamond sword of sharpness 3 and unbreaking named "x"
set {_i} to gold block of mending
set {_i} to grass block named "x" with all flags hidden
```

- `of <enchantment> [level]` adds enchantments (level 1 if left out).
- `with all flags hidden` hides the enchantment, attribute and similar tooltip
  lines.
- Item names that old scripts use: `dirt block`, `endstone`, `wheat seed`,
  `bottle o' enchanting`, `redstone dust`, `gold <tool>`, `daylight sensor`,
  `player's head`, `grass`, `bed`.

```
delete player's held item
set 1st line of (player's held item)'s lore to "x"
set line 2 of player's tool's lore to "y"
remove all armor stand from the player
```

## World and time

```
remove 4 seconds from "world"'s time
add 100 to time of world "world"
draw 100 cloud at location 0.75 meter above player with offset vector(0.5, 1, 0.5) with extra 0
teleport player at {spawn}
```

## Events

```
on damage of horse or donkey or skeleton horse:
on rightclick on chest or barrel or a bed:
on walking on stone:                  # same as 'on step on'
on walking on (red terracotta, lime terracotta or blue terracotta):
on food bar change:                   # same as 'on food level change'
on tab complete of "/cmd":            # read, but never runs (a warning)
```

## Events without a player

```
on explode:
    world is "world"                  # event-world
on redstone change:
    if event-block's world is "world":
on spawn of a skeleton:
    kill the entity's vehicle         # the entity: the spawned mob
    kill the entity
on rightclick on a creature:          # any entity; or name kinds: a zombie or a villager
    make player ride creature         # creature: the clicked entity
```

- `event-world` is the place of the event when the host knows it (`on spawn`,
  `on redstone change`), else the player's world, else the main world. Pumpkin
  does not say the world for `on explode` and `on form`, so there it is the
  main world.
- `the entity` (also `creature`) is the real entity in `on spawn` and in a
  click on an entity. `event-entity` stays the entity's type as text.
- `on rightclick on <entity>` listens to a new `entity click` event. It
  fires on the first (`interact`) packet of a right click.
- `the entity's vehicle` is `vehicle of the entity`.
- `set 1st line of the item's lore to ...` in `on drop` is read, but does
  nothing (a warning says so): the drop event gives no item to change.

## Not built

- `set data value of ...` (1.8 only; modern Skript has no data values).
- `on tab complete of "/cmd"` does nothing: Pumpkin plugins cannot change
  command completions.
- `item is frosted ice` in `on form`: the form event only gives the position,
  not the new block.
- `format slot`, `create a gui`, YAML and other addon syntax.
