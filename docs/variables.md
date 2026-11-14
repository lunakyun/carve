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
- a boolean (`true` / `false`). Scripts can't make one, but `variables.json`
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

Global variables are saved in `variables.json` in the plugin's data folder:

- about 5 seconds after a change (one save for many changes),
- and when the server stops.

The file is written to `variables.json.tmp` first and then renamed, so a crash
during the save does not leave a half-written file.

The file is plain JSON, sorted by name:

```json
{
  "joins::steve": 3,
  "last-joined": {
    "player": {
      "name": "Steve",
      "uuid": "069a79f4-44e9-4726-a5be-fca90e38aaf5"
    }
  },
  "motd": "Have fun!"
}
```

You can edit it while the server is stopped. If the file is not valid JSON,
Carve moves it to `variables.json.broken`, logs an error and starts with no
variables, so you can fix it by hand.
