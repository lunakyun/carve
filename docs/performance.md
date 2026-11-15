# Performance

Carve runs inside the server, so scripts should cost as little as possible.
Here is how we measure it, and the numbers.

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
| d | Compiling every file in `examples/` once (six files when this was measured). |
| e | One event with no triggers for it (the bench runs 10,000 of them). |
| f | A loop of 10,000 rounds that adds to a list. |
| g | A loop of 1,000 calls of a one-line function. |
| h | 1,000 passes through a 10-branch `if` chain that compares with negative numbers. |

## Results

One run of the bench on a Windows 11 desktop, native release build (opt-level 3,
LTO). Times change from machine to machine and from run to run, so compare
them only with a run on your own machine. The host-call counts are exact.

| Case | ns/run | Host calls/run |
|---|---:|---:|
| a) join, 20 effects | 6,642 | 20 |
| b) 50x `set {_x}` + 1 | 2,014 | 3 |
| c) if/else-if x10 | 617 | 4 |
| d) compile examples | 771,620 | 0 |
| e) fire, no triggers | 4 | 0 |
| f) 10k list adds | 5,303,880 | 3 |
| g) 1k function calls | 171,126 | 3 |
| h) 1k if-chain | 66,618 | 3 |

The bench runs the native build only. It does not time the plugin as wasm.

### Plugin build

The release profile uses `opt-level = 3` and LTO. `carve.wasm` on the release
page is that build (about 5.5 MB). `carve-small.wasm` is the same code built
for size (about 4.5 MB) and a little slower, for hosts that limit uploads to
5 MB.

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

## Loops

Each round of a loop costs a few instructions. All of them count towards the
100,000-instruction budget of a run slice (see [limits](limits.md)).
