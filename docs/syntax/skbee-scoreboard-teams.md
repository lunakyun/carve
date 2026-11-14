# Scoreboards, teams and sidebar boards

This is Carve's version of the scoreboard and team parts of SkBee. The
syntax is close to SkBee's so scripts port easily.

## How it works

Pumpkin's plugin API gives each player a private scoreboard. It cannot read
the world scoreboard back. So Carve keeps its own copy of your objectives,
scores and teams in the global variables file. Every change is also sent to
the server with the normal `/scoreboard` and `/team` commands.

What this means for you:

- Reads (`score of`, `team of`, `members of team`) come from Carve's copy.
- The copy is saved with your variables, so it survives a restart.
- Things made outside Carve (by hand with `/team add`, or by another plugin)
  are not in the copy. Carve does not know about them. Scores you set on an
  objective Carve did not register still go to the server, with a warning.
- Names of objectives and teams use letters, numbers and `_ - . +`, up to
  64 characters. A name with a space is an error.
- A score holder is a player or a one word name. A player resolves to their
  name.

Sidebar boards are per player and are not saved. They are drawn by the host.
They work for Java players only.

## Objectives

```
register objective "kills"
register a new objective "kills" with criteria "playerKillCount" named "&cKills"
create new objective "pts" with display name "Points"
unregister objective "kills"
delete objective "kills"
```

The criteria default to `dummy`. `named` and `with display name` are the
same. The optional parts can come in any order. Registering a name twice is
an error. Unregistering also forgets every score in it.

```
objective "kills" exists
objective "kills" doesn't exist
objective criteria of "kills"
objective display name of "kills"
criteria of objective "kills"
```

Reading an objective that does not exist gives `<none>`.

## Scores

```
set score of player in objective "kills" to 5
set the score of "Bob" in "kills" to 5
add 3 to score of player in "kills"
remove 1 from the score of player in "kills"
reset score of player in "kills"
reset all scores of player
```

The word `objective` before the objective name is optional. A score is a whole
number from -2147483648 to 2147483647. Other numbers are rounded.

```
send "%score of player in ""kills""%"
set {_k} to score of player in "kills" ? 0
if score of player in "kills" > 10:
    send "nice"
```

A score that was never set is `<none>`, so `? 0` gives a default.

## Teams

```
register team "red"
unregister team "red"
clear team "red"                       # remove every member
add player to team "red"
remove player from team "red"
reset team of player                   # leave whatever team they are in
```

A player is in one team at a time. Adding them to another team moves them.
A name works too: `add "Alex" to team "red"`. Adding someone to a team that
was never registered is an error.

```
team "red" exists
team "red" does not exist
player is in team "red"
player is not in team "red"
team of player                          # the name, or <none>
members of team "red"                   # a list of names
team size of "red"                      # how many members
```

`members of team "red"` works in `loop` and in `set {x::*} to`.

### Team settings

```
set prefix of team "red" to "&c[R] "
set suffix of team "red" to " &7!"
set display name of team "red" to "Reds"
set color of team "red" to dark red
set friendly fire of team "red" to false
set see friendly invisibles of team "red" to true
set nametag visibility of team "red" to hide for other teams
set collision rule of team "red" to push own team
```

You can also write `set team prefix of "red" to ...`. Read them back with
`prefix of team "red"` or `team prefix of "red"`, and the same for `suffix`,
`color`, `display name`, `friendly fire`, `see friendly invisibles`,
`nametag visibility` and `collision rule` (all with `team ...`).

- Colors: the 16 Minecraft names (`dark red`, `light purple`, ...) and
  `reset`. `pink`, `purple` and `grey` also work. Write them as bare words or
  as text.
- Nametag visibility: `always`, `never`, `hide for other teams`,
  `hide for own team`.
- Collision rule: `always`, `never`, `push other teams`, `push own team`.
- Prefix, suffix and display name accept `&` color codes.
- A team that has never been changed reads back the defaults: empty prefix
  and suffix, color `reset`, friendly fire `true`, visibility `always`.

## Sidebar boards (fastboard style)

A board is a sidebar that belongs to one player. Different players can see
different boards. The board shows when you set a title or a line.

```
set title of board of player to "&6My Server"
set line 1 of board of player to "Kills: 3"
set line 3 of board of player to "&7play.example.com"
set lines of board of player to "Rank: Gold", "Coins: 40" and ""
delete line 2 of board of player
delete board of player
```

- `player's board` works in place of `board of player`.
- Line 1 is the top. Lines go from 1 to 15. A gap is a blank line.
- `set lines ...` replaces every line. Use commas or `and` between lines.
- A board with no title gets a blank one.
- Setting a board for a player who is offline does nothing.
- Texts accept `&` color codes. Two equal lines are kept apart for you.
- The numbers on the right are hidden.
- Boards are kept in memory. They are gone after a restart. Set them again
  in `on join`.
- `delete board` also removes the board from memory.

## Errors you may see

- `'x y' is not a valid objective name`: use letters, numbers and `_ - . +`.
- `objective 'x' is already registered` / `team 'x' is already registered`.
- `team 'x' is not registered`: register the team first.
- `'plaid' is not a team color`.
- `board line must be a whole number from 1 to 15`.
- `a board has at most 15 lines`.
