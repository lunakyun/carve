# Lines from real Skript servers

These forms come from scripts that servers already run. They work like they do
in Skript. The ones at the end need another plugin or something Pumpkin can't
do: they still load, with a warning at load time (see
[errors](../errors.md)), so the rest of the script keeps working.

## Sounds

```
play sound "ENTITY_TNT_PRIMED" with volume 0.7 and pitch 1 at {_loc} to all players
play sound "BLOCK_NOTE_BLOCK_PLING" with volume 1 and pitch 2 to all players
play sound "entity.creeper.primed" at player's location to {_players::*}
```

`to` takes one player, a list of players or `all players`. Every one of them
gets the sound packet, played at the `at` place (or where they stand). Bukkit
names like `ENTITY_TNT_PRIMED` are the vanilla ids with `.` written as `_`.

## Blocks and events

```
on place of tnt:
    set {_loc} to event-block's location
    add 1 to y-coordinate of {_loc}
    spawn primed tnt at {_loc}
    set {_tnt} to last spawned tnt
```

- `on place of chest or barrel:` works like `on break of <block>`.
- `event-block` is the placed or broken block, `event-block's location` its
  position. `event-block is emerald` compares the item `emerald` with the block,
  like Skript does: it is false for an `emerald block`.
- `damage cause is entity explosion or block explosion or primed tnt` takes a
  list of causes.
- `item is sword or armor or zombie skull` in `on drop`: `sword`, `axe`,
  `pickaxe`, `shovel`, `hoe`, `helmet`, `chestplate`, `leggings`, `boots` and
  `armor` match every item of that kind. `zombie skull` is `zombie head`.
  `item` is the dropped item in `on drop`, `on pick up` and `on consume`.
- A name with a color is an item, not a color: `light blue wool`,
  `light gray stained glass`.
- `a` and `an` can stand before an item: `player has an egg`.
- `on <event> with priority highest:` (see [triggers](triggers.md)).

## Shooting and pushing

```
shoot primed tnt from player at speed 2.4
set {_tnt} to last shot primed tnt
make {_bot} shoot an egg at speed 1
push {_tnt} in direction of player with force 0.2
```

The new entity starts at the shooter's eyes and flies the way they look (players
only). `primed tnt` is the `tnt` entity.

## Players

```
force teleport player to {spawn}
set player's hunger meter to 18
remove levitation from the player
remove jump boost from {_p}
apply glowing to {_p} for 30 seconds
set size of world border of player to 140
set bossbar progress of {bar} to 50
set bossbar visibility of {bar} to false
if idle time of player > 1 minute:
if player's offhand tool is tnt:
set {_at} to location 3 meters in front of player
```

- `remove <effect> from <player>` needs no `potion effect` words. `swiftness`
  is `speed`.
- `size of world border of <player>` is the border of that player's world.
- `set bossbar progress|title|color|style|visibility of <id> to ...` works
  with any text or variable that holds the bossbar id. Progress is 0 to 100.
- `idle time of <player>` counts from the last move, click, place, break or
  chat the plugin saw. It needs the plugin to listen to movement, which it does
  only when a script uses `idle time`.
- `3 meters in front of` follows where a player looks. For other places, "in
  front" is south. `behind`, `to the left of` and `to the right of` work too.
- `location(x, y, z, world, yaw, pitch)` is accepted. Locations here have no
  rotation, so yaw and pitch are read and dropped. `world("name")` and
  `world({_w})` give a world.

## Numbers, text and lists

```
if player count is greater than 3:
if ({stock} ? 0) <= 0:
subtract 1 from {_x}
subtract 1 minute from {_when}
set {_ml::*} to myFunctionThatReturnsAList()
set {_w::*} to {_input} split at " "
set {_who} to a random element out of "finn" and "soviet"
```

- `is greater than`, `is less than`, `is at least`, `is at most` and their
  `is not` forms are `>`, `<`, `>=` and `<=`.
- `(x ? 0)` in brackets can stand on either side of a comparison.
- `a function that doesn't exist` is reported as `unknown function 'name'`.
- `make console execute "cmd"` is `execute console command`.

## Chat

```
on chat:
    replace all "ez" in message with "gg"
    set message to formatted message
    set chat format to "&7%player%&8: &f%message%"
```

`set message to` changes what is sent. Pumpkin's chat event has no format
field, so `set chat format to` cancels the chat and sends your formatted line to
the recipients itself. Other plugins that change chat don't see that message.

## Loads with a warning

| Line | What happens |
|---|---|
| `"%region at player%"` | Needs WorldGuard. Always `<none>`, so `contains "none"` is true everywhere. |
| `player's prefix`, `prefix of player` | Needs a chat plugin. Always empty text. |
| `set explosive yield of <entity> to 15` | Does nothing: the plugin API has no explosion power on entities. |
| `on tab complete:` | Never runs: no such event. `tab arg-1` still parses. |
| `suggestions:` in a command | Checked, never run: Pumpkin 0.2.2 crashes on plugin tab completion. `current input` reads as empty text. |

An `if` or `loop` with only a comment under it is a warning too: it is left
out and the rest of the script loads. (An empty trigger is still an error.)
