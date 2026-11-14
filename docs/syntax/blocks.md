# Blocks

Read, place and break blocks in the world.

A block value is a location with whole numbers. `block at location(1.7, 64.2, 3.9)`
is the location `1, 64, 3`. You can keep a block in a variable and use it later.

## Getting a block

| Syntax | What it gives |
|---|---|
| `block at <location or player>` | The block that holds that point. |
| `block below <place>` / `block above <place>` | The block under or over it. |
| `block north of <place>` | The next block. `north` is -z, `south` +z, `east` +x, `west` -x. |
| `block {_loc}` | Same as `block at {_loc}`. |
| `targeted block of <player>` | The block the player looks at (up to 100 blocks). `<none>` when there is none. |
| `<player>'s targeted block` | Same. |
| `targeted block` | The targeted block of the event player. |
| `location at x, y, z [in world "w"]` | A location. Without a world, the default world is used. |

```
set {_b} to block below player
set {_t} to targeted block of player
set {_spot} to location at 10, 64, -5 in world "world"
```

## Type of a block

`type of block at <place>` gives the block type as an item, like `stone`.
`type of {_b}` works for a block you stored.

```
send "You stand on %type of block below player%."
if type of block at {_spot} is stone:
    send "stone here"
```

## Set and break

| Syntax | What it does |
|---|---|
| `set <block> to <type>` | Place a block. `<block>` is any form above. |
| `break <block>` | Break the block. Also `destroy <block>`. |

```
set block at location at 0, 64, 0 to oak planks
set block below player to diamond block
break targeted block of player
```

The type must be a plain block name. Text with odd characters (like `;`) is
refused with an error. A place outside the world limits is skipped with a warning.

## Condition

`<block> is [not] <type> [or <type>]`

```
if block below player is grass block or dirt:
    send "soft ground"
if targeted block of player is not air:
    send "you are looking at something"
```

The condition is false when the block cannot be read.

## How it works on Pumpkin

- Reading, setting and targeting use the plugin API directly (`get-block`,
  `set-block-by-name`, `ray-trace-block`).
- `break` removes the block and updates neighbours. It does not drop items.
- On a host without those calls, `set` and `break` fall back to the
  `/setblock` command (in the current world only). Reading blocks and
  `targeted block` then give nothing.

## Not supported yet

- Block data (`facing`, `waterlogged`), block hardness, drops of a block.
- `break event-block`: `clicked block` has no position yet.
- Entity types in `type of`.
