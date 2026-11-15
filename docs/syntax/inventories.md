# Inventories

Chest menus, slots, clearing, adding and removing items, and the
`on inventory click` event.

Effects and conditions on a player skip silently when the player is offline.
Item names work like everywhere else in Carve: `diamond`, `2 diamonds`,
`apple`.

## Open a chest menu

```
open chest with 3 rows named "Shop" to player
open a chest with 1 row to player
open chest with {_rows} rows titled "&aMenu" for player
```

- Rows are 1 to 6. A wrong number written in the script is a load error. A
  wrong number from a variable is a runtime error, and nothing opens.
- The title may use color codes. With no title it is `Chest`.
- Words that mean the same: `named`, `titled`, `called`, `with title`.
- The menu starts empty. Fill it with `set slot`.

## Inventory values

Build a menu in a variable first, fill it, and open it later. This is how
Skript shop scripts work.

```
set {_gui} to chest inventory with 3 rows named "Shop"
set slot 4 of {_gui} to diamond named "&bBuy"
set slot 5 of {_gui} to 2 emeralds
open {_gui} to player

send "%name of {_gui}%" to player
send "%slot 4 of {_gui}%" to player
loop items in {_gui}:
    send "%loop-value%" to player
if {_gui} contains diamond:
    send "has a diamond" to player
clear {_gui}
```

- `[a] [new] chest inventory [with N rows] [named "x"]`. `gui` or `menu`
  work in place of `inventory`, and `titled`, `called`, `with name` in place
  of `named`. Rows are 1 to 6 (3 when not given). A wrong number is a runtime
  error. The default title is `Chest`.
- Slots count from 0 (top left) to `rows * 9 - 1`. A slot outside the
  inventory is a runtime error.
- Items keep their names, lore and enchantments in the menu.
- A copy of the value is the same inventory: `set {_b} to {_gui}` and then
  `{_b} is {_gui}` is true. Changing a slot of a variable after `open` also
  changes the menus that are open from it.
- Inventory values can be kept in global variables. They are saved to
  `variables.json` with their items.
- `clear {_gui}` empties it. On a variable that holds something else, `clear`
  deletes the variable (like Skript).
- `items in player's inventory` (or `items in inventory of player`) lists the
  player's items, armor and off hand included. `items in player's current
  inventory` lists the open menu.

### Telling menus apart in clicks

```
on inventory click:
    if name of event-inventory is "Shop":
        cancel event
        if clicked slot is 4:
            send "You bought a diamond." to player
    if event-inventory is {shop}:
        send "This is the stored shop." to player
```

- `event-inventory` is the menu Carve opened, with its name and its live
  contents (`slot 4 of event-inventory`, `items in event-inventory`).
  `set slot 4 of event-inventory to stone` changes the open menu.
- `clicked inventory` is the same menu, but `<none>` when the player clicked
  their own inventory below the menu.
- Both are `<none>` for windows Carve did not open (a chest block, say).

## Slots

```
set slot 4 of player's current inventory to diamond
set slot 0 of player's inventory to 5 apples
set slot 2 of current inventory of player to air
set {_item} to slot 3 of player's current inventory
```

- Slots count from 0.
- `player's inventory` is the player's own inventory. Slots 0 to 8 are the
  hotbar, 9 to 35 the rest, 36 to 39 the boots, leggings, chestplate and
  helmet, and 40 the off hand.
- `player's current inventory` is the menu the player has open, if Carve
  opened it. If there is none, it is the player's own inventory.
- `air` empties the slot.
- A slot that is empty reads as unset: `if slot 3 of player's inventory is not set:`.
- A slot below 0 is a runtime error.

## Items in the inventory

```
clear player's inventory
clear player's current inventory
remove 2 diamonds from player
remove 3 stone from player's inventory
add 4 emeralds to player's inventory

if player's inventory contains 2 diamonds:
if player's inventory does not contain stone:
```

- `clear player's inventory` empties the player's own inventory, armor and
  off hand included.
- `remove` takes up to that many. If the player has fewer, it takes what they
  have. It only looks at the 36 main slots.
- `remove ... from {list::*}` and `add ... to {list::*}` still work on lists.
  The inventory forms are only used when the target is a player or says
  `inventory`.

## Close

```
close player's inventory
```

This line loads, but it does nothing yet. Pumpkin's plugin API v0.1 has no
call to close a window, so Carve writes a warning to the log. The newer v0.2
API has `close-inventory`. When Carve moves to v0.2 this starts working.

## Events

```
on inventory click:
    if clicked slot is 4:
        cancel event
        send "You clicked the diamond." to player
    send "%click type%" to player

on inventory close:
    send "Bye." to player
```

In `on inventory click`:

| Value | What it is |
|---|---|
| `clicked slot` | The slot number that was clicked. |
| `clicked item` | The item in that slot, or unset if it was empty. |
| `click type` | Text like `left`, `right`, `shift left`, `middle`, `drop`. |

- `cancel event` stops the click. Do this in menus so players cannot take the
  items out.
- The event fires for every window, not only menus made by Carve. It also fires
  for clicks in the player's own inventory.
- `on inventory close` cannot be cancelled.

## Not done

- Closing a window (see above).
- `inventory type`, `inventory holder`, `viewers of`, and slot ranges
  (`slots 0 to 8 of {_gui}`).
- Rows other than a chest (hopper, furnace, anvil and so on).
