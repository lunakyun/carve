# Colors

Carve text can carry colors. Every message the plugin sends (`send`, `broadcast`, titles, action bars, command replies) is parsed for color codes. You do not need `colored` for that.

## Codes and tags

| Form | Example | Meaning |
| --- | --- | --- |
| Legacy code | `&a`, `&c`, `&l`, `&r` | Color, bold (`&l`), italic (`&o`), underline (`&n`), strikethrough (`&m`), obfuscated (`&k`), reset (`&r`). `§` works too. |
| Hex tag | `<#ff8000>` | Any RGB color. |
| Hex code | `&#ff8000`, `&x&f&f&8&0&0&0` | Same, in the legacy style. |
| Named tag | `<red>`, `<dark_blue>`, `<gold>` | The 16 chat colors. `<grey>` and `<gray>` both work. |
| Format tag | `<bold>`, `<italic>`, `<underlined>`, `<strikethrough>`, `<obfuscated>` | Short forms: `<b>`, `<i>`, `<u>`, `<st>`, `<obf>`. |
| Reset | `<reset>` | Clears color and formatting. |
| Closing tag | `</bold>`, `</red>` | Turns one thing off. |

A legacy color code (`&a`) clears earlier formatting, like vanilla. Tags do not.

Text that is not a valid code stays as it is. `Tom & Jerry`, `1 < 2` and `<unknown>` print unchanged.

```
on join:
    send "<#ff8000>Welcome <bold>%player%</bold>&r to the server"
    send "&aGreen &lbold green"
```

## colored / uncolored

```
set {_t} to colored "<red>Hello"
set {_plain} to uncolored {_t}
```

- `colored <text>` (also `formatted`) turns codes and tags into `§` codes. Doing it twice changes nothing.
- `uncolored <text>` (also `unformatted`) removes every code and tag and leaves the plain text.

They apply to the next value only, so write `colored {x}` or `colored "text"`.

## Not done yet

- `color of <item>`, `rgb(...)` color values, color items.
- Gradients and `<rainbow>` tags.
