# Time, weather, game rules and difficulty

All of these work on the default world unless you name a world. To name one,
add `in world "name"` (or `of`). A player also works: it means the world they
are in.

## Time

Time of day is a number of ticks, from 0 to 23999. A day has 24000 ticks.
Tick 0 is 6:00 in the morning, like in Skript and vanilla.

```
send "%time%"
send "%time in world "arena"%"
send "%time of day%"
```

### Clock times

A clock time is a number of ticks, so you can compare and set with it.

```
set time to 6:00        # tick 0
set time to 6:30 pm     # tick 12500
set time to 12 am       # midnight, tick 18000
if time is 6 am:
```

Hours are 0 to 24 (or 1 to 12 with `am` or `pm`). Minutes are 0 to 59.

### Set, add and remove

```
set time to day
set the time to night
set time in world "arena" to noon
set time to 1000
add 500 to time
add 1 minute to the time
remove 5 seconds from time in world "arena"
```

Named times: `day` (1000), `noon` (6000), `sunset` (12000), `night` (13000),
`midnight` (18000), `sunrise` (23000). A number past one day wraps around.
Setting the time keeps the day count when the server can tell it. Adding a
timespan uses 1 tick per 50 ms.

### Day and night

```
if it is day:
if it is night:
if it is not night in world "arena":
```

Night is tick 13000 up to 22999. Everything else is day.

## Weather

The weather expression is text: `clear`, `rain` or `thunder`.

```
send "%weather%"
send "%weather in world "arena"%"
if weather is "rain":
```

### Set

```
set weather to rain
set weather to thunder
set the weather to clear
set weather in world "arena" to rain
set weather to rain for 5 minutes
set weather to clear for 100
```

Also accepted: `raining`, `rainy`, `sunny`, `storm`, `thundering`. With `for`,
a number means ticks. A timespan is turned into ticks. The longest length is
1,000,000 ticks. A length is sent to the `weather` command, which only
changes the default world.

### Conditions

```
if it is raining:
if it is thundering:
if it is sunny:
if it is not raining:
if it isn't raining in world "arena":
if weather is clear:
if weather is not thunder:
```

`it is raining` is true in rain and in thunder. `it is sunny` means clear.
`weather is rain` is true for plain rain only.

## Game rules

```
send "%gamerule keepInventory%"
set gamerule keepInventory to true
set gamerule randomTickSpeed to 10
set the gamerule pvp in world "arena" to false
if gamerule keepInventory is true:
```

A rule reads as `true`, `false` or a number. Write the name as you like:
`keepInventory`, `keep_inventory` and `"keep-inventory"` are the same rule.
The vanilla names are checked when the script loads, so a typo is an error
with the line number. Old names such as `doDaylightCycle` or `doMobSpawning`
are mapped to their new names (`advance_time`, `spawn_mobs`). The value must
be `true`, `false` or a whole number, or it is a runtime error.

## Difficulty

```
send "%difficulty%"
set difficulty to hard
set the difficulty in world "arena" to peaceful
if difficulty is hard:
if difficulty is not easy:
```

Values: `peaceful`, `easy`, `normal`, `hard`, or the numbers 0 to 3.

## How it runs on the server

- Time, weather and game rules use the Pumpkin world API, per named world.
- Difficulty has no API yet. Setting it runs the `difficulty` command, which
  changes the default world. Reading difficulty gives `<none>`, so a
  difficulty condition is false.
- A weather change with `for` runs the `weather` command (default world only).
- If a host cannot read a value, the expression is `<none>` and a plain
  condition is false (a negated one is true).

## Not done yet

- `on weather change` and `at 6:00` events.
- `formatted time` and a separate time type.
- Per-player weather and time (`player weather`, `set player time`).
