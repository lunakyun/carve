# The `/carve` command

`/carve` (or `/cv`) manages scripts while the server runs. It works for
players and from the server console. Every reply starts with `[Carve]`.

| Command | Permission | What it does |
|---|---|---|
| `/carve help` | none | Lists the subcommands you are allowed to use. |
| `/carve reload [all\|scripts]` | `carve:reload` | Reloads every enabled script. On any error the old scripts keep running. |
| `/carve reload <script>` | `carve:reload` | Reloads one file. On an error only that file keeps its old version. |
| `/carve enable <script\|all>` | `carve:enable` | Enables a disabled script and loads it right away. |
| `/carve disable <script\|all>` | `carve:enable` | Disables a script and stops its triggers right away. |
| `/carve list [enabled\|disabled]` | `carve:admin` | Scripts with their trigger and error counts. |
| `/carve info` | `carve:admin` | Versions and counts: scripts, triggers, commands, functions, variables, data folder. |
| `/carve info <script>` | `carve:admin` | A script's triggers, commands, functions and options. |
| `/carve errors [<script>]` | `carve:admin` | Errors of the last load or reload, with `file:line` and "did you mean". |
| `/carve vars list [<prefix>]` | `carve:vars` | Variables (the first 20), optionally only those starting with a prefix. |
| `/carve vars get <name>` | `carve:vars` | One variable, with its type. |
| `/carve vars set <name> <value>` | `carve:vars` | Set a global variable. Saved like any other change. |
| `/carve vars delete <name>` | `carve:vars` | Delete a variable, or a whole list with `name::*`. |
| `/carve run <effect>` | `carve:run` | Runs one line of Carve now, as you. Good for testing. |
| `/carve timings [on\|off\|reset]` | `carve:admin` | Measures how long triggers take. `/carve timings` shows the top 10. |

Script names can be written with or without `.carve`: `/carve reload homes`.

## Permissions

`carve:admin` allows every subcommand. The others allow one area each. All of
them are registered when Carve loads, with op level 2 as the default, so ops
have them. The console can always use everything. `/carve help` is open to
everyone, but it only lists what the player may use.

## Reloading

- **Reload everything:** `/carve reload` (also `all` or `scripts`) compiles
  every enabled script together. If any file has an error, nothing changes
  and the errors are shown. `on load` triggers run after a successful full
  reload.
- **Reload one file:** `/carve reload <script>` compiles that one file with
  the others. If it has an error, the other scripts and the old version of
  that file keep running. `on load` doesn't run for a single file.
- **New commands** (`command /x`) can't be added while the server runs
  (Pumpkin's plugin API only allows that at load). Carve says which commands
  need a restart.

The summary line shows how many scripts and triggers were loaded and how long
it took.

## Enabling and disabling

This follows Skript's convention: a disabled script is the same file with a
`-` in front of its name (`-homes.carve`). Carve ignores those files when it
loads. `/carve disable homes` renames the file and stops its triggers at once.
`/carve enable homes` renames it back and loads it.

If other scripts need the one you disable (they call one of its functions),
Carve refuses and renames the file back. Enabling a script with errors keeps
it enabled on disk, shows the errors, and doesn't run it until it loads.

## Variables

```
/carve vars set coins::Steve 100
/carve vars get coins::steve
/carve vars list coins::
/carve vars delete coins::*
```

- **Values:** `100` becomes a number, `true` or `false` a boolean, and
  anything else text. Quotes are optional: `"Hello there"`.
- **Names:** braces are optional, and names are lowercased like in scripts.
- **Not supported:** local variables (`{_x}`) and whole lists can't be set
  here.

## Running a line

```
/carve run broadcast "Restart in 5 minutes"
/carve run give 1 diamond to player
/carve run send "%{coins::%uuid of player%} ? 0%"
```

The line runs as a command trigger: `player` is you, and from the console
there is no player. Functions from your scripts can be called. If the line has
a mistake, you get the error and the "did you mean" suggestions right away.

## Timings

```
/carve timings on
/carve timings
/carve timings reset
/carve timings off
```

While timings are on, Carve counts each trigger's runs and measures the total
and the longest run. Runs are measured per slice (the part between two
`wait`s), so waiting doesn't count as time spent. Timings cost a little speed,
so turn them off when you're done. `off` also clears them.

## Tab completion

Subcommand names complete in the chat box. Script names and values don't
complete yet. Pumpkin 0.2.2 crashes when it asks a plugin for completions
(its WASM host calls into Tokio from a thread without a runtime), so Carve
doesn't offer them. The completion logic is ready in `carve-core` for when
Pumpkin fixes this.
