# SkBee parity

SkBee is the most popular Skript addon. This page lists which SkBee features
Carve has built in. Carve's syntax is close to SkBee's, so scripts port
easily. It is not copied word for word.

✅ done, 🟡 partial, ⛔ blocked (the reason is given).

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
