# Known limits

These are the places where Carve works differently from Skript, or can't do
something yet. Most of them come from Pumpkin's plugin API (v0.1), not from
Carve itself. For the full feature-by-feature status, see the
[Skript parity checklist](./skript-parity.md).

## Commands and permissions

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
- **Tab completion** works for `/carve` subcommands only. Pumpkin 0.2.2 crashes
  when it asks a plugin for completions, so Carve offers none for script
  names or script command arguments.

## Reloading

- A single-file reload (`/carve reload <script>`) doesn't run `on load`.
  A full reload does.

## Events

- **Late damage:** a few hits of an attacking type come without the attacking
  entity (`/damage`, the ender dragon's wing attack, snowballs). Carve sees
  those only after Pumpkin has applied them, so `on damage` still runs, with
  `attacker` `<none>`, but cancelling or changing the damage has no effect.
- **Death:**
  - Pumpkin's death event doesn't say who did it, so `attacker` in `on death`
    is the last entity that hit the victim in the 5 seconds before.
  - A mob's death can't be cancelled and has no `death message` (Pumpkin's mob
    death event has neither).
  - Cancelling a death leaves the player at 0 health. Pumpkin marks the player
    as dead before the event runs.
- **Move:** `on move` runs when the player moves to another block, not on
  every small move or turn of the head.
- **Consume:** `on consume` runs when the player starts eating or drinking,
  not when the item is used up. Pumpkin fires its event on the right click.
- **Pick up:** `on pick up` has no `event-item` (Pumpkin 0.2.2 sends the
  entity type id instead of the item), and `cancel event` doesn't stop the
  pickup, because Pumpkin fires the event after the item is in the inventory.
- **World names:** Pumpkin 0.2.2 gives every dimension the level folder's
  name, so Carve names the nether and the end by their dimension id
  (`minecraft:the_nether`, `minecraft:the_end`), not `world_nether` and
  `world_the_end` like Skript.
- **Nested events:** if the server calls back into Carve while a script is
  running, that event is skipped with a warning. Console commands from scripts
  run after the script finishes, which avoids this in the common case.
  `set weather`, `strike lightning` and `create explosion` fire their
  `on weather change`, `on lightning strike` and `on explode` events after the
  script instead, so cancelling those events there has no effect. Other events
  a script's own effect would start (a block placed by `set block`, for
  example) are skipped. A `grant advancement` from a script is one such case:
  its `on advancement done` does not run.
- **Recipe discover:** Pumpkin fires `on recipe discover` when the client marks
  a recipe as seen in its recipe book, not when the recipe is unlocked, and
  `recipe id` is `display_<n>`.
- **Weather and lightning events** only come from plugins in Pumpkin 0.2.2:
  the `/weather` command, the natural weather cycle and `/summon
  lightning_bolt` don't fire them.
- **`kill` on a mob** is not a console command, so the mob dies inside the
  script and `on damage` doesn't run for that hit.

## Entities

- **Mobs keep moving while a script runs.** Pumpkin ticks entities on its own
  thread, so `y velocity of {_e}` read right after `set y velocity of {_e}` can
  already show one tick of gravity and drag.

## Inventories

- **No closing windows.** `close player's inventory` compiles, but Pumpkin's
  plugin API v0.1 has no call to close a window, so it only logs a warning.
- **Chest menus only.** Inventory values (`chest inventory with 3 rows`) are
  chests. Hoppers, furnaces and other window types can't be built.
- **`event-inventory` only knows Carve's menus.** For a chest block or any
  other window Carve did not open, `event-inventory` is `<none>`.

## Variables

- A player in a variable name is written as the player's **name**, not their
  UUID (Skript uses the UUID by default). A player who changes their name
  starts over. Use `%uuid of player%` in the name if you want Skript's
  behavior.
- If `variables.json` is broken, Carve moves it to `variables.json.broken` and
  starts with no variables.

## Running scripts

- Each run slice (the part between two `wait`s) can run at most 100,000
  instructions. A run that hits the limit stops with an error, so a runaway
  loop can't freeze the server.
- A function used as a value (`set {_x} to f()`) can't `wait`. Call it on its
  own line instead. Functions can call each other 64 levels deep.
- When a command run from the console reaches a `wait`, messages sent to the
  console after the wait go to the server log.

## Plugin API v0.2

Pumpkin is working on a plugin API v0.2. It is switched off in normal Pumpkin
builds until it's finished, so Carve uses v0.1. v0.2 adds calls that would
lift several limits above (closing inventories, real entity spawning, tags and
scores). Carve's engine doesn't depend on the API version, so a v0.2 build
can be added once Pumpkin turns it on.
