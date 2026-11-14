# NBT and item components (SkBee style)

This page covers NBT compounds and a few item data components. The syntax
is close to [SkBee](https://github.com/ShaneBeee/SkBee/wiki), so most SkBee
scripts port with small changes.

## What items you can reach

Carve has no item objects that carry data. Item data is read and written on
the item in a player's main hand: `player's tool`, `tool of player` or
`held item of player`. If the player holds nothing, reads give `<none>` and
writes do nothing.

NBT stored in variables works for any value. Variables can keep NBT, also
across restarts. It is saved as SNBT text.

## NBT compounds

A compound is written as SNBT text. Numbers can have a type suffix:
`1b` byte, `2s` short, `3` int, `4L` long, `1.5f` float, `1.5d` double.
`true` and `false` are bytes (1 and 0). Arrays look like `[I;1,2]`,
`[B;1]`, `[L;5]`. Quotes inside a script string are doubled: `""`.

| Expression | What it gives |
|---|---|
| `nbt from "{a:1}"`, `nbt compound from "..."` | a compound parsed from text (a bad text is a script error) |
| `nbt of player's tool` | the item's custom data as a compound |
| `nbt of {_x}` | the compound (or SNBT text) in a variable |
| `snbt of <nbt>` | the compound as text |
| `tag names of <nbt>` | the keys, as a list |

A compound printed in a message shows its SNBT text.

## Reading tags

```
set {_n} to nbt from "{power:5,id:{name:""blade""}}"
send "%int tag ""power"" of {_n}%"
send "%string tag ""id;name"" of {_n}%"
send "%tag ""power"" of nbt of player's tool%"
```

Syntax: `[type] tag "path" of <nbt>`. The `<nbt>` can also be
`player's tool` directly.

- Types: `byte`, `short`, `int` (or `integer`), `long`, `float`, `double`,
  `string` (or `text`), `boolean`, `compound`, `list`, `byte array`,
  `int array`, `long array`.
- Without a type you get the tag as it is.
- If the tag is missing, or is not of the asked type, you get `<none>`.
- Numbers come back as numbers. Lists and arrays come back as lists.
- A path uses `;` between compounds: `"display;Name"`. A list item is
  `"items[0]"`.

## Changing tags

```
set int tag "power" of nbt of player's tool to 7
set string tag "id;name" of {_n} to "blade"
set tag "flag" of {_n} to true
delete tag "power" of nbt of player's tool
remove tag "id;name" from {_n}
set nbt of player's tool to nbt from "{a:1}"
add "{b:2}" to nbt of player's tool
merge nbt from "{c:3}" into {_n}
delete nbt of player's tool
```

- The target is a variable like `{_n}`, or the player's tool.
- Without a type, a whole number becomes an int, other numbers a double,
  text a string, `true` and `false` a byte, and NBT a compound.
- With a type the value is converted (`byte` is cut to -128..127).
- `list tag` and the array types take text like `"[1,2,3]"` or a list.
- Missing compounds along a path are created.
- `add <nbt> to nbt of ...` and `merge` copy keys in. Compounds merge deeply.
- On an item, an empty compound removes the custom data.

## Conditions

```
if {_n} has tag "power":
if player's tool has int tag "power":
if {_n} does not have string tag "id;name":
```

Also `doesn't have`. A typed check is false when the tag has another type.

## Item components

| Expression and effect | Value |
|---|---|
| `custom model data of player's tool` | number |
| `max stack size of player's tool` | number, 1 to 99 |
| `rarity of player's tool` | `common`, `uncommon`, `rare` or `epic` |
| `item model of player's tool` | text like `minecraft:stick` |
| `glint override of player's tool` | true or false |
| `unbreakable of player's tool` | true or false |

```
set custom model data of player's tool to 42
set rarity of player's tool to epic
set item model of player's tool to "my:model"
set glint override of player's tool to true
set unbreakable of player's tool to true
reset custom model data of player's tool
```

`reset`, `delete`, `clear` and `remove` take a component away.
`set unbreakable of ... to false` does the same. A missing component reads
as `<none>` (`false` for `unbreakable`).

## How it works on Pumpkin

The plugin API moves components as raw bytes. Carve builds and reads those
bytes itself. The main-hand item is read, changed and put back.

## Limits

- Only the main-hand item. Other slots, dropped items and item values in
  variables carry no data.
- Entities and blocks: the plugin API only offers namespaced keys with no
  list of keys, so Carve cannot give a whole compound.
- No `uuid` tag type, no tag types inside lists beyond what SNBT gives.
- Enchantments, lore, names, attributes and other components are not here.
