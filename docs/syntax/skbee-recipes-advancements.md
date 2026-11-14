# Recipes, advancements and statistics

SkBee-style syntax, built in. See [skbee.md](../skbee.md) for what works on
Pumpkin and what does not.

## Advancements

```
grant advancement "story/mine_stone" to player
award advancement "minecraft:story/root" to player
revoke advancement "story/mine_stone" from player
grant criterion "mined_stone" of advancement "story/mine_stone" to player
revoke criterion "mined_stone" of advancement "story/mine_stone" from player
```

An id with no namespace gets `minecraft:`. Offline players are skipped.

Condition:

```
if player has advancement "story/root":
if player doesn't have advancement "story/root":
if player has not completed advancement "story/root":
```

Event:

```
on advancement done:
    send "You earned %advancement id%" to player
```

`advancement id` is the full id, like `minecraft:story/root`. You can cancel
the event. `on advancement` and `on advancement complete` mean the same.

## Recipes

The id comes first and must be plain text. With no namespace it becomes
`carve:<id>`. Items are normal item words (`diamond sword`, `2 torches`) or
text (`"minecraft:diamond"`, `"#minecraft:planks"` for a tag).

Shaped: 1 to 3 rows, all the same length, 1 to 3 characters wide. A space is
an empty slot. Every other character needs a `where` entry.

```
on load:
    register shaped recipe "super_sword" for diamond sword with shape " d ", " d ", " s " where d is diamond block and s is stick
```

Shapeless: 1 to 9 ingredients. A number repeats an item.

```
register shapeless recipe "torches" for 4 torches using coal and stick
register shapeless recipe "mix" for stone using 3 dirt, gravel
```

Cooking: `furnace` (or `smelting`), `blasting`, `smoking`, `campfire`.

```
register furnace recipe "fast_iron" for iron ingot using raw iron with xp 0.7 cooking time 5 seconds
register campfire recipe "steak" for cooked beef using beef cooking time 300 ticks
```

Defaults: no experience, 200 ticks for furnace and campfire, 100 ticks for
blasting and smoking. Cooking time is in `ticks` or `seconds`.

Smithing recipes are a load error, because Pumpkin cannot register them.

### Recipe book

```
discover recipe "crafting_table" for player
unlock recipe "my:super_sword" to player
undiscover recipe "my:super_sword" for player
lock recipe "stick" from player
```

Event:

```
on recipe discover:
    send "New recipe: %recipe id%" to player
```

## Statistics

Names are `category:name`. The category is one of `mined`, `crafted`, `used`,
`broken`, `picked_up`, `dropped`, `killed`, `killed_by` or `custom`. With no
category the name is a `custom` statistic. Spaces and dashes become `_`.

```
set statistic "custom:jump" of player to 5
add 3 to statistic "jump" of player
remove 1 from statistic "custom:jump" of player
set {_n} to statistic "jump" of player
send "Jumps: %{_n}%" to player
```

The read gives `<none>` when the player is offline or the server cannot read
that statistic. Values never go below 0.

On Pumpkin only `custom` statistics work, such as `jump`, `play_time`,
`walk_one_cm`, `deaths`, `mob_kills` and `damage_dealt`. The other categories
log a warning and do nothing.
