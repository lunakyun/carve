# Item, mob and level events

These events cover items, mobs, game modes and levels. Each one gives you
some event values. Event values only exist in the event that offers them.

Pumpkin defines all of these events in its plugin API. Some may not be fired
by the server yet, so check each one in game.

## Events

| Event | Names that work | Event values | Can cancel |
|---|---|---|---|
| Drop | `on drop`, `on item drop` | `event-item`, `player` | yes |
| Pick up | `on pick up`, `on pickup` | `event-item`, `player` | yes |
| Consume | `on consume`, `on eat`, `on drink` | `event-item`, `player` | yes |
| Projectile hit | `on projectile hit` | `event-location`, `hit player` | yes |
| Mob spawn | `on mob spawn`, `on spawn` | `event-entity`, `event-location`, `spawn reason` | yes |
| Gamemode change | `on gamemode change` | `old gamemode`, `new gamemode`, `player` | yes |
| Craft | `on craft` | `event-recipe`, `event-item`, `player` | yes |
| Level change | `on level change` | `old level`, `new level`, `player` | no |

## Event values

- `event-item` is an item. `event-item is diamond` works like any item check.
- `event-entity` is the entity type as text, like `zombie` or `zombie_villager`.
  Use `event-entity is [a|an] zombie` or `event-entity is not a creeper`.
- `event-location` is a location.
- `spawn reason` is text such as `natural` or `spawner`.
- `old gamemode` and `new gamemode` (also written `old game mode`,
  `previous gamemode`) are mode names: `survival`, `creative`, `adventure`, `spectator`.
- `old level` and `new level` are numbers.
- `event-recipe` (or `recipe id`) is the recipe id without `minecraft:`.
- `hit player` is the player a projectile hit. It is `<none>` when it hit
  something else.

## Filters in the event line

```
on spawn of zombie:
    broadcast "a zombie spawned"

on spawn of a zombie villager:
    cancel event

on gamemode change to creative:
    send "welcome to creative"
```

`of <mob>` only works with `on spawn` and `on mob spawn`.
`to <mode>` only works with `on gamemode change`.

## Examples

```
on drop:
    if event-item is diamond:
        cancel event
        send "you can't drop diamonds"

on eat:
    if event-item is golden_apple:
        send "yum"

on level change:
    send "level %old level% to %new level%"

on mob spawn:
    if event-entity is creeper:
        cancel event
```

## Limits

- `player` does not exist in `on mob spawn` and `on projectile hit`. Nobody
  started them. Use `hit player` for projectiles.
- `on pick up` only runs when a player picks the item up. Mobs picking up
  items are ignored.
- `on projectile hit` has no projectile type, because the API only gives an id.
- `on craft` only gets the recipe id. `event-item` is filled from that id,
  which is right for recipes named after their result (like `stick`) and
  wrong for the others.
- `on level change` can't be cancelled.
