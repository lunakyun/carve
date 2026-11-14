# Known limits

Carve is young. These are the things that work differently from what you might
expect, or don't exist yet. Plans for some of them are in the
[roadmap](../README.md#roadmap).

## Commands

- **New commands need a restart.** Pumpkin's plugin API registers commands
  only while the plugin loads, and it can't unregister them. A reload updates
  the code of existing script commands. A new command needs a restart, and a
  removed command answers "no longer in any script" until the next restart.
  If you change an existing command's arguments, restart as well.
- **Permission nodes are registered only at load.** A node you add with a
  reload isn't registered until the next restart.
- **Unregistered nodes are denied, even for ops.** Nodes without a namespace
  are put in Carve's namespace (`heal` becomes `carve:heal`). Pumpkin denies
  any node that was never registered, unless it is given to the player
  directly. A node from another plugin (`other:x`) works only if that plugin
  registered it.

## Events

- **Events:** join, quit, chat, break, place, death, damage, respawn, clicks,
  `on load` and `every`. Inventory, GUI, sneaking and other events are not
  there yet.
- **Damage:**
  - `on damage` is for players only.
  - `attacker` is a player or `<none>`; mobs show as `<none>`.
  - The damage can't be changed, only cancelled.
  - For hits from an entity, Carve runs the trigger from Pumpkin's
    "damage by entity" event. A hit of an entity type (arrow, mob attack, ...)
    that has no attacking entity therefore doesn't run `on damage`.
- **Death:**
  - `on death` has no attacker (Pumpkin doesn't say who did it).
  - `on death` has no death message.
  - Cancelling a death leaves the player at 0 health.
- A block break without a player has no `player`.
- If the server calls back into Carve while a script is running (a nested
  event), that event is skipped with a warning. Console commands from scripts
  run after the script finishes, which avoids this in the common case.

## Language

- **Missing values:** no dates or time spans (`now`, `time since`), no
  vectors, no entities other than players.
- **Items:** no item names, lore or enchantments. `give` uses Pumpkin's
  `/give` command, so `give ... named "..."` doesn't exist.
- **Inventory:** `player has` counts the 36 main slots only.
- **Lists:** lists can't be passed to or returned from functions. There is
  no `set {list::*} to a, b and c` yet; add the items one by one.
- **Effects not there yet:** sounds, game modes, blocks in the world, and
  `set damage`.
- **Functions:** a function used as a value can't `wait`. Functions can call
  each other 64 levels deep.
- **Commands:** no `cooldown:` entry.

## Variables

- Names are lowercased.
- A player in a variable name is written as the player's name, not their UUID.
  A player who changes their name starts over.
- If `variables.json` is broken, Carve moves it to `variables.json.broken` and
  starts with no variables.

## Waits and the console

- When a command run by the console reaches a `wait`, messages sent to the
  console after the wait go to the server log.
- After a run's first `wait`, `cancel event` does nothing and logs a warning,
  because the event already happened.
- Each run slice (the part between two waits) can run at most 100,000
  instructions. A run that hits the limit stops with an error.
