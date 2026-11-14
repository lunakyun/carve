# Scoreboards, boss bars, titles and the tab list

Everything here is an effect. Nothing here reads a value back yet.

`all players` (or `everyone`) works wherever a player is allowed, in
the title, action bar, tab list and boss bar show/hide effects.

## Sidebar scoreboard

Each player has their own sidebar. It is made the first time you set a
title or a line. Line 1 is at the top. Lines go from 1 to 15. The red
score numbers are hidden.

```
set title of player's scoreboard to "&6My Server"
set line 1 of player's scoreboard to "&7Coins: &e%{coins::%player%}%"
set line 2 of scoreboard of player to "&7Kills: &e5"
clear line 2 of player's scoreboard
clear player's scoreboard
```

| Syntax | What it does |
|---|---|
| `set title of <player>'s scoreboard to <text>` | Sets the sidebar title |
| `set line <n> of <player>'s scoreboard to <text>` | Sets one line |
| `clear line <n> of <player>'s scoreboard` | Removes one line |
| `clear <player>'s scoreboard` | Removes the whole sidebar |

Other ways to write the same thing: `scoreboard of <player>` instead of
`<player>'s scoreboard`, `sidebar` instead of `scoreboard`,
`<player>'s scoreboard title`, and `hide`, `reset`, `remove` or `delete`
instead of `clear`.

A line number outside 1 to 15 is a runtime error. Two lines with the same
text are fine. Bedrock players have no sidebar yet.

## Boss bars

A boss bar has a text id that you choose. Use the same id later to change
it. Making a bar does not show it to anyone. Making a bar with an id that
exists replaces the old one.

```
create bossbar "raid" with title "&cRaid" with color red with style notched 10 with progress 100
show bossbar "raid" to player
set progress of bossbar "raid" to 40
set title of bossbar "raid" to "&cRaid: wave 2"
hide bossbar "raid" from player
remove bossbar "raid"
```

| Syntax | What it does |
|---|---|
| `create bossbar <id> [with title <text>] [with color <color>] [with style <style>] [with progress <0-100>]` | Makes a bar |
| `set progress of bossbar <id> to <number>` | 0 to 100. Out of range numbers are cut to the limit |
| `set title of bossbar <id> to <text>` | New title |
| `set color of bossbar <id> to <color>` | New color |
| `set style of bossbar <id> to <style>` | New style |
| `show bossbar <id> to <player>` | The player sees it |
| `hide bossbar <id> from <player>` | The player does not |
| `remove bossbar <id>` | Hides it from everyone and deletes it |
| `add <player> to bossbar <id>` / `remove <player> from bossbar <id>` | Same as show and hide |
| `add <flag> flag to bossbar <id>` / `remove <flag> flag from bossbar <id>` | Turns a flag on or off |

Words you can use in place of the ones above: `boss bar` for `bossbar`,
`titled`, `colored` and `styled` for `with title` and so on, and
`named`, `called` or `with id` before the id. The title defaults to the id,
the color to pink, the style to solid and the progress to 100.

Colors: `pink`, `blue`, `red`, `green`, `yellow`, `purple`, `white`.

Styles: `solid`, `notched 6`, `notched 10`, `notched 12`, `notched 20`.
`segmented 6` and `notches 10` also work.

Flags: `darken sky`, `fog`, `dragon bar`.

A color or style can also be a text, like `with color {_c}`. It is checked
when the line runs. A word that is not a color or style is an error when
the script loads.

If you use an id that has no bar, Carve writes a warning and does nothing.

## Titles and the action bar

```
send title "Round 1" to player
send title "Round 1" with subtitle "Fight!" to player for 3 seconds
send title "Hi" to player for 2 seconds with fade in 5 ticks and fade out 1 second
send title "Server restart" to all players for 5 seconds
clear title of player
reset title of player
send actionbar "Hello" to player
send action bar "Hello" to all players
```

`for <time>` is how long the title stays. The defaults are fade in
0.5 seconds, stay 3.5 seconds and fade out 1 second. A title without
`for` or `fade` keeps whatever times the player already has.

`clear title` takes the title off the screen. `reset title` also puts the
times back to the defaults.

## Tab list header and footer

```
set tab list header of player to "&6My Server"
set tab list footer of player to "&7play.example.com"
set tab list header for all players to "&6Welcome"
set tab list header and footer of player to "same text on both"
reset tab list of player
clear tab list footer of player
```

Setting only the header keeps the footer as it was. `reset` and `clear` make
the part empty. `tab list` can also be written `tablist`.
`<player>'s tab list header` works too.

## Not done yet

- Reading a bar's progress, title, color or style.
- Adding to or removing from a bar's progress with `add` and `remove`.
- Scoreboard tags, teams, and scores that are not on the sidebar.
- Sidebars for Bedrock players.
