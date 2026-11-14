# SkBee in Carve

Carve builds the most useful parts of the SkBee addon in. The syntax is close
to SkBee, so scripts are easy to port. Each area has its own section.

Legend: ✅ done, 🟡 partial, ⛔ blocked.

## Bounds

Docs: [syntax/skbee-bound.md](syntax/skbee-bound.md). In SkBee, a bound is a
region with an id. In Carve it is held as its id text.

| SkBee feature | Status | Note |
|---|---|---|
| Create bound with id within two locations | ✅ | `create bound with id "x" within A and B` |
| Temporary bound | ✅ | not saved |
| Delete bound | ✅ | |
| Bound with id | ✅ | `bound "x"` or `bound with id "x"` |
| Bound at location | ✅ | |
| Bounds at location (all) | ✅ | list |
| All bounds | ✅ | list |
| Location / player is within bound | ✅ | |
| Bound exists | ✅ | |
| On bound enter / exit | ✅ | polled every 2 ticks, grid index, per-player block cache |
| Event-bound | ✅ | |
| Bound owners and members | ✅ | add, remove, set, list, role conditions |
| Bound flags | 🟡 | yes/no flags only. SkBee also has typed bound values |
| Bound lesser / greater corner | ✅ | readable and settable |
| Bound world | ✅ | read only |
| Bound volume / size | 🟡 | `volume of bound` only |
| Persistence in the data folder | ✅ | `bounds.json`, saved 5 seconds after a change and on unload |
| Bound bounding box | 🟡 | corners only, no box type |
| Blocks in bound | ⛔ | needs a block iteration API that the Pumpkin plugin API does not have yet |
| Entities in bound | ⛔ | no entity list API in the plugin API |
| Bound enter/exit for non-players | ⛔ | no entity move events in the plugin API |
| Bound values (custom data) | ⛔ | no typed value store yet; Carve variables can hold this |

# SkBee parity

Carve adds the most useful parts of the SkBee addon by default, with syntax close to SkBee so scripts port easily. Legend: ✅ done, 🟡 partial, ⛔ blocked.

## Text components

Syntax details: [syntax/skbee-text.md](syntax/skbee-text.md). Carve components are MiniMessage markup text, not a separate type. That is the main difference from SkBee.

- ✅ Text component from text (`text component from "x"`)
- ✅ Hover event showing text (`hover event showing "x"`)
- ✅ Hover event showing an item (`hover event showing 2 diamonds`; id and amount only)
- 🟡 Hover event showing an entity (MiniMessage tag only, no expression)
- ✅ Click event: open url, run command, suggest command, copy to clipboard, change page
- ✅ Set hover event / click event of a component (single variable only)
- ✅ Insertion (`set insertion of {c} to "x"`, `<insertion:..>`)
- ✅ Translate component with values (`translate component "key" with "a" and "b"`)
- ✅ Keybind component (`keybind component "key.jump"`)
- ✅ Merge components (`merged components of a, b and c`, or of a list variable)
- ✅ Send component to a player, send action bar component, broadcast component
- ✅ MiniMessage: `mini message from "..."` and the tags `<hover>`, `<click>`, `<insertion>`, `<key>`, `<lang>`, `<newline>`
- ✅ MiniMessage gradient (`<gradient:red:blue>`, any number of colors, optional shift) and `<rainbow>`
- ✅ Plain text of a component (`plain text of x`)
- 🟡 Component styles as expressions (use tags like `<bold>` or `&l` instead)
- 🟡 Events on a list of components (set events on one variable at a time)
- ⛔ Fonts and shadow color on a component: the plugin API supports them, but Carve has no syntax yet
- ⛔ Component object tags (`<sprite>`, `<head>`) and `<selector>`/`<nbt>` tags: not planned


## Scoreboards and teams

Syntax: [syntax/skbee-scoreboard-teams.md](syntax/skbee-scoreboard-teams.md).

Pumpkin's plugin API only gives each player a private scoreboard. It has no
call to read or change the world scoreboard. Carve keeps its own copy and
mirrors changes with `/scoreboard` and `/team` commands. See the syntax page
for what that means.

### Objectives

- ✅ Register an objective, with criteria and display name.
- ✅ Unregister an objective.
- ✅ Objective exists (condition).
- ✅ Objective criteria and display name (expressions).
- ⛔ Set the display slot of the world scoreboard (sidebar, player list, below
  name). Pumpkin has no `/scoreboard objectives setdisplay`, and the plugin
  API can only set slots on a player's own scoreboard. Use a board (below)
  for a sidebar.
- ⛔ Change an objective's display name, render type or number format after
  it is registered. Pumpkin has no `/scoreboard objectives modify`.

### Scores

- ✅ Set, add, remove and reset the score of a player or a name.
- ✅ Reset every score of a holder.
- ✅ Read a score (`<none>` when unset).
- 🟡 Score holders are players and one word names. Entities by UUID or by
  selector (`@e`) are not supported.
- ⛔ A list of all scores in an objective. Not read back from the server,
  and Carve does not offer it from its copy yet.

### Teams

- ✅ Register and unregister a team.
- ✅ Add and remove members; clear a team; a player is in one team at a time.
- ✅ Team of a player, members of a team, team size, is in team, team exists.
- ✅ Prefix, suffix, display name, color.
- ✅ Friendly fire, see friendly invisibles.
- ✅ Nametag visibility, collision rule.
- 🟡 Death message visibility. Pumpkin has the command, Carve does not use it
  yet.
- ⛔ Teams and members made outside Carve are unknown to Carve.
- ⛔ Per-player scoreboard objects (`scoreboard of player`) as values.

### Boards (SkBee fastboard)

- ✅ Per-player sidebar with a title and up to 15 lines.
- ✅ Set one line, set all lines, delete a line, delete the board.
- ✅ Colors with `&` codes. Numbers on the right are hidden.
- 🟡 Java Edition only. Bedrock players get nothing.
- 🟡 Boards are not saved across restarts. Set them in `on join`.
- ⛔ Per-line placeholders that refresh by themselves (SkBee's variable
  binding). Set the line again, for example from an `every 1 second` trigger.
