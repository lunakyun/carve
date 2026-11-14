# Contributing to Carve

Thanks for helping! This page explains how to build Carve, run the tests and
add new syntax.

## Setup

You need Rust (stable, edition 2024) and the wasm target:

```
rustup target add wasm32-wasip2
rustup component add rustfmt clippy
```

## Build and test

```
cargo test -p carve-core                                     # all language tests, no server needed
cargo build -p carve-wasm --release --target wasm32-wasip2   # the plugin
cargo run --release -p carve-core --example bench            # the benchmark
cargo fmt --all                                              # format
cargo clippy -p carve-core --all-targets                     # lints
```

The plugin ends up in `target/wasm32-wasip2/release/carve.wasm`.

## How the code is laid out

- `crates/carve-core`: the language. No Pumpkin code here. It talks to the
  game only through the `Host` trait in `host.rs`.
- `crates/carve-wasm`: the Pumpkin plugin. It implements `Host`.
- `crates/carve-core/tests/core.rs`: tests, using a mock host.

[docs/architecture.md](docs/architecture.md) explains how a script goes from
text to instructions.

## Adding an effect

Say you want `feed %player%`.

1. **Pattern.** In `crates/carve-core/src/compile.rs`, add an `EffKind`
   variant and a line to `EFFECT_PATTERNS`:

   ```rust
   ("feed %player%", EffKind::Feed),
   ```

   Patterns are tried in order. Put more specific patterns first.

2. **Effect variant.** Add `Feed(Expr)` to `enum Effect`, and build it in
   `build_effect`:

   ```rust
   EffKind::Feed => Effect::Feed(take_expr(c, 0)?),
   ```

   Captures are numbered by slot, from left to right in the pattern.

3. **Host call.** If the game has to do something new, add a method to the
   `Host` trait (`fn feed(&mut self, player: &PlayerRef);`). Implement it in
   `WasmHost` and add it to the `delegate!` list for `TickHost` in
   `crates/carve-wasm/src/lib.rs`. Also implement it in the test mock and the
   bench host.

4. **Engine arm.** In `crates/carve-core/src/engine.rs`, handle it in
   `Engine::effect`:

   ```rust
   Effect::Feed(p) => {
       if let Some(p) = self.player(p, ex, host)? {
           host.feed(&p);
       }
   }
   ```

5. **Docs line.** Add a section to `docs/syntax/effects.md` with the pattern
   written exactly as in `EFFECT_PATTERNS`, an example and notes. The
   `docs_list_every_pattern` test fails until you do, so you can't forget.

6. **Tests.** Add the line to `every_effect_pattern_parses` and test what it
   does with the mock host.

Conditions work the same way: `CONDITION_PATTERNS`, `CondKind`, `build_cond`,
`Engine::cond`, and `docs/syntax/conditions.md`.

## Rules for code

- `carve-core` must not depend on Pumpkin.
- No panics in non-test code: no `unwrap`, `expect` or indexing that can fail.
  Errors in scripts become `ScriptError`s (load time) or logged runtime errors.
- Keep the existing tests passing. Add tests for new behavior.
- Run `cargo fmt` before you commit.
- If a change could affect speed, run the bench before and after.

## Docs

Docs are for students and server owners. Use short sentences and plain
English. Only document what the code does today; ideas go in the roadmap in
`README.md`. Add a line to `CHANGELOG.md` under `[Unreleased]`.

## Pull requests

- One topic per pull request.
- Say what changed and how you tested it.
- CI runs the tests on Windows and Linux and builds the plugin.
