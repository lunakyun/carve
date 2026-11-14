# SkBee parity

Carve builds the most useful parts of SkBee in. Each area has its own heading. Legend: ✅ done, 🟡 partial, ⛔ blocked.

## Particles and display entities

Syntax: [syntax/skbee-particles-display.md](syntax/skbee-particles-display.md).

### Particles

- ✅ Draw particles with a count, at a location or a player (`play 5 flame particles at player`).
- ✅ Offset: one value (`with spread N`) or per axis (`with offset x, y, z`).
- ✅ Extra / speed (`with extra N`, `with speed N`).
- ✅ Force (`with force`). Uses `/particle ... force`.
- 🟡 Dust color and size (`with color`, `with size`). Parsed and checked, but ⛔ cannot be sent: the Pumpkin API and `/particle` have no particle data field. The particle is skipped with a warning.
- 🟡 Dust color transition (`with to color`). Same limit.
- 🟡 Block particles (`block`, `block_marker`, `falling_dust` with `with block <id>`). Same limit.
- 🟡 Item particles (`with item <id>`). Same limit.
- 🟡 Other data particles (`entity_effect`, `tinted_leaves`, `sculk_charge`, `shriek`, `vibration`, `trail`). Same limit.
- ⛔ Particles shown to one player only. Not wired yet (the API has `spawn-particles` on a player, so this is possible later).
- ⛔ Particle shapes (`draw a circle`, lines, spheres) and `SkBee particle` objects. Not done.

### Display entities

- ✅ Text display: spawn with text, colors, billboard, scale, translation, view range, background, opacity, line width, alignment, shadow, see through, default background, brightness, glow, shadow radius and strength.
- ✅ Block display: spawn with a block id, scale, billboard and the shared options (Pumpkin plugin only).
- ✅ Item display: spawn with an item id and display mode (Pumpkin plugin only).
- 🟡 Spawn only. There are no entity values in Carve yet, so `set text of {_d} to ...` and `last spawned display` are not available.
- ⛔ Transformation rotation (left and right quaternions) and interpolation. Not exposed yet.
- ⛔ Block display with block states (`oak_stairs[facing=north]`). Only the default state is used.
- ⛔ Item display from a held item with enchantments or names. Only the plain item id is used.
- ⛔ Block and item displays on hosts that only run console commands: Pumpkin's `/summon` does not read the block or item NBT.

### Fireworks

- ⛔ Not done. The API can spawn a `firework_rocket` entity, but has no way to set its explosion (colors, shape, flight time), and Pumpkin's `/summon` does not read firework NBT. A bare rocket does nothing useful. Revisit when the API adds a firework resource.
