# More events

These events give extra values with `event-<name>`. For example,
`event-motd` in `on server list ping`. Some values can be changed with
`set event-<name> to <value>`. Using a value the event does not have is a
load error, and the error lists the values that exist.

```
on server list ping:
    set event-motd to "Welcome! %event-players% online"
    set event-max-players to 100

on sign change:
    if event-line-1 contains "bad":
        cancel event
    set event-line-4 to "[%player%]"

at 18:00 in real time:
    broadcast "It is 6 pm UTC."
```

## Entities in event values

Carve has no entity type yet. An entity value (`event-entity`, `event-vehicle`,
`event-target`, `event-attacker`) is the player when the entity is a player.
Otherwise it is the entity's number id. Blocks are locations. Items are items.

## Events

`cancel event` works where the last column says yes.

| Trigger | Also written as | Values | Cancel |
|---|---|---|---|
| `on server list ping:` | `on ping:`, `on list ping:` | `event-motd`\*, `event-max-players`\*, `event-players`\*, `event-hostname`, `event-address` | no |
| `on sign change:` | `on sign edit:` | `player`, `event-block`, `event-line-1`\* to `event-line-4`\* | yes |
| `on smelt:` | `on furnace smelt:` | `event-block`, `event-source`, `event-result` | yes |
| `on start smelt:` | `on furnace start smelt:` | `event-block`, `event-source`, `event-time`\* | yes |
| `on sponge absorb:` | | `event-block` | yes |
| `on spread:` | `on block spread:` | `event-block`, `event-source-block`, `event-world` | yes |
| `on spawn change:` | `on world spawn change:` | `event-world`, `event-previous-location`, `event-new-location` | no |
| `on swim toggle:` | `on toggle swim:` | `player` (if a player), `event-entity`, `event-swimming` | yes |
| `on tame:` | `on entity tame:` | `player` (the owner), `event-entity`, `event-owner` | yes |
| `on target:` | `on entity target:` | `event-entity`, `event-target` | yes |
| `on shoot:` | `on shoot bow:` | `event-entity`, `event-weapon`, `event-force` | yes |
| `on throwing of an egg:` | `on egg throw:` | `player`, `event-hatching`\*, `event-hatches`\*, `event-hatch-type` | yes |
| `on tool change:` | `on player tool change:` | `player`, `event-previous-slot`, `event-new-slot` | yes |
| `on vehicle collision:` | | `event-vehicle`, `event-entity`, `event-block` | yes |
| `on vehicle create:` | | `event-vehicle` | yes |
| `on vehicle damage:` | | `event-vehicle`, `event-damage`, `event-attacker` | yes |
| `on vehicle destroy:` | | `event-vehicle`, `event-attacker` | yes |
| `on vehicle enter:` | | `event-vehicle`, `event-entity` | yes |
| `on vehicle exit:` | | `event-vehicle`, `event-entity` | yes |
| `on vehicle move:` | | `event-vehicle`, `event-from`, `event-to` | yes |
| `on weather change:` | | `event-world`, `event-weather` (`rain` or `clear`) | yes |
| `on weather change to rain:` | `... to rainy:` | same | yes |
| `on weather change to clear:` | `... to sun:` | same | yes |
| `on world init:` | | `event-world` | no |
| `on world load:` | | `event-world` | no |
| `on world save:` | | `event-world` | yes |
| `on world unload:` | | `event-world` | yes |
| `on zombie break door:` | `on break door:` | `event-entity`, `event-block` | yes |
| `on sheep regrow wool:` | | `event-entity` | yes |
| `on slime split:` | | `event-entity`, `event-count`\* | yes |
| `on vault display item:` | | `event-block`, `event-world`, `event-item` | yes |
| `on villager career change:` | | `event-entity`, `event-profession`, `event-reason` | yes |
| `at 14:30 in real time:` | `at 6pm in real time:` | none | no |

\* can be changed with `set`.

`player` exists only where the table lists it. The other events have no
player.

## Real-time triggers

`at 14:30 in real time:` runs once a day at that time. Write `HH:MM` (24 hour)
or `6pm`. The clock is UTC, because plugins cannot read the server's time
zone. It checks once per tick, so it can run up to 50 ms late. A time that
passes while the server is down does not run.

## Limits

- Pumpkin does not fire `sheep regrow wool`, `slime split`, `vault display item`
  or `villager career change` yet. They are wired and will work when it does.
- `on shoot` runs only for mob bows and crossbows. `on spread` only for fire.
  `on spawn change` only for `/setworldspawn`.
- `on zombie break door` runs for any mob that breaks a door. Carve cannot ask
  an entity id for its type.
- Not possible: stonecutter recipe select, spectate, stop using item, unknown
  command, and world border change events. The plugin API has no such events.
