# Timespans

A timespan is a length of time. A date is a point in time.
Both are stored as whole milliseconds. One tick is 50 ms.

## Writing a timespan

```
5 minutes
1 hour and 30 seconds
1 day, 2 hours and 3 minutes
2 ticks
1.5 seconds
```

Units: `millisecond(s)`, `tick(s)`, `second(s)`, `minute(s)`, `hour(s)`, `day(s)`,
`week(s)`, `month(s)` (30 days), `year(s)` (365 days).

A number with no unit is still a plain number. `5 minutes` is a timespan.

## Printing

A timespan prints as text: `1 minute and 30 seconds`, `0.1 seconds`.
It is rounded to the millisecond. Weeks, months and years print as days.
A date prints as `2026-11-14 11:10:05` (UTC).

## Waiting

```
wait 5 minutes
wait 1 hour and 30 seconds
set {delay} to 10 seconds
wait {delay}
```

A wait is rounded to the nearest tick. Waiting on a plain number variable
(`wait {n}`) is a runtime error, because the unit is unknown. Write
`wait {n} seconds` or `wait {n} ticks` instead.

## Math

| Expression | Result |
|---|---|
| `timespan + timespan`, `timespan - timespan` | timespan |
| `timespan * number`, `number * timespan`, `timespan / number` | timespan |
| `timespan / timespan` | number |
| `date + timespan`, `date - timespan` | date |
| `date - date` | timespan |

Anything else (`now + now`, `5 minutes * 5 minutes`) is an error.
When a variable holds the value, the check happens when the script runs.

## Dates

| Expression | Meaning |
|---|---|
| `now` | The current date and time. |
| `5 minutes ago`, `{span} ago` | `now` minus the timespan. |
| `2 hours later`, `{span} later` | `now` plus the timespan. |
| `time since {date}` | How long ago the date was. |
| `time until {date}` | How long until the date. |

```
set {last::%uuid of player%} to now
if time since {last::%uuid of player%} < 30 seconds:
    send "slow down"
```

## Parts of a timespan

| Expression | Meaning |
|---|---|
| `minutes of {t}` | The minutes part (0 to 59). |
| `total minutes of {t}` | The whole span measured in minutes. |

Works with `milliseconds`, `ticks`, `seconds`, `minutes`, `hours`, `days`,
`weeks`, `months` and `years`.

```
set {t} to 1 hour and 30 minutes
send "%minutes of {t}%"        # 30
send "%total minutes of {t}%"  # 90
send "%{t}'s minutes%"         # 30, the same as minutes of {t}
```

## Comparing

`<`, `>`, `<=`, `>=`, `is` and `is not` work on timespans and dates.
`1 minute is 60 seconds` is true.

## Saving

Timespans and dates saved in variables survive a restart.

## Not supported yet

- `{t}'s months` and `{t}'s years` (use `months of {t}`).
- `add 5 minutes to {t}` (use `set {t} to {t} + 5 minutes`).
- Timespans from text, like `"5 minutes" parsed as timespan`.
- `a minute`, `an hour` without a number.
