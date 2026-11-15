# Expressions: not, limits, items, last-event facts, light

Item, damage and last-event expressions. Each one works anywhere an expression works.

## Logic and numbers

```
set {_a} to not true            # false
maximum double value            # 1.7976931348623157e308
maximum float value             # 3.4028234663852886e38
```

`not` on an unset value gives `true`. `not` on anything that is not a boolean gives `<none>`.

## Items

```
item amount of 5 diamonds                 # 5
key of diamond sword                      # minecraft:diamond_sword
namespaced key of stone                   # same thing
max durability of diamond pickaxe         # 1561 (<none> for items that don't wear out)
max item use time of bow                  # a timespan (3600 seconds)
cooldown of ender pearl for player        # timespan left, 0 seconds when none
set cooldown of ender pearl for player to 5 seconds
reset cooldown of ender pearl for player
```

`item cooldown of ...` and `item cooldown` in the effects also work. The durability and use time
numbers come from a table built into Carve, so unusual modded items give `<none>`.

## Text and players

```
last color of "&aHello &cWorld"          # red
last colors of "&#ff8800hi"               # #ff8800
language of player                        # en_us   (also: locale of player)
max freeze time of player                 # 7 seconds
```

`last color of` gives a color name for the 16 classic colors and `#rrggbb` for any other color.
It gives `<none>` when the text has no color.

## What happened last

Carve starts remembering these when the plugin loads. A restart forgets them.

```
last damage of player             # number: how much the last hit took
last damage cause of player       # text like fall, player_attack, arrow
last attacker of player           # the player who hit, <none> for mobs and the world
last death location of player     # a location
resource pack response of player  # text such as successfully_loaded, declined
last resource pack response       # same, for the event player
```

## Join message

In `on join`:

```
on join:
    send join message                       # the message Pumpkin was going to show
    set the join message to "&e%player% is here!"
```

Colors work. Outside `on join`, `join message` is `<none>`.

## World lookups

```
location at (10, 64, -3) in world "world"       # builds a location
location at (1, 2, 3)                           # no world
light level at {_loc}                           # the larger of block and sky light
block light level at {_loc}
sky light level at {_loc}
highest block at {_loc}                         # a location (also: lowest solid block at ...)
```

`highest block at` and `lowest block at` look down (or up) the column of the location. Air, liquids
and plants count as not solid. Lowest starts at y -64, so it finds bedrock in the overworld.

## Not supported on Pumpkin

Loot tables, leash holder, love time, interaction entities, minecart speed, the plugin list, server
icon and login history. The API has no way to read them. See `docs/skript-parity.md`.
