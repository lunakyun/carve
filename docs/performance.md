# Performance

Carve runs inside the server, so scripts should cost as little as possible.
This page shows how we measure that and what the numbers are.

## The benchmark

```
cargo run --release -p carve-core --example bench
```

The bench needs no extra crates. It uses a fake host that does no real work.
Each host call bumps an atomic counter. That stands in for the cost of a real
host call, which in the plugin crosses the wasm component boundary. The bench
prints two numbers per case:

- **ns/run**: time for one run, best of 5 rounds.
- **host calls/run**: how many times the script called the host.

Host calls matter most. In the real plugin each one is a trip out of the wasm
sandbox and into the server, which costs far more than anything the bench host
does.

The cases:

| Case | What it runs |
|---|---|
| a | An `on join` trigger with 20 effects that use `player`, text templates and `{joins::%player%}`. Includes building the event (3 host calls). |
| b | `set {_x} to 0`, then 50 lines of `set {_x} to {_x} + 1`. |
| c | An `if` / `else if` chain of 10 branches where the last branch matches. |
| d | Compiling every file in `examples/` once. The table below used the two original examples (`welcome.carve`, `heal.carve`). |
| e | One event with no triggers for it (the bench runs 10,000 of them). |

## Results

Measured on a Windows 11 desktop, native build. "Before" is commit `7d495a0`,
which used `opt-level = "s"`. "After" is shown at both opt levels so you can
see what the code changes did on their own.

| Case | Before ns | After ns (opt "s") | After ns (opt 3) | Before calls | After calls |
|---|---:|---:|---:|---:|---:|
| a) join, 20 effects | 10,712 | 4,937 | 4,497 | 74 | 21 |
| b) 50x `set {_x}` + 1 | 10,343 | 1,467 | 1,383 | 3 | 3 |
| c) if/else-if x10 | 1,705 | 715 | 601 | 5 | 5 |
| d) compile examples | 68,000 | 44,000 | 39,300 | 0 | 0 |
| e) fire, no triggers | 10 | 10 | 9 | 0 | 0 |

Case (a) also went from 74 host calls to 21. In the plugin that is the biggest
win, because every call it saves is a trip across the wasm boundary.

### Wasm: opt-level 3 or "s"?

We ran the same bench as wasm (target `wasm32-wasip1`, under Node's WASI):

| Case | opt "s" ns | opt 3 ns |
|---|---:|---:|
| a | 3,990 | 3,510 |
| b | 2,110 | 1,690 |
| c | 720 | 530 |

| Plugin build | Size of `carve.wasm` |
|---|---:|
| opt-level "s" | 801,226 bytes |
| opt-level 3 | 869,120 bytes |

Opt-level 3 is 12 to 26% faster for 8% more size, so the release profile now
uses `opt-level = 3`.

## What changed

- **Player lookups are cached per run slice.** Each run keeps a small
  UUID-to-name table, so a player is looked up at most once until the next
  `wait`. The table is cleared at every `wait`, because a player can leave
  while a run waits.
- **The plugin caches player handles.** `WasmHost` keeps the `Player` handle
  for each UUID for one call into the plugin, so ten effects on the same
  player do one lookup, not ten.
- **The event is only built when needed.** The plugin reads the player from an
  event (3 host calls) only when some script listens to that event type.
- **Variable names are worked out at compile time.** `{coins}` gets its
  lowercased key once, when the script loads. Only names with `%...%` in them
  are built at run time.
- **Constant folding.** `2 + 3 * 4` becomes `14` when the script loads, and
  `"%7 / 2%"` becomes `"3.5"`. Division by zero is not folded, so it is still
  reported as a runtime error with its line number.
- **Local variables use a flat list** instead of a sorted map. A run has few
  locals, so a short list is faster. `delete {_list::*}` still works.
- **Fewer strings.** Text templates are written into one buffer, and numbers
  are read from variables without copying.
- **Faster loading.** While matching one line against all patterns, each token
  span is parsed once and the result is shared. The matcher also stops trying
  longer spans once a parse failed before reaching the end, and it skips
  patterns whose words are not in the line.

## What we did not change

- **Global variables stay in a sorted map.** Updates no longer copy the key,
  which was the real cost. The bench uses only a few globals, so it gives no
  reason to switch to a hash map, and the sorted map keeps the saved
  variables in order for free.
- **No bytecode for expressions.** In case (b), about 10 ns of the roughly
  25 ns per line is reading `{_x}`. The rest is running the instruction and
  storing the result. Expression trees do not dominate, so a bytecode VM would
  add code for little gain.

## v2

v2 added loops, functions, more condition kinds and more types. The bench
below compares v1 (`3757017`) with v2 on the same machine, as the median of 5
alternating runs. Case d now always compiles the six v1 example files, so it
stays comparable as examples are added.

| Case | v1 ns | v2 ns | Change |
|---|--:|--:|--:|
| a) join, 20 effects | 4,708 | 4,878 | +3.6% |
| b) 50x `set {_x}` + 1 | 1,530 | 1,571 | +2.7% |
| c) if/else-if x10 | 624 | 549 | -12% |
| d) compile 6 examples | 153,858 | ~171,000 | ~+11% |
| e) fire, no triggers | 9 | 9 | 0% |

- **Getting there.** The first v2 build was 20–40% slower in b, c and d. Most
  of it came back by keeping the hot functions (`eval`, `cond`, `effect`) small
  and moving rare cases out of line. A direct path for `{var} <op> number`
  comparisons made c faster than v1.
- **Compiling.** It got about 11% slower (about 3 µs per file) because there
  are more patterns and expression forms to try. That only happens when
  scripts load or reload.
- **Loops.** Each round of a loop costs a few instructions. All of them count
  towards the 100,000-instruction runaway guard.

