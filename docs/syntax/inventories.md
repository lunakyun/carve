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
- A menu you build first and open later, `inventory type`, and drag, open,
  item move and pickup events.
- Rows other than a chest (hopper, furnace, anvil and so on).
- `first empty slot`, `size of inventory`, `inventory holder`.
