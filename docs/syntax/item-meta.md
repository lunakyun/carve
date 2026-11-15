# Item meta

An item can carry a name, lore and enchantments. This page shows how to write them and how to read them back.

## Making an item with meta

Add the parts after the item. They can come in any order.

```
give player diamond sword named "&bBlade"
give player diamond sword with lore "first line" and "second line"
give player diamond sword enchanted with sharpness 5
give 2 diamond swords named "&6Twin" with lore "a pair" enchanted with sharpness 3 and unbreaking 3 to player
```

| Part | Example |
| --- | --- |
| Name | `named "&bBlade"` (also `with name "&bBlade"`) |
| Lore | `with lore "a"` or `with lore "a" and "b"` (one line per text) |
| Enchantments | `enchanted with sharpness 5`, `enchanted with sharpness 5 and unbreaking 3` |

Names and lore take the same color codes as `send` (see [colors](colors.md)).

An enchantment is its vanilla name, then a level. The level can be a number or a Roman numeral (`sharpness v`). With no level, the level is 1. Names with several words work: `sweeping edge 3`, `luck of the sea 2`, `fire aspect`. Skript style names also work: `curse of binding`, `curse of vanishing`, `sweeping`.

The item is a normal value. Keep it in a variable and give it later:

```
set {_blade} to diamond sword named "&bBlade" with lore "sharp"
give player {_blade}
```

Variables saved to `variables.json` keep the name, lore and enchantments.

## Reading item meta

| Syntax | Result |
| --- | --- |
| `name of player's tool` | The name, or `<none>` when it has none. `player's tool's name` also works. |
| `lore of player's tool` | The lore lines, as a list. |
| `enchantments of player's tool` | A list like `sharpness 5`, `unbreaking 3`. |
| `item type of x` | The item type alone, amount 1, no meta. |
| `item amount of x` or `amount of x` | The amount. |

Lore and enchantments are lists, so you can loop over them or store them:

```
on join:
    loop lore of player's tool:
        send "lore: %loop-value%"
    set {_e::*} to enchantments of player's tool
```

Names and lore read from a held item come back as plain text. Colors are not kept.

## Changing the held item

```
set name of player's tool to "&6Excalibur"
reset name of player's tool
set lore of player's tool to "&7Legendary" and "&7Sharp"
add "&8Line 3" to lore of player's tool
clear lore of player's tool
enchant player's tool with sharpness 5 and unbreaking 3
disenchant player's tool
```

- These change the real item in the player's main hand. `player's held item`
  and `tool of player` work too.
- An empty hand does nothing.
- `enchant` keeps the other enchantments and replaces the level of one that is
  already there. An enchantment without a level gets level 1.
- `disenchant` removes every enchantment.
- `set lore of ... to {_lines::*}` takes a list.

## Conditions

```
if player's tool is enchanted:
if player's tool is enchanted with sharpness:
if player's tool is enchanted with sharpness 5:
if player's tool is not enchanted with fire aspect:
```

- With no level, any level matches.
- With a level, that level or higher matches.

## Comparing items

`is` compares the item type only. The amount, name, lore and enchantments are ignored.

```
if player's tool is diamond sword:
    send "any diamond sword, named or not"
```

The same goes for `player has diamond sword named "x"`: it counts diamond swords and ignores the name.

## How give works

`give` runs the vanilla `give` command with item components, as the console. Pumpkin reads names and lore as plain strings, so Carve turns color codes into `§` codes.

## Limits

- Only the main-hand item can be edited in place. Other slots: build the item
  with `named` / `with lore` and `set slot`.
- Other item parts (flags, custom model data, tooltips, attributes) are not supported.
