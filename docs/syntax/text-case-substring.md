# Text: case, substrings and characters

All of these are expressions. They work anywhere a value fits, also inside
`%...%` in a text. Positions count characters, starting at 1.
A `<none>` input gives `<none>`.

## Case

| Syntax | Example result |
|---|---|
| `uppercase of x`, `upper case of x`, `x in upper case` | `"abc"` becomes `ABC` |
| `lowercase of x`, `lower case of x`, `x in lower case` | `"ABC"` becomes `abc` |
| `x in proper case` | `"hello wORLD"` becomes `Hello World` |
| `capitalized x` | `"hello world"` becomes `Hello world` (first letter only) |

## Pieces of a text

| Syntax | Result |
|---|---|
| `substring of x from a to b` (also `subtext`) | Characters a to b, both included. Too big or small numbers are cut to fit. |
| `substring of x from a` | From a to the end. |
| `first N characters of x` | The first N characters. |
| `last N characters of x` | The last N characters. |
| `character at index N of x` | One character, or `<none>` if N is out of range. |

## Finding

`index of "l" in "hello"` is `3`. It is `0` when the text is not found.
`first index of ...` is the same. `last index of "l" in "hello"` is `4`.
The search is case sensitive.

## Other

- `nl`, `newline` or `new line`: a line break. Use it inside a text:
  `"line one%nl%line two"`.
- `trimmed x`: removes spaces at the start and end.
- `reversed "abc"` or `reversed text of {x}`: `cba`. A list variable after
  `reversed` is left for lists.

## Example

```
on join:
    set {_name} to "steve"
    send "Welcome, %capitalized {_name}%!"
    send "%{_name} in upper case%, %first 3 characters of {_name}%"
    if index of "e" in {_name} is 3:
        send "third letter is e"
```
