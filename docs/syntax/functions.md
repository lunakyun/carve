# Functions and options

## Functions

A function is a named block you can call from any script. Functions are
checked when scripts load: a call with the wrong number of arguments, a wrong
argument type or an unknown name is a load error with `file:line`.

```
function coins(p: player) :: number:
    return {coins::%{_p}%}

function pay(from: player, to: player, amount: number = 10):
    remove {_amount} from {coins::%{_from}%}
    add {_amount} to {coins::%{_to}%}
    send "You got %{_amount}% coins from %{_from}%." to {_to}

command /pay <player> <number>:
    trigger:
        if coins(player) < arg-2:
            send "You only have %coins(player)% coins."
            stop
        pay(player, arg-1, arg-2)
```

### The first line

`function name(param: type, param: type = default) :: return type:`

- **Name:** letters, digits and `_`. Names ignore case, and each name can only
  be used once across all scripts.
- **Parameter types:** `text` (or `string`), `number` (or `integer`),
  `player`, `boolean`, `item`, `location`, `world` and `object` (anything).
- **Defaults:** a parameter with `= value` can be left out of a call. Every
  parameter after it needs a default too.
- **Return type:** `:: type` says the function gives back a value. Leave it out
  for a function that only does things.

### Inside a function

- Parameters are local variables: `name: text` is `{_name}`.
- Locals belong to one call, so a function can't see or change the caller's
  `{_...}` variables.
- `return value` gives back a value. Its type must match the return type.
- `return` or `stop` ends a function that returns nothing.
- Event values don't exist inside functions, so pass what you need as
  parameters. That covers `player`, `message`, `arg` and `cancel event`.
- `send "..."` without `to` goes to whoever started the run.

### Calling

- **As a value:** `coins(player)`, or `"%coins(player)%"` inside text. Text
  arguments work inside text too: `"%greet("Steve")%"`.
- **On its own line:** `pay(player, arg-1, 5)`. The result, if any, is dropped.

A function called on its own line may `wait`. A function used as a value can't,
because the value is needed right away. That is a runtime error at the line of
the `wait`.

Functions can call themselves (recursion), up to 64 calls deep. Every
instruction counts towards the same runaway guard as the trigger that called
it.

### Reload

Functions are compiled together with all scripts. A broken line in a function
is skipped and the rest of it loads. If the function's first line (or a
default value) is broken, the function doesn't load, and each call to it is
reported as an error where it is.

## Options

```
options:
    prefix: &8[&6Shop&8]&r
    price: 25

command /buy:
    trigger:
        send "{@prefix} That costs {@price} coins."
```

- `{@name}` is replaced with the option's text before the line is read. It
  works anywhere in the file: in text, numbers or whole effects.
- Options belong to their file. An unknown `{@name}` is a load error.
