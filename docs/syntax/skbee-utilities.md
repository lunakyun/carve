# SkBee utilities

Number and text helpers, random UUIDs and extra player client properties.
All of these are expressions. They work anywhere a value fits, also inside
`%...%` in a text. A `<none>` input gives `<none>`.

## Numbers

| Syntax | Example result |
|---|---|
| `n with commas` | `1234567` becomes `1,234,567`; `1234.5` becomes `1,234.5` |
| `n with commas and 2 decimal places` | `1234567.891` becomes `1,234,567.89` |
| `n rounded to 2 decimal places` | `3.14159` becomes the number `3.14` |
| `abbreviated n` | `1500` becomes `1.5K`; `2000000` becomes `2M` (K, M, B, T) |
| `roman numeral of n` (also `roman of n`) | `1994` becomes `MCMXCIV`; works for 1 to 3999 |
| `number from roman numeral "XIV"` | `14`. Only the proper spelling counts, so `"IIII"` is `<none>`. |
| `hex of n` | `255` becomes `ff` |
| `number from hex "ff"` | `255`. Also accepts `#ff` and `0xff`. |

Use brackets for a negative number: `(-1234567) with commas`.

## Text and IDs

| Syntax | Result |
|---|---|
| `json of x` | A JSON value. Text is quoted and escaped (`"a ""q"""` becomes `"a \"q\""`). Numbers and booleans stay bare. `<none>` gives `null`. |
| `random uuid` | A new random UUID (version 4) as text. |

## Player client properties

These need an online player. Both word orders work: `view distance of player`
and `player's view distance`.

| Syntax | Result |
|---|---|
| `client brand of p` (also `brand of p`) | `vanilla`, `fabric`, ... (Java players only) |
| `locale of p` (also `language`, `client locale`) | `en_us` |
| `view distance of p` (also `client view distance`) | A number, in chunks (Java players only) |
| `pose of p` | `standing`, `crouching`, `sleeping`, `swimming`, `fall flying`, ... |
| `main hand of p` | `left` or `right` (Java players only) |

## Example

```
on join:
    send "&aWelcome! You are on %client brand of player% with %view distance of player% chunks."
    set {_c} to 1234567
    send "Balance: %{_c} with commas% coins (%abbreviated {_c}%)"
    set {id::%player%} to random uuid
```
