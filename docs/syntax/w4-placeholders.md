# Placeholders (syntax)

Overview and the plugin protocol are in [placeholders](../placeholders.md).

## Expressions

```
placeholder %text% [for %player%]
parse placeholders in %text% [for %player%]
```

`for` can also be written `from` or `of`. The player defaults to the event's
player.

## Trigger

```
on placeholder request [for prefix "<prefix>"]:
    the identifier    # text after "<prefix>_"
    the prefix
    set the result to ...
```

## Example

```
on placeholder request for prefix "shop":
    if the identifier is "coins":
        set the result to "%{coins::%player%} ? 0%"

command /me:
    trigger:
        send parse placeholders in "%%player_name%% has %%shop_coins%% coins" for player
```
