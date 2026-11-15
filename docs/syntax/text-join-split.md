# Text: join, split, length, replace

## Join a list

`join {list::*} with ", "` gives one text. Items print the way they do in
`send`. Items that are not set are skipped.

```
add "a" to {l::*}
add "b" to {l::*}
send join {l::*} with ", "      # a, b
```

## Split text into a list

`split "a,b,c" at ","` makes a list. It works in two places:

- `set {x::*} to split "a,b,c" at ","` replaces the list.
- `loop split "a,b,c" at ",":` loops over the pieces.

An empty separator (`""`) splits into single characters. `by` works like `at`.
Using `split` for a plain variable like `{x}` is an error.

## Length

- `length of <text>`
- `amount of characters in <text>` (also `number of`, `size of`)

Both count characters, not bytes.

## Replace

As an effect, on a text variable:

- `replace all "a" with "b" in {var}`
- `replace all "a" in {var} with "b"`
- `replace first "a" with "b" in {var}`

`all` is optional (`replace "a" with "b" in {var}` replaces every match).
If the variable is not text, nothing changes.

As an expression, giving a new text:

- `{t} with all "a" replaced with "b"`
- `{t} with first "a" replaced by "b"`

### Regex replace

```
replace all regex "[0-9]+" in {_x} with "#"
replace first regex "a+" with "b" in {_x}
regex replace " +" in {_x} with " "
send {_t} with all regex "[0-9]" replaced with "_"
```

- The pattern uses the same regex as `matches`: Java-like, no lookaround.
- The replacement is plain text. Carve's regex has no groups, so `$1` stays
  `$1`.
- A broken pattern is a runtime error with the reason.

## Not supported yet

- Matching is exact and case sensitive. Skript ignores case by default.
- `replace` on a whole list (`{list::*}`) and on non-variable targets.
- Regex groups in the replacement (`$1`).
