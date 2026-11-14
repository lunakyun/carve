# Particles and display entities (SkBee style)

Carve takes SkBee's ideas for particles and display entities and puts them in built-in syntax. No addon is needed.

## Particle options

These add to `play N <particle> particles at <place>`. Put each option after its own `with`.

```
play 20 flame particles at player with offset 1, 0.5, 1 with extra 0.05
play 5 cloud particles at player with spread 2 with speed 0.1
play 1 end_rod particles at location(0, 80, 0) with force
play 10 dust particles at player with color rgb 255, 128, 0 with size 2
play 10 dust particles at player with color "#ff8000"
play 10 dust_color_transition particles at player with color red with to color blue
play 20 block particles at player with block oak planks
play 5 item particles at player with item diamond sword
```

| Option | Meaning |
| --- | --- |
| `spread N` | The same random spread on x, y and z. |
| `offset x, y, z` | A different spread on each axis. Wins over `spread`. Each value is clamped to 0..64. |
| `speed N` or `extra N` | The particle speed (SkBee calls it extra). Clamped to 0..64. |
| `force` | Show the particle to players far away too (up to 512 blocks). |
| `color <color>` | A name (`red`), a hex text (`"#ff8000"` or `"ff8000"`) or `rgb r, g, b` (0..255). |
| `to color <color>` | The end color of `dust_color_transition`. |
| `size N` | Dust size, clamped to 0.01..4. Default 1. |
| `block <id>` | The block for `block`, `block_marker` and `falling_dust`. |
| `item <id>` | The item for `item` particles. |

Defaults: dust is red, size 1. Block and item particles default to `stone`.

### Limit: particle data

Pumpkin's plugin API and its `/particle` command cannot send particle data. That means these particles are not shown yet:
`dust`, `dust_color_transition`, `block`, `block_marker`, `falling_dust`, `item`, `entity_effect`, `tinted_leaves`, `sculk_charge`, `shriek`, `vibration`, `trail`.

Carve still compiles them and checks the options. At run time it logs one warning and skips the particle, because an empty payload can disconnect clients. A host that can send data returns `true` from `supports_particle_data` and reads `ParticleFx::data`.

## Display entities

```
spawn text display at player with text "&aHello"
spawn text display at location(0, 70, 0) with text "<#ff8000>Sale" with billboard center with scale 2
spawn block display of stone at location(0, 70, 0) with scale 0.5
spawn item display of diamond sword at location(0, 70, 0) with mode head
```

The verb can be `spawn`, `summon`, `create`, `show`, `make`, `draw` or `play`. Articles are allowed: `spawn a new text display at ...`.

After `at` you can give a location or a player. Text accepts colors (`&a`, `<#ff8000>`, `<red>`) and `%variables%`.

| Option | Applies to | Meaning |
| --- | --- | --- |
| `text <text>` | text | The text. You can also write `showing "..."` or `of "..."` before `at`. |
| `billboard <mode>` | all | `fixed`, `vertical`, `horizontal` or `center`. |
| `scale N` or `scale x, y, z` | all | Size. Clamped to 0..100. |
| `translation x, y, z` | all | Shift from the spawn point. Clamped to -1000..1000. |
| `view range N` | all | Distance multiplier, 0..100. |
| `brightness N` | all | Fixed light level 0..15 (sky and block). |
| `glow <color>` | all | Glow color override. |
| `shadow radius N`, `shadow strength N` | all | Shadow settings. |
| `background <color>` or `background transparent` | text | Background color (opaque). |
| `opacity N` | text | Text opacity 0..255. |
| `line width N` | text | Wrap width in pixels. |
| `alignment <left/center/right>` | text | Text alignment. |
| `shadow` | text | Drop shadow on the text. |
| `see through` | text | Visible through blocks. |
| `default background` | text | The vanilla background color. |
| `mode <mode>` | item | `none`, `head`, `gui`, `ground`, `fixed`, `thirdperson_lefthand`, `thirdperson_righthand`, `firstperson_lefthand`, `firstperson_righthand`. Spaces work: `third person left hand`. |

Text longer than 4000 characters is cut.

### Limits

- Spawn only. Carve has no entity values yet, so you cannot change a display after it spawns. Set everything in the spawn line.
- Text displays work on every host (the default host runs `/summon`).
- Block and item displays need the plugin API. The `/summon` command cannot fill them. In the Pumpkin plugin they work. On a host that only runs commands, Carve logs a warning.
- Interpolation and rotation are not available yet.
