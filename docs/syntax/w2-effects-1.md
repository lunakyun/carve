# Wave 2 effects, part 1

## Connect (send a player to another server)

```
connect player to server "play.example.net"
connect player to "play.example.net:25570"
connect player to the server "play.example.net" on port 25570
```

The default port is 25565. A bad address is a runtime error. In the plugin this uses the
player transfer call. Other hosts run the vanilla `transfer` command.

## Force respawn

```
force player to respawn
force respawn of player
```

## Hide and reveal players

```
hide player from {_viewer}
reveal player to {_viewer}
```

Players only. The first player is the one who is hidden. The second is the one who stops seeing them.

## Equip

```
equip player with diamond helmet
```

The item goes to the slot it belongs in: helmets, chestplates, leggings and boots to the armor
slots, a shield to the off hand, anything else to the main hand. Uses the vanilla `item replace`
command. One item per effect.

## Copy into variable

```
copy {a} into {b}
copy {list::*} to {other::*}
```

A list copies key by key and replaces the old target list. After the copy the two are
independent. Copying an unset variable clears the target. Both sides must be lists or both single values.

## Not possible on Pumpkin

Allay duplicate, leash drop, aging, bone meal, fishing lure, cancel active item, charge creeper,
clear entity storage, boosting firework, brewing fuel, dance, enderman teleport, force attack,
force eating, force panda on back and the equippable component entries. Pumpkin's plugin API
has no way to reach them.
