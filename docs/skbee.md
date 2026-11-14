# SkBee in Carve

Carve builds the most useful parts of [SkBee](https://github.com/ShaneBeee/SkBee/wiki)
in, with syntax close to SkBee so scripts port easily.

Legend: ✅ done · 🟡 partial · ⛔ blocked (reason given)

## NBT and item components

Syntax and examples: [syntax/skbee-nbt.md](syntax/skbee-nbt.md).

| SkBee feature | Status | Note |
|---|---|---|
| NBT compound type and SNBT text | ✅ | `nbt from "{a:1}"`, all tag kinds, printing back to SNBT |
| `nbt compound of <item>` | 🟡 | main-hand item only (`nbt of player's tool`); this is the item's `custom_data` |
| `nbt compound of <entity>` | ⛔ | the plugin API has namespaced keys only, no whole compound |
| `nbt compound of <block>` | ⛔ | same: block entities offer namespaced keys only |
| `nbt compound of <file>` / `<world>` | ⛔ | no file or world NBT access in the plugin API |
| Get tags (`int tag "a" of ...`) | ✅ | all types except `uuid`; paths with `;` and `list[0]` |
| Set tags (`set string tag ...`) | ✅ | creates compounds on the way |
| Delete tags | ✅ | `delete tag ...`, `remove tag ... from ...` |
| Has tag condition | ✅ | with or without a type |
| Merge compounds | ✅ | `add <nbt> to nbt of ...`, `merge ... into ...` |
| Tag names (keys) | ✅ | `tag names of <nbt>` |
| NBT lists and arrays | 🟡 | read as lists; set from text like `"[1,2,3]"` or from a list |
| `uuid tag` | ⛔ | not done yet (stored as an int array in vanilla) |
| NBT items (`nbt item from nbt`) | ⛔ | Carve items do not carry data |
| Custom data on items | ✅ | the `custom_data` component, via `nbt of player's tool` |
| `custom model data` | 🟡 | first float only, main-hand item |
| `unbreakable` | ✅ | main-hand item |
| `max stack size` | ✅ | main-hand item, 1 to 99 |
| `rarity` | ✅ | main-hand item |
| `enchantment glint override` | ✅ | main-hand item |
| `item model` | ✅ | main-hand item |
| Other components (food, tool, equippable, ...) | ⛔ | their byte formats are not built; the WIT exposes them as raw bytes |
| Persistent data containers | ⛔ | not part of this area |
| NBT events and `nbt` of projectiles | ⛔ | no plugin API |
