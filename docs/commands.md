# Commands

Scripts can add their own commands.

```
command /heal [<player>]:
    permission: heal
    description: Heal yourself or another player
    trigger:
        if arg is set:
            heal arg
            send "Healed %arg%."
        else:
            heal player
            send "You feel much better."
```

## The command line

```
command /name <type> [<type>]:
```

- The `/` is optional: `command heal:` works too.
- The name can use letters, digits, `_` and `-`. It is lowercased, so
  `/Heal` and `/heal` are the same command.
- Each argument is a type in `< >`. Put it in `[ ]` to make it optional.

## Arguments

| Type | Also | Gives | The player types |
|---|---|---|---|
| `<text>` | `<string>` | text | one word, or the rest of the line if it is the last argument |
| `<number>` | `<integer>` | number | a number like `5` or `2.5` |
| `<player>` | | player | the name of an online player |

Rules:

- Optional arguments must come after all required ones.
- If the last argument is `<text>`, it takes everything that is left, spaces
  and all. `/say <text>` gets the whole message.
- Inside the trigger, use `arg-1`, `arg-2`, and so on. `arg` is `arg-1`.
- An optional argument that was left out is `<none>`. Check it with
  `if arg-2 is set:`.
- `<integer>` is the same as `<number>`. It does not reject `2.5`.

When the arguments are wrong, the command does not run and the player sees
one of these:

```
Usage: /pay <player> <number>
'abc' is not a number. Usage: /pay <player> <number>
Player 'Notch' is not online
Too many arguments. Usage: /pay <player> <number>
```

## Entries

| Entry | Required | What it does |
|---|---|---|
| `trigger:` | yes | The code that runs. |
| `permission: node` | no | Only players with this permission can use the command. |
| `description: text` | no | Shown in the server's command help. Without it, the usage line is used. |
| `aliases: /h, /hi` | no | Other names for the command. Like the command itself, they need a restart to appear. |
| `usage: text` | no | Sent instead of the generated usage line when the arguments are wrong. |
| `executable by: players` | no | `players` or `console`. Others get an error message. |

There is no `cooldown:` entry yet. Store the time in a variable instead.

## Permissions

```
command /fly:
    permission: fly
    trigger:
        execute console command "say %player% wants to fly"
```

- **Namespacing.** A node without a namespace is put in Carve's namespace.
  `heal` becomes `carve:heal`. `carve:heal` and `other:thing` stay as they are.
  The same applies to `has permission` conditions.
- **Registration.** When the plugin loads, Carve registers each `carve:` node
  used in a command's `permission:` entry. Ops (level 2 and up) have them by
  default. Anyone else needs to be given the node.
- **Unregistered nodes are denied, even for ops.** Pumpkin denies any node
  that was never registered, unless it was given to the player directly. This
  matters in two cases:
  - A node from another plugin (`other:thing`) works only if that plugin
    registered it.
  - Nodes are registered only at load. A node you add with `/carve reload` is
    not registered until the next restart, so until then only players who
    were given it directly can use the command.
- **The console** can always run script commands. The permission check only
  applies to players.
- Without a `permission:` entry, anyone can use the command. (All script
  commands also sit behind `carve:command`, which everyone has by default.)

A player without permission sees:

```
You don't have permission to use this command.
```

## Reloading and restarting

```
/carve reload
/cv reload
```

`/carve reload` needs `carve:admin` (op level 2 by default). It reads every
script again. If any file has an error, nothing changes: the old scripts stay
active and the errors are shown to you and written to the log.

**New commands need a restart.** Pumpkin lets a plugin register commands only
while it loads, and it can't remove them. So after a reload:

- An existing command runs its new code right away.
- A new command is listed in the reload message as needing a restart.
- A command you removed from the scripts still exists until the restart. It
  answers "/name is no longer in any script. Restart the server to remove it."

If you change the arguments of an existing command, restart too. Pumpkin keeps
the shape the command had at load time, such as whether it takes arguments at
all, so a reload alone can leave the command unusable in its new form.

## Messages to the sender

Inside a command, `send "..."` without `to` replies to whoever ran it. That is
the player, or the console. When a console run reaches a `wait`, replies after
the wait go to the server log.
