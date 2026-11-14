# SkBee parity

SkBee is the most popular Skript addon. Carve builds the useful parts in by
default, with syntax close to SkBee so scripts port easily.

Legend: ✅ done, 🟡 partial, ⛔ blocked (with the reason).

## Recipes, advancements and statistics

Syntax and examples: [skbee-recipes-advancements](syntax/skbee-recipes-advancements.md).

| SkBee feature | Status | Notes |
|---|---|---|
| Register shaped recipe | ✅ | `register shaped recipe "id" for <item> with shape ... where ...`. Id, shape and items are checked at load. |
| Register shapeless recipe | ✅ | 1 to 9 ingredients, `3 dirt` repeats an item. |
| Register furnace, blasting, smoking, campfire recipe | ✅ | Experience and cooking time are supported. |
| Recipe group and recipe category | 🟡 | Not settable. Pumpkin's defaults are used. |
| Ingredient tags (`#minecraft:planks`) | ✅ | Use text: `"#minecraft:planks"`. |
| Ingredients with NBT, custom names or exact item match | ⛔ | Pumpkin's recipe API only takes item ids and tags. |
| Smithing recipes (transform, trim) | ⛔ | Pumpkin's plugin API has no smithing recipe type. |
| Stonecutting, brewing, and other recipe types | ⛔ | Same: the plugin API only has shaped, shapeless and cooking. |
| Remove or replace a recipe | ⛔ | The plugin API can only register recipes, not remove them. |
| Discover and undiscover a recipe for a player | ✅ | Uses the `recipe give` and `recipe take` console command. |
| `on recipe discover` event | ✅ | `recipe id` is the recipe. Cancellable. |
| Recipe book click and recipe craft events | ⛔ | Not wired yet. The API has events, but no script syntax is added. |
| Grant and revoke an advancement | ✅ | Whole advancement or one criterion. |
| `player has advancement "x"` | ✅ | With negations. |
| `on advancement done` event | ✅ | `advancement id` is the advancement. Cancellable. |
| Create custom advancements | ⛔ | The plugin API can only read and award existing ones. Use a datapack. |
| Advancement progress, criteria lists, display info | ⛔ | Not exposed to scripts yet. |
| Get, set, add and remove a player statistic | 🟡 | Custom statistics (`custom:jump`, `custom:play_time`, ...) only. See below. |
| Mined, crafted, used, broken, picked up, dropped, killed statistics | ⛔ | Pumpkin identifies these by registry number, which Carve cannot look up from a block or item name yet. The script syntax accepts them and the host answers "not supported". |
| Villager and merchant trades (read or set recipes) | ⛔ | Pumpkin's plugin API has villager events (`acquire trade`, `replenish trade`) but no way to read or change a merchant's trade list. |

Custom recipes are registered with the live server when the line runs. Run
the line in `on load` so it happens once per start. A reload runs it again,
and the server may refuse a duplicate id; the script gets a normal runtime
error in that case.
