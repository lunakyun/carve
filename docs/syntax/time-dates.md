# Dates

A date is one moment in time. It is stored as milliseconds since 1970-01-01 UTC.
All dates use UTC.

## Getting a date

```
set {start} to now
set {d} to date from unix timestamp 1700000000
set {d} to unix date of 1700000000
```

The unix timestamp is in seconds. It may have decimals.

## Unix timestamp

```
set {t} to unix timestamp of now
set {t} to unix timestamp of {start}
```

## Formatting

```
send "It is %now formatted as "yyyy-MM-dd HH:mm"%"
set {text} to formatted now as "EEEE d MMMM"
set {text} to {start} formatted
```

Without `as`, the format is `yyyy-MM-dd HH:mm:ss`. Printing a date does the same.

Format letters follow Java:

| Letters | Meaning |
|---|---|
| `yyyy`, `yy` | year |
| `MM`, `M` | month number |
| `MMM`, `MMMM` | month name (short, full) |
| `dd`, `d` | day of month |
| `HH`, `H` | hour, 0 to 23 |
| `hh`, `h` | hour, 1 to 12 |
| `mm`, `ss` | minute, second |
| `SSS` | milliseconds |
| `a` | AM or PM |
| `EEE`, `EEEE` | weekday name (short, full) |
| `'text'` | literal text |

## Comparing

Use `<`, `>`, `<=`, `>=`, `=` and `is not`. A later date is greater.

```
if now > {ban-end}:
    send "Your ban is over."
```

## Saving

Dates in variables are saved as `{"date": <millis>}`.
They load back as dates.

## Not supported yet

Time spans (`5 minutes`), `5 minutes ago`, and date arithmetic. They need time spans first.
Time zones other than UTC are not supported.
