# Vectors, server facts and more

## Vectors

A vector is three numbers: x, y and z. It prints as `x: 1, y: 2, z: 3`. It can be stored in variables
(also saved to disk), passed to functions (`v: vector`) and compared with `is`.

Yaw and pitch use the Minecraft way: yaw 0 faces south (+z), yaw 90 faces west (-x), pitch 90 looks
straight down.

### Make a vector

```
vector(1, 2, 3)
vector from 1, 2, 3
a vector from 1, 2 and 3
vector from yaw 90 and pitch 0
vector from direction of player          # where the player looks
vector from direction north              # north, south, east, west, up, down
vector from location(1, 2, 3)            # also: vector of player
vector from {a} to {b}                   # b - a, also: vector between {a} and {b}
random vector                            # length 1
spherical vector radius 2, yaw 0, pitch 0
cylindrical vector radius 3, yaw 90, height 5
```

### Math

```
{a} + {b}      {a} - {b}      -{a}
{a} * 2        2 * {a}        {a} / 2
{a} * {b}      # each part times each part
{loc} + {v}    {loc} - {v}    {loc} offset by {v}
{a} dot {b}    {a} cross {b}
{a} projected onto {b}         projection of {a} onto {b}
angle between {a} and {b}      # degrees
length of {v}   vector length of {v}   squared length of {v}
normalized {v}  {v} normalized
```

Dividing by zero is a runtime error. Normalizing a zero vector gives a zero vector.

### Parts

```
x of {v}     {v}'s y     z component of {v}
yaw of {v}   pitch of {v}
```

These are read only. To change one part, build a new vector:
`set {v} to vector(5, y of {v}, z of {v})`.

### To a location

```
location from vector {v} in world "nether"
{v} to location
{v} to location in world "world"
```

## Server facts

```
tps          server tps          ticks per second     # current TPS
mspt                                                   # milliseconds per tick
server version      minecraft version      version of server      version string of server
view distance of server      server view distance
```

TPS is one current value, not the 1, 5 and 15 minute averages. The Minecraft version is fixed
when Carve is built.

## Players

```
time played of player          # a timespan, from the play time statistic
time lived of player           # a timespan
view distance of player        client view distance of player
yaw of player                  pitch of player
```

```
if time played of player > 10 hours:
    send "veteran"
```

## Worlds

```
loop all worlds:
    send "%loop-value%"
set {w::*} to all worlds
environment of world "nether"        # normal, nether or the end
world environment of world of player
```

## Items

```
3 of diamond              # an amount of an item
type of 5 diamonds        # diamond (amount 1)
```

## whether

`whether` turns a comparison into true or false:

```
set {t} to whether {a} is 5
set {t} to whether {a} is not set
set {t} to whether {a} is greater than 3     # also: is less than, is at least, is at most, >, <, >=, <=, =, !=
```

## Not possible

Quaternions, villager and warden data, vehicles, world seeds and item components are not reachable from
the plugin API, or need entity values that Carve does not have yet.
See `docs/skript-parity.md` for each entry.
