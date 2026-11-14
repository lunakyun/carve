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

- There are five events: join, quit, chat, break and place.
- A block break without a player has no `player`.
- If the server calls back into Carve while a script is running (a nested
  event), that event is skipped with a warning. Console commands from scripts
  run after the script finishes, which avoids this in the common case.

## Language

- No loops, functions, options, or `and` / `or` in conditions.
- No item, location, world or entity types. `teleport` uses the player's
  current world.
- No way to loop over a list. `{list::*}` only works with `delete`.
- Text can only be compared as a whole. There is no "contains" or "starts
  with".
- No boolean values in scripts. They can only come from `variables.json`.

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
