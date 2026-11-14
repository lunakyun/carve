# Ray tracing, worlds, chunks and tags

## Ray trace

```
set {_r} to ray trace from player with max distance 50
set {_r} to ray trace from location(0, 70, 0, world "world") in direction vector(0, -1, 0)
set {_r} to ray trace from player including fluids ignoring entities
```

- From a player the ray starts at the eyes and goes where the player looks. From a location, give `in direction`.
- Default distance is 20 blocks, the longest is 512.
- `including fluids` makes water and lava count. `ignoring blocks` or `ignoring entities` skips that kind.
- The result is the nearer of the first block and the first entity. If nothing is hit, `{_r}` is not set.

Read it with `hit block of {_r}`, `hit entity of {_r}`, `hit block face of {_r}` (`up`, `down`, `north`, `south`, `east`, `west`), `hit position of {_r}` and `hit distance of {_r}`.

Limits: blocks are found by stepping through the world and every non-air block counts as a full cube, so slabs and fences are hit as whole blocks. Entities are found only on hosts that can search along a ray; on Pumpkin 0.1 `hit entity` is not set. `with ray size` is a script error.

## Worlds

```
create world "arena"
create a new world named "hell" with environment nether
load world "old"
unload world "hell"
if world "arena" is loaded:
```

Names use letters, digits, `_`, `-` and `.`. A server that cannot do it (Pumpkin 0.1) makes the line a script error with the reason; the rest of the script keeps loading.

## Chunks

```
set {_c} to chunk at player
set {_c} to chunk at 3, 4 in world "world"
send "%chunk x of {_c}%, %chunk z of {_c}%"
set {_all::*} to chunks in radius 2 around player
if {_c} is loaded:
```

`chunks in radius` is limited to radius 16.

## Minecraft tags

```
if {_b} is tagged with minecraft tag "logs":
set {_l::*} to minecraft block tag "logs"
set {_t::*} to minecraft tags of oak log
```

The vanilla block and item tags are built in. An unknown tag name logs a warning and the check is false.
