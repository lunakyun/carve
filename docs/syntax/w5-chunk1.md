# Cooldowns, drops, mobs and potion flags

## Command cooldowns

```
command /heal:
    cooldown: 1 minute
    cooldown message: Wait %remaining time% before healing again.
    cooldown bypass: heal.nocooldown
    trigger:
        heal player
        if player's health is 20:
            cancel the cooldown
```

- `cooldown:` takes a time span (`10 seconds`, `1 minute and 30 seconds`, `2.5 hours`). It is per player; the console has no cooldown.
- `cooldown message:` may use `%remaining time%`, `%elapsed time%` and `%cooldown time%`. Without it the player sees "You have to wait ... before using this command again".
- A player with the `cooldown bypass:` permission never waits.
- `cancel the cooldown` in the trigger makes this use not count; `uncancel the cooldown` undoes that.
- Cooldowns live in memory and reset when the server restarts. `cooldown storage:` is a script error.

## Failing a command

```
command /pay <player> <number>:
    trigger:
        if arg-2 < 1:
            fail the command execution with the error message "Pay at least 1."
```

The message goes to the sender and the trigger stops there. It only works inside a command.

## Drops and kept inventories

```
on break of stone:
    cancel the drops
on break of oak log:
    cancel the drops of xp
on death:
    keep the inventory and experience
```

- `cancel the drops`, `cancel the drops of items` and `cancel the drops of experience` work in `on break` and `on death`.
- `keep the inventory`, `keep the experience` and both together work in `on death`. Carve switches the world's `keep_inventory` rule on for that death and back after the respawn, so other deaths are not changed.

## Dye and glint of the held item

```
dye player's tool red
color player's tool rgb(30, 200, 90)
force player's tool to glint
make player's tool stop glinting
reset the enchantment glint override of player's tool
```

Dye works on leather armor and other dyeable items. Only the item in the player's main hand can be changed.

## Creepers, TNT and fireworks

```
ignite creeper {_c}
stop ignition of creeper {_c}
instantly explode creeper {_c}
detonate last spawned entity
launch a firework at location(0, 70, 0)
```

- `detonate` explodes creepers, primed TNT and TNT minecarts at once; other entities are ignored.
- `launch firework` sends up a plain rocket that bursts after about a second. Pumpkin ignores firework data on rockets made this way, so an effect (`launch firework effect {_e} at ...`) and `timed` are accepted but have no effect yet.

## Dropping items

```
drop 3 diamonds
drop stone at location(1, 70, 1) without velocity
```

Without `at`, the items drop at the player of the event.

## Whitelist and world border

```
enforce the whitelist
expand the world border to 500
shrink the world border by 100 over 2 seconds
expand the radius of the world border of world "world" by 10
```

`enforce the whitelist` kicks every online player who is not on the whitelist and not an op, when the whitelist is on.

## Leashes, babies, breeding and persistence

```
leash {_cow} to player
unleash {_cow}
make last spawned entity a baby
make {_cow} an adult
make {_cow} unbreedable
make {_cow} persistent
```

- Leashes are written as the mob's `leash` data. Pumpkin 0.2.2 keeps that data but does not attach the lead while the mob is loaded, so players see no lead yet.
- `make ... unbreedable` gives the mob a 5 minute breeding cooldown (age 6000); `breedable` clears it.

## Server icon

```
command /icon:
    trigger:
        load the server icon from file "icon.png"
on server list ping:
    set the server icon to the last loaded server icon
```

The file is read from Carve's data folder (`plugins/data/carve`) and must be a 64x64 PNG.

## Signs, pathfinding and potion effects

```
make block at location(2, 70, 2) have glowing text
make {_zombie} pathfind to player at speed 1.5
make {_zombie} stop pathfinding
make player's potion effects ambient
hide the particles of player's potion effects
hide the potion icon of player's potion effects
make player's potion effects infinite
```

Pathfinding keeps steering the mob for up to a minute, until it arrives or `stop pathfinding` runs. The potion lines change the effects the player has now.

## Conditions

```
if {_cow} can age:
if {_golem} can't grow up:
```

`can age` is true for mobs that have a baby form.

## Prepare craft

```
on preparing craft:
    send "You can make %event-item%"
```

Runs when a recipe matches in a crafting grid. `event-item` is the result and `event-recipe` the recipe id.
