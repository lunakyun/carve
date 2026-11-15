# Variables

Variables hold values between lines, between runs, and across restarts.

```
on join:
    add 1 to {joins::%player%}
    send "You joined %{joins::%player%}% times." to player
```

## Global and local

| Kind | Written as | Lives for | Saved to disk |
|---|---|---|---|
| global | `{name}` | until deleted | yes |
| local | `{_name}` | one run of a trigger | no |

A local variable starts empty every time the trigger runs. It is kept across a
`wait` in the same run. Two runs never share locals, even of the same trigger.

## Names

- A name can hold almost any text: `{spawn-message}`, `{stats::total joins}`.
- Names are not case sensitive. `{Coins}` and `{coins}` are the same variable.
- `%...%` inside a name inserts a value: `{coins::%player%}`. A player is
  inserted as their name, so this is `{coins::steve}` for Steve.

Because the player's name is used, a player who changes their name starts
over with new variables.

## Lists

Carve has no real list type, but `::` in a name works like one by convention:

```
set {home::%player%} to "spawn"
set {kills::%player%} to 0
delete {kills::*}
```

`delete {kills::*}` deletes every variable whose name starts with `kills::`.
It works for locals too: `delete {_list::*}`. The `::*` form only works with
`delete`. You can't loop over a list yet.

## Values

A variable holds one of:

- a number,
- text,
- a player,
- a boolean (`true` / `false`). Scripts can't make one, but the saved data
  can hold one.

A variable that was never set is `<none>`. Setting a variable to `<none>`
deletes it:

```
set {a} to {never-set}   # deletes {a}
```

## Players in variables

```
set {last-joined} to player
```

A stored player keeps the player's UUID and name. When the variable is used,
Carve checks whether that player is online right now. If they are offline,
effects on them are skipped and `name of` gives `<none>`. Printing the
variable in text prints the stored name.

## Saving

Carve keeps everything it has to remember in one file, `carve.redb`, in the
plugin's data folder. Global variables are saved there:

- about 5 seconds after a change (one write for many changes),
- when you run `/carve reload`,
- and when the server stops.

Only the variables that changed are written, all in one transaction. A
transaction either happens completely or not at all, so a crash (or a power
cut) never leaves a half-written file: after it, the variables are as they
were at the last write.

`carve.redb` is a binary database (made with [redb](https://www.redb.org)),
not something to edit by hand. Stop the server and copy the file to back it
up. Besides variables it holds the bounds and the players that have joined
before (for `on first join`).

### Files from older versions

Before 0.3 Carve wrote `variables.json`, `bounds.json` and `first_join.txt`.
The first time a newer Carve starts and `carve.redb` is empty, it reads those
files, writes everything into `carve.redb`, logs how much it imported and
renames the files to `variables.json.old`, `bounds.json.old` and
`first_join.txt.old`. A file that is not valid is renamed to `<name>.broken`
and Carve logs an error and starts without it. If `carve.redb` already has
data, the old files are left alone.
