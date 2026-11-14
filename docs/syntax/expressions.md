# Expressions

Expressions are the values you put into effects and conditions: numbers,
text, variables, the player, and so on.

## Kinds of values

| Kind | Example | Printed as |
|---|---|---|
| number | `5`, `2.5` | `5`, `2.5` (at most 4 decimals) |
| text | `"hello"` | `hello` |
| player | `player` | the player's name |
| boolean | only from `variables.json` | `true` / `false` |
| not set | an unset variable | `<none>` |

## Numbers

```
set {_a} to 10
set {_b} to -2.5
```

Write numbers with a `.` for decimals. There is no `1e5` style.

## Math

```
set {_total} to {_price} * {_amount} + 5
set {_half} to ({_a} + {_b}) / 2
```

- `+`, `-`, `*` and `/`, with the usual order: `*` and `/` before `+` and `-`.
- Brackets `( )` group.
- `-` in front negates: `-{_x}`.
- Math needs numbers. Text in quotes or a `<text>` argument is a load error. A variable
  that holds text is read as a number when the line runs, so `"5"` works but
  `"abc"` is a runtime error.
- Dividing by zero is a runtime error.
- Math on numbers only, like `60 * 20`, is worked out once when the script
  loads.

## Text

```
send "Hello, %player%! You have %{coins::%player%}% coins."
send "Sale: 50%% off"
send "She said ""hi"""
```

- Text goes between double quotes.
- `%...%` inserts the value of any expression. Inside you can use variables,
  `player`, math, and so on: `"%{_a} + 1%"`.
- `%%` is a plain `%` sign.
- `""` is a `"` inside text.
- Color codes with `&` work: `&a` green, `&c` red, `&e` yellow, `&l` bold,
  and the rest of the Minecraft color codes.

## Variables

```
{coins}
{coins::%player%}
{_temp}
```

`{name}` is a global variable, saved to disk. `{_name}` is a local variable
that lives for one run. A variable that was never set is `<none>`. See
[variables](../variables.md).

## `player`

```
send "Hi %player%"
heal player
```

The player of the event, or the player who ran the command. In text it
prints the player's current name. If the player is offline (for example after
a `wait`), or there is no player (the console ran the command), it is
`<none>`.

You can write `the player`. A `the` in front of any expression is ignored.

## `name of %player%`

```
send "Your name is %name of player%"
set {owner} to name of player
```

The player's name as text. Use it when you want to store the name, not the
player. If the player is offline, it is `<none>`.

## `message`

```
on chat:
    send "You said: %message%" to player
```

The chat text. Only in `on chat:`. Anywhere else it is a load error.

## `arg`, `arg-1`, `arg-2`, ...

```
command /pay <player> <number>:
    trigger:
        send "Paying %arg-2% to %arg-1%"
```

The command's arguments, counted from 1. `arg` and `argument` are the same as
`arg-1`. An optional argument that was left out is `<none>`. Using an
argument number the command doesn't have is a load error. Only in commands.

The type of each argument comes from the command line: `<number>` gives a
number, `<player>` a player, `<text>` text.
