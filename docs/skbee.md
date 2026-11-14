# SkBee in Carve

Carve builds the most useful parts of the SkBee addon in. The syntax is close
to SkBee, so scripts are easy to port. Each area has its own section.

Legend: ✅ done, 🟡 partial, ⛔ blocked.

## Bounds

Docs: [syntax/skbee-bound.md](syntax/skbee-bound.md). In SkBee, a bound is a
region with an id. In Carve it is held as its id text.

| SkBee feature | Status | Note |
|---|---|---|
| Create bound with id within two locations | ✅ | `create bound with id "x" within A and B` |
| Temporary bound | ✅ | not saved |
| Delete bound | ✅ | |
| Bound with id | ✅ | `bound "x"` or `bound with id "x"` |
| Bound at location | ✅ | |
| Bounds at location (all) | ✅ | list |
| All bounds | ✅ | list |
| Location / player is within bound | ✅ | |
| Bound exists | ✅ | |
| On bound enter / exit | ✅ | polled every 2 ticks, grid index, per-player block cache |
| Event-bound | ✅ | |
| Bound owners and members | ✅ | add, remove, set, list, role conditions |
| Bound flags | 🟡 | yes/no flags only. SkBee also has typed bound values |
| Bound lesser / greater corner | ✅ | readable and settable |
| Bound world | ✅ | read only |
| Bound volume / size | 🟡 | `volume of bound` only |
| Persistence in the data folder | ✅ | `bounds.json`, saved 5 seconds after a change and on unload |
| Bound bounding box | 🟡 | corners only, no box type |
| Blocks in bound | ⛔ | needs a block iteration API that the Pumpkin plugin API does not have yet |
| Entities in bound | ⛔ | no entity list API in the plugin API |
| Bound enter/exit for non-players | ⛔ | no entity move events in the plugin API |
| Bound values (custom data) | ⛔ | no typed value store yet; Carve variables can hold this |
