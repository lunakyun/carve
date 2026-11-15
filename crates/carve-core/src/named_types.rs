//! More value types: about sixty named-value types (click type, damage
//! cause, tree type, ...), colors, quaternions, time periods, chunks, queues
//! and world borders.
//!
//! Named-value types are one table ([`ENUMS`]). A value is a [`EnumVal`]
//! (type index + value index), so it is small, cheap to copy and compares
//! without allocating. Every one of them:
//! - can be written as a bare word when the word is not already something
//!   else (`left`, `full moon`, `shift left`),
//! - can be made from text with `"left" parsed as click type`,
//! - compares equal to text with the same name (`click type is "left"`),
//! - prints as its name with spaces, and is saved with the variables,
//! - works as a function parameter or return type (`function f(c: click type)`).
//!
//! The other types are made with `rgb(r, g, b)`, `quaternion(w, x, y, z)`,
//! `timeperiod(from, to)`, `chunk of <location>`, `queue(...)` and
//! `world border of <world>`.
//!
//! Every expression of this module is one boxed [`Node`] (`Expr::T1`), so the
//! hot `Expr` and `eval` paths stay small.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::data::named_types as data;
use crate::expr::{Expr, P, Ty};
use crate::lexer::Tok;
use crate::value::Value;

/// A value of one of the named-value types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumVal {
    /// Index into [`ENUMS`].
    pub ty: u8,
    /// Index into that type's `values`.
    pub idx: u16,
}

impl EnumVal {
    pub fn def(self) -> &'static EnumDef {
        &ENUMS[usize::from(self.ty)]
    }

    /// The id, like `shift_left`.
    pub fn id(self) -> &'static str {
        self.def()
            .values
            .get(usize::from(self.idx))
            .copied()
            .unwrap_or("?")
    }

    pub fn type_name(self) -> &'static str {
        self.def().name
    }
}

/// One named-value type.
pub struct EnumDef {
    pub name: &'static str,
    /// Other names the type goes by in `parsed as` and function headers.
    pub aliases: &'static [&'static str],
    /// Value ids, lowercase with `_`.
    pub values: &'static [&'static str],
    /// May a value be written as a bare word? False where the words already
    /// mean something else in Carve (entity types, difficulty, weather).
    pub literal: bool,
}

const fn def(
    name: &'static str,
    aliases: &'static [&'static str],
    values: &'static [&'static str],
    literal: bool,
) -> EnumDef {
    EnumDef {
        name,
        aliases,
        values,
        literal,
    }
}

/// Type ids above the named-value types.
/// Inventory values (see `gaps`).
pub const T_INVENTORY: u8 = 249;
pub const T_COLOR: u8 = 250;
pub const T_QUATERNION: u8 = 251;
pub const T_TIMEPERIOD: u8 = 252;
pub const T_CHUNK: u8 = 253;
pub const T_QUEUE: u8 = 254;
pub const T_BORDER: u8 = 255;

const CONVERGING: &[&str] = &[
    "enchant",
    "nautilus",
    "portal",
    "vault_connection",
    "reverse_portal",
];
const DIRECTIONAL: &[&str] = &[
    "bubble",
    "bubble_column_up",
    "campfire_cosy_smoke",
    "campfire_signal_smoke",
    "cloud",
    "crit",
    "damage_indicator",
    "dragon_breath",
    "end_rod",
    "enchanted_hit",
    "firework",
    "flame",
    "poof",
    "smoke",
    "large_smoke",
    "soul",
    "soul_fire_flame",
    "squid_ink",
    "glow_squid_ink",
    "totem_of_undying",
    "spit",
    "sneeze",
    "snowflake",
    "scrape",
    "wax_on",
    "wax_off",
    "electric_spark",
];
const SCALABLE: &[&str] = &[
    "sweep_attack",
    "explosion",
    "explosion_emitter",
    "gust",
    "gust_emitter_large",
];

pub static ENUMS: &[EnumDef] = &[
    def("attribute type", &["attribute"], data::ATTRIBUTES, true),
    def(
        "banner pattern type",
        &[],
        &[
            "base",
            "square_bottom_left",
            "square_bottom_right",
            "square_top_left",
            "square_top_right",
            "stripe_bottom",
            "stripe_top",
            "stripe_left",
            "stripe_right",
            "stripe_center",
            "stripe_middle",
            "stripe_downright",
            "stripe_downleft",
            "small_stripes",
            "cross",
            "straight_cross",
            "triangle_bottom",
            "triangle_top",
            "triangles_bottom",
            "triangles_top",
            "diagonal_left",
            "diagonal_up_right",
            "diagonal_up_left",
            "diagonal_right",
            "circle",
            "rhombus",
            "half_vertical",
            "half_horizontal",
            "half_vertical_right",
            "half_horizontal_bottom",
            "border",
            "curly_border",
            "gradient",
            "gradient_up",
            "bricks",
            "globe",
            "creeper",
            "skull",
            "flower",
            "mojang",
            "piglin",
            "flow",
            "guster",
        ],
        true,
    ),
    def(
        "boss bar style",
        &["bossbar style"],
        &[
            "solid",
            "segmented_6",
            "segmented_10",
            "segmented_12",
            "segmented_20",
        ],
        true,
    ),
    def(
        "cat type",
        &["cat variant"],
        &[
            "tabby",
            "black",
            "red",
            "siamese",
            "british_shorthair",
            "calico",
            "persian",
            "ragdoll",
            "white",
            "jellie",
            "all_black",
        ],
        true,
    ),
    def("chicken variant", &[], &["temperate", "warm", "cold"], true),
    def(
        "click type",
        &[],
        &[
            "left",
            "right",
            "shift_left",
            "shift_right",
            "middle",
            "drop",
            "control_drop",
            "double_click",
            "number_key",
            "swap_offhand",
            "window_border_left",
            "window_border_right",
            "creative",
            "unknown",
        ],
        true,
    ),
    def("cow variant", &[], &["temperate", "warm", "cold"], true),
    def(
        "damage cause",
        &[],
        &[
            "block_explosion",
            "campfire",
            "contact",
            "cramming",
            "custom",
            "dragon_breath",
            "drowning",
            "dryout",
            "entity_attack",
            "entity_explosion",
            "entity_sweep_attack",
            "fall",
            "falling_block",
            "fire",
            "fire_tick",
            "fly_into_wall",
            "freeze",
            "hot_floor",
            "kill",
            "lava",
            "lightning",
            "magic",
            "melting",
            "poison",
            "primed_tnt",
            "projectile",
            "sonic_boom",
            "starvation",
            "suffocation",
            "suicide",
            "thorns",
            "void",
            "wither",
            "world_border",
        ],
        true,
    ),
    def("damage type", &[], data::DAMAGE_TYPES, true),
    def("difficulty", &[], &["peaceful", "easy", "normal", "hard"], false),
    def(
        "direction",
        &[],
        &["north", "south", "east", "west", "up", "down"],
        false,
    ),
    def(
        "display billboard",
        &["billboard"],
        &["fixed", "vertical", "horizontal", "center"],
        true,
    ),
    def(
        "display text alignment",
        &["text alignment"],
        &["center", "left", "right"],
        true,
    ),
    def("enchantment", &[], data::ENCHANTMENTS, true),
    def("entity effect", &[], data::ENTITY_STATUSES, true),
    def("entity type", &[], crate::entity_names::ENTITY_TYPES, false),
    def(
        "equipment slot",
        &[],
        &[
            "hand", "off_hand", "feet", "legs", "chest", "head", "body", "saddle",
        ],
        true,
    ),
    def(
        "firework type",
        &[],
        &["ball", "large_ball", "star", "burst", "creeper"],
        true,
    ),
    def(
        "fishing state",
        &[],
        &[
            "fishing",
            "caught_fish",
            "caught_entity",
            "in_ground",
            "failed_attempt",
            "reel_in",
            "bite",
        ],
        true,
    ),
    def("frog variant", &[], &["temperate", "warm", "cold"], true),
    def(
        "game effect",
        &[],
        &[
            "bow_fire",
            "click1",
            "click2",
            "door_close",
            "door_toggle",
            "ender_signal",
            "extinguish",
            "firework_shoot",
            "iron_door_close",
            "iron_door_toggle",
            "record_play",
            "smoke",
            "step_sound",
            "potion_break",
            "instant_potion_break",
            "trapdoor_close",
            "trapdoor_toggle",
            "zombie_chews_iron_door",
            "zombie_converted_villager",
            "wither_shoot",
            "enderdragon_shoot",
            "ender_dragon_growl",
        ],
        true,
    ),
    def(
        "gene",
        &["panda gene"],
        &[
            "normal",
            "aggressive",
            "lazy",
            "worried",
            "playful",
            "weak",
            "brown",
        ],
        true,
    ),
    def(
        "input key",
        &[],
        &["forward", "backward", "left", "right", "jump", "sneak", "sprint"],
        true,
    ),
    def(
        "inventory action",
        &[],
        &[
            "nothing",
            "pickup_all",
            "pickup_some",
            "pickup_half",
            "pickup_one",
            "place_all",
            "place_some",
            "place_one",
            "swap_with_cursor",
            "drop_all_cursor",
            "drop_one_cursor",
            "drop_all_slot",
            "drop_one_slot",
            "move_to_other_inventory",
            "hotbar_move_and_readd",
            "hotbar_swap",
            "clone_stack",
            "collect_to_cursor",
            "unknown",
        ],
        true,
    ),
    def(
        "inventory close reason",
        &["inventory close reasons"],
        &[
            "unknown", "player", "teleport", "cant_use", "plugin", "open_new", "unloaded",
        ],
        true,
    ),
    def(
        "inventory type",
        &[],
        &[
            "chest",
            "dispenser",
            "dropper",
            "furnace",
            "workbench",
            "crafting",
            "enchanting",
            "brewing",
            "player",
            "creative",
            "merchant",
            "ender_chest",
            "anvil",
            "smithing",
            "beacon",
            "hopper",
            "shulker_box",
            "barrel",
            "blast_furnace",
            "lectern",
            "smoker",
            "loom",
            "cartography",
            "grindstone",
            "stonecutter",
            "composter",
            "chiseled_bookshelf",
            "jukebox",
            "decorated_pot",
            "crafter",
            "trial_vault",
        ],
        true,
    ),
    def("item component", &[], data::COMPONENTS, true),
    def(
        "item display transforms",
        &["item display transform", "item display mode"],
        &[
            "none",
            "thirdperson_lefthand",
            "thirdperson_righthand",
            "firstperson_lefthand",
            "firstperson_righthand",
            "head",
            "gui",
            "ground",
            "fixed",
        ],
        true,
    ),
    def(
        "item flag",
        &[],
        &[
            "hide_enchants",
            "hide_attributes",
            "hide_unbreakable",
            "hide_destroys",
            "hide_placed_on",
            "hide_additional_tooltip",
            "hide_dye",
            "hide_armor_trim",
        ],
        true,
    ),
    def(
        "moon phase",
        &[],
        &[
            "full_moon",
            "waning_gibbous",
            "last_quarter",
            "waning_crescent",
            "new_moon",
            "waxing_crescent",
            "first_quarter",
            "waxing_gibbous",
        ],
        true,
    ),
    def("particle", &[], crate::fx_names::PARTICLES, false),
    def(
        "converging particle effect",
        &["converging particle"],
        CONVERGING,
        true,
    ),
    def(
        "directional particle effect",
        &["directional particle"],
        DIRECTIONAL,
        true,
    ),
    def("scalable particle effect", &["scalable particle"], SCALABLE, true),
    def("pig variant", &[], &["temperate", "warm", "cold"], true),
    def(
        "potion effect event action",
        &[],
        &["added", "changed", "cleared", "removed"],
        true,
    ),
    def(
        "potion effect event cause",
        &[],
        &[
            "action",
            "arrow",
            "attack",
            "axolotl",
            "beacon",
            "cold",
            "command",
            "conduit",
            "contagion",
            "death",
            "dolphin",
            "expiration",
            "food",
            "hero_of_the_village",
            "illusion",
            "milk",
            "patrol_captain",
            "penalty",
            "plugin",
            "potion_drink",
            "potion_splash",
            "spider_poison",
            "totem",
            "turtle_helmet",
            "unknown",
            "villager_trade",
            "warding",
            "wither_rose",
        ],
        true,
    ),
    def("potion effect type", &["potion type"], data::EFFECT_TYPES, true),
    def(
        "potion effect type category",
        &[],
        &["beneficial", "harmful", "neutral"],
        true,
    ),
    def(
        "quit reason",
        &[],
        &["disconnected", "kicked", "timed_out", "erroneous_state"],
        true,
    ),
    def(
        "resource pack state",
        &["resource pack status"],
        &[
            "successfully_loaded",
            "declined",
            "failed_download",
            "accepted",
            "downloaded",
            "invalid_url",
            "failed_reload",
            "discarded",
        ],
        true,
    ),
    def("respawn reason", &[], &["death", "end_portal", "plugin"], true),
    def(
        "sound category",
        &[],
        &[
            "master", "music", "records", "weather", "blocks", "hostile", "neutral", "players", "ambient",
            "voice", "ui",
        ],
        true,
    ),
    def(
        "spawn reason",
        &[],
        &[
            "natural",
            "jockey",
            "chunk_gen",
            "spawner",
            "egg",
            "spawner_egg",
            "lightning",
            "bed",
            "build_snowman",
            "build_irongolem",
            "build_wither",
            "village_defense",
            "village_invasion",
            "breeding",
            "slime_split",
            "reinforcements",
            "nether_portal",
            "dispense_egg",
            "infection",
            "cured",
            "ocelot_baby",
            "silverfish_block",
            "mount",
            "trap",
            "ender_pearl",
            "drowned",
            "shoulder_entity",
            "explosion",
            "patrol",
            "raid",
            "default",
            "custom",
            "command",
            "enchantment",
            "duplication",
            "summoned",
            "sheared",
            "metamorphosis",
        ],
        true,
    ),
    def(
        "teleport cause",
        &[],
        &[
            "ender_pearl",
            "command",
            "plugin",
            "nether_portal",
            "end_portal",
            "spectate",
            "end_gateway",
            "chorus_fruit",
            "dismount",
            "exit_bed",
            "consume",
            "unknown",
        ],
        true,
    ),
    def(
        "teleport flag",
        &[],
        &[
            "retain_vehicles",
            "retain_passengers",
            "relative_x",
            "relative_y",
            "relative_z",
            "relative_yaw",
            "relative_pitch",
        ],
        true,
    ),
    def(
        "transform reason",
        &[],
        &[
            "cured",
            "infection",
            "drowned",
            "sheared",
            "lightning",
            "split",
            "unknown",
        ],
        true,
    ),
    def(
        "tree type",
        &[],
        &[
            "tree",
            "big_tree",
            "redwood",
            "tall_redwood",
            "birch",
            "jungle",
            "small_jungle",
            "cocoa_tree",
            "jungle_bush",
            "red_mushroom",
            "brown_mushroom",
            "swamp",
            "acacia",
            "dark_oak",
            "mega_redwood",
            "tall_birch",
            "chorus_plant",
            "crimson_fungus",
            "warped_fungus",
            "azalea",
            "mangrove",
            "tall_mangrove",
            "cherry",
            "pale_oak",
        ],
        true,
    ),
    def(
        "unleash reason",
        &[],
        &["player_unleash", "holder_gone", "distance", "unknown"],
        true,
    ),
    def(
        "villager career change reason",
        &[],
        &["loss_of_job", "employment"],
        true,
    ),
    def("villager profession", &[], data::PROFESSIONS, true),
    def(
        "villager type",
        &[],
        &["desert", "jungle", "plains", "savanna", "snow", "swamp", "taiga"],
        true,
    ),
    def("visual effect", &[], crate::fx_names::PARTICLES, false),
    def("weather type", &["weather"], &["clear", "rain", "thunder"], false),
    def(
        "wolf variant",
        &[],
        &[
            "pale", "spotted", "snowy", "black", "ashen", "rusty", "woods", "chestnut", "striped",
        ],
        true,
    ),
    def(
        "world environment",
        &["environment"],
        &["normal", "nether", "the_end"],
        false,
    ),
    def("zombie nautilus variant", &[], &["temperate", "warm"], true),
];

/// Names of types that are plain object slots: the value is whatever
/// entity, player or thing the script already holds, so a parameter of this
/// type is not checked.
const LOOSE: &[&str] = &[
    "entity",
    "living entity",
    "projectile",
    "vehicle",
    "display entity",
    "nameable",
    "metadata holder",
    "audience",
    "any named thing",
    "any numbered thing",
    "any sized thing",
    "any valued thing",
    "anything with contents",
    "experience",
    "slot",
    "inventory",
    "gamerule value",
    "offline player",
    // (see `entity_block_properties`): values that are texts, lists or plain words in Carve
    "block data",
    "enchantment type",
    "entity type with amount",
    "executable",
    "potion effect",
    "type",
];

/// Words that are too general to mean a named value on their own.
const NOT_LITERAL: &[&str] = &[
    "none", "unknown", "default", "custom", "plugin", "command", "normal", "player", "damage",
];

// ---- names -----------------------------------------------------------------

/// Lowercase, with spaces and dashes turned into `_`.
pub fn norm(s: &str) -> String {
    s.trim()
        .chars()
        .map(|c| {
            if c == ' ' || c == '-' {
                '_'
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect()
}

fn norm_name(s: &str) -> String {
    s.trim()
        .chars()
        .map(|c| {
            if c == '_' || c == '-' {
                ' '
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn type_id_exact(name: &str) -> Option<u8> {
    let n = norm_name(name);
    match n.as_str() {
        "color" | "colour" => return Some(T_COLOR),
        "inventory" => return Some(T_INVENTORY),
        "quaternion" => return Some(T_QUATERNION),
        "timeperiod" | "time period" => return Some(T_TIMEPERIOD),
        "chunk" => return Some(T_CHUNK),
        "queue" => return Some(T_QUEUE),
        "world border" | "worldborder" => return Some(T_BORDER),
        _ => {}
    }
    ENUMS
        .iter()
        .position(|d| d.name == n || d.aliases.contains(&n.as_str()))
        .map(|i| i as u8)
}

/// Type for a name in a function header or `parsed as`: `click type`,
/// `color`, ... or a loose name like `entity` (`Ty::Object`). `click types`
/// is a list.
pub fn ty_from_name(name: &str) -> Option<Ty> {
    let n = norm_name(name);
    if LOOSE.contains(&n.as_str()) {
        return Some(Ty::Object);
    }
    if let Some(id) = type_id_exact(&n) {
        return Some(Ty::T1(id));
    }
    let singular = n.strip_suffix('s')?;
    if LOOSE.contains(&singular) || type_id_exact(singular).is_some() {
        return Some(Ty::List);
    }
    None
}

/// Display name of a type id.
pub fn ty_name(id: u8) -> &'static str {
    match id {
        T_COLOR => "color",
        T_INVENTORY => "inventory",
        T_QUATERNION => "quaternion",
        T_TIMEPERIOD => "timeperiod",
        T_CHUNK => "chunk",
        T_QUEUE => "queue",
        T_BORDER => "world border",
        _ => ENUMS.get(usize::from(id)).map_or("value", |d| d.name),
    }
}

/// A type name starting at `at` in the tokens, longest first. Gives the type
/// and the number of tokens used.
pub fn type_at(toks: &[Tok], at: usize) -> Option<(Ty, usize)> {
    let mut words = Vec::new();
    for t in toks.get(at..)? {
        match t {
            Tok::Word(w) if words.len() < 6 => words.push(w.as_str()),
            _ => break,
        }
    }
    (1..=words.len()).rev().find_map(|n| {
        let t = ty_from_name(&words.get(..n)?.join(" "))?;
        Some((t, n))
    })
}

// ---- literals --------------------------------------------------------------

/// What a bare word phrase can be.
#[derive(Debug, Clone, Copy)]
enum Lit {
    Enum(EnumVal),
    Color(u8, u8, u8),
}

/// The 16 dye colors, with the colors the game paints them in.
const DYES: [(&str, (u8, u8, u8)); 16] = [
    ("white", (0xF9, 0xFF, 0xFE)),
    ("orange", (0xF9, 0x80, 0x1D)),
    ("magenta", (0xC7, 0x4E, 0xBD)),
    ("light_blue", (0x3A, 0xB3, 0xDA)),
    ("yellow", (0xFE, 0xD8, 0x3D)),
    ("lime", (0x80, 0xC7, 0x1F)),
    ("pink", (0xF3, 0x8B, 0xAA)),
    ("gray", (0x47, 0x4F, 0x52)),
    ("light_gray", (0x9D, 0x9D, 0x97)),
    ("cyan", (0x16, 0x9C, 0x9C)),
    ("purple", (0x89, 0x32, 0xB8)),
    ("blue", (0x3C, 0x44, 0xAA)),
    ("brown", (0x83, 0x54, 0x32)),
    ("green", (0x5E, 0x7C, 0x16)),
    ("red", (0xB0, 0x2E, 0x26)),
    ("black", (0x1D, 0x1D, 0x21)),
];

fn index() -> &'static HashMap<String, Lit> {
    static INDEX: OnceLock<HashMap<String, Lit>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut m = HashMap::new();
        for (name, (r, g, b)) in DYES {
            m.insert(name.to_string(), Lit::Color(r, g, b));
        }
        m.insert("grey".into(), Lit::Color(0x47, 0x4F, 0x52));
        m.insert("light_grey".into(), Lit::Color(0x9D, 0x9D, 0x97));
        for (ti, d) in ENUMS.iter().enumerate() {
            if !d.literal {
                continue;
            }
            for (vi, v) in d.values.iter().enumerate() {
                if NOT_LITERAL.contains(v) {
                    continue;
                }
                m.entry((*v).to_string()).or_insert(Lit::Enum(EnumVal {
                    ty: ti as u8,
                    idx: vi as u16,
                }));
            }
        }
        m
    })
}

fn lit_value(l: Lit) -> (Value, Ty) {
    match l {
        Lit::Enum(e) => (Value::Enum(e), Ty::T1(e.ty)),
        Lit::Color(r, g, b) => (Value::Color { r, g, b }, Ty::T1(T_COLOR)),
    }
}

/// Parse `text` as a value of type `ty`.
pub fn parse_text(ty: u8, text: &str) -> Value {
    match ty {
        T_COLOR => parse_color(text).map_or(Value::None, |(r, g, b)| Value::Color { r, g, b }),
        T_QUATERNION | T_TIMEPERIOD | T_CHUNK | T_QUEUE | T_BORDER | T_INVENTORY => Value::None,
        _ => {
            let Some(d) = ENUMS.get(usize::from(ty)) else {
                return Value::None;
            };
            let n = norm(text);
            let n = n.strip_prefix("minecraft:").unwrap_or(&n);
            d.values
                .iter()
                .position(|v| *v == n)
                .map_or(Value::None, |i| Value::Enum(EnumVal { ty, idx: i as u16 }))
        }
    }
}

/// A color from a dye name, a chat color name or `#rrggbb`.
pub fn parse_color(text: &str) -> Option<(u8, u8, u8)> {
    let n = norm(text);
    if let Some((_, c)) = DYES.iter().find(|(k, _)| *k == n) {
        return Some(*c);
    }
    crate::colors::name_rgb(text)
}

// ---- values ----------------------------------------------------------------

impl Value {
    /// Does the text name this named value? (`click type is "left"`)
    pub fn enum_matches_text(e: EnumVal, t: &str) -> bool {
        let n = norm(t);
        let n = n.strip_prefix("minecraft:").unwrap_or(&n);
        n == e.id()
    }
}

/// `==` for the values of this module. `None` when neither side is one.
pub fn loose_eq(a: &Value, b: &Value) -> Option<bool> {
    Some(match (a, b) {
        (Value::Enum(x), Value::Enum(y)) => x == y || x.id() == y.id(),
        (Value::Enum(e), Value::Text(t)) | (Value::Text(t), Value::Enum(e)) => {
            Value::enum_matches_text(*e, t)
        }
        (Value::Color { r, g, b }, Value::Color { r: r2, g: g2, b: b2 }) => (r, g, b) == (r2, g2, b2),
        (Value::Color { r, g, b }, Value::Text(t)) | (Value::Text(t), Value::Color { r, g, b }) => {
            parse_color(t) == Some((*r, *g, *b))
        }
        (Value::Quaternion { .. }, Value::Quaternion { .. })
        | (Value::Timeperiod { .. }, Value::Timeperiod { .. })
        | (Value::Chunk { .. }, Value::Chunk { .. })
        | (Value::Queue(_), Value::Queue(_)) => a == b,
        (Value::Border(x), Value::Border(y)) => x.eq_ignore_ascii_case(y),
        _ => return None,
    })
}

/// `6:00` style clock text for a tick of day (0 is 6:00).
fn clock(ticks: i64) -> String {
    let t = ticks.rem_euclid(24000);
    let hour = (t / 1000 + 6) % 24;
    let minute = (t % 1000) * 60 / 1000;
    format!("{hour}:{minute:02}")
}

/// Print the value types of this module. Returns false for any other value.
pub fn push_to(v: &Value, out: &mut String) -> bool {
    use crate::value::push_num;
    match v {
        Value::Enum(e) => out.push_str(&e.id().replace('_', " ")),
        Value::Color { r, g, b } => out.push_str(&format!("#{r:02x}{g:02x}{b:02x}")),
        Value::Quaternion { w, x, y, z } => {
            for (k, n) in [("w", w), ("x", x), ("y", y), ("z", z)] {
                if k != "w" {
                    out.push_str(", ");
                }
                out.push_str(k);
                out.push_str(": ");
                push_num(out, *n);
            }
        }
        Value::Timeperiod { start, end } => {
            out.push_str(&clock(*start));
            out.push_str(" to ");
            out.push_str(&clock(*end));
        }
        Value::Chunk { x, z, world } => {
            out.push_str(&format!("chunk {x}, {z}"));
            if let Some(w) = world {
                out.push_str(" in '");
                out.push_str(w);
                out.push('\'');
            }
        }
        Value::Queue(items) => {
            out.push_str("queue of ");
            if items.is_empty() {
                out.push_str("nothing");
            }
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                it.push_to(out);
            }
        }
        Value::Border(w) => {
            out.push_str("world border of '");
            out.push_str(w);
            out.push('\'');
        }
        _ => return false,
    }
    true
}

/// Name of the type of one of this module's values.
pub fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Enum(e) => e.type_name(),
        Value::Color { .. } => "color",
        Value::Quaternion { .. } => "quaternion",
        Value::Timeperiod { .. } => "timeperiod",
        Value::Chunk { .. } => "chunk",
        Value::Queue(_) => "queue",
        Value::Border(_) => "world border",
        _ => "value",
    }
}

/// JSON for a saved variable.
pub fn to_json(v: &Value) -> Option<serde_json::Value> {
    use serde_json::json;
    Some(match v {
        Value::Enum(e) => json!({"enum": {"type": e.type_name(), "value": e.id()}}),
        Value::Color { r, g, b } => json!({"color": [r, g, b]}),
        Value::Quaternion { w, x, y, z } => json!({"quaternion": [w, x, y, z]}),
        Value::Timeperiod { start, end } => json!({"timeperiod": [start, end]}),
        Value::Chunk { x, z, world } => json!({"chunk": {"x": x, "z": z, "world": world}}),
        Value::Border(w) => json!({"border": w}),
        _ => return None,
    })
}

/// Back from JSON; `None` when the object is not one of ours.
pub fn from_json(o: &serde_json::Map<String, serde_json::Value>) -> Option<Value> {
    use serde_json::Value as J;
    if let Some(e) = o.get("enum").and_then(J::as_object) {
        let id = type_id_exact(e.get("type")?.as_str()?)?;
        return match parse_text(id, e.get("value")?.as_str()?) {
            Value::None => None,
            v => Some(v),
        };
    }
    let nums = |k: &str| -> Option<Vec<f64>> { o.get(k)?.as_array()?.iter().map(J::as_f64).collect() };
    if let Some(c) = nums("color") {
        let (r, g, b) = (*c.first()?, *c.get(1)?, *c.get(2)?);
        return Some(Value::Color {
            r: r as u8,
            g: g as u8,
            b: b as u8,
        });
    }
    if let Some(q) = nums("quaternion") {
        return Some(Value::Quaternion {
            w: *q.first()?,
            x: *q.get(1)?,
            y: *q.get(2)?,
            z: *q.get(3)?,
        });
    }
    if let Some(p) = nums("timeperiod") {
        return Some(Value::Timeperiod {
            start: *p.first()? as i64,
            end: *p.get(1)? as i64,
        });
    }
    if let Some(c) = o.get("chunk").and_then(J::as_object) {
        return Some(Value::Chunk {
            x: c.get("x")?.as_i64()? as i32,
            z: c.get("z")?.as_i64()? as i32,
            world: c.get("world").and_then(J::as_str).map(str::to_string),
        });
    }
    o.get("border")
        .and_then(J::as_str)
        .map(|w| Value::Border(w.into()))
}

// ---- expressions -----------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// A literal value.
    Const(Value),
    /// `rgb(r, g, b)`
    Rgb,
    /// `color from hex "#rrggbb"`
    HexToColor,
    /// `hex of <color>`
    HexOf,
    /// `quaternion(w, x, y, z)`
    Quat,
    /// `axisAngle(angle, x, y, z)` or `axisAngle(angle, vector)`
    AxisAngle,
    /// `timeperiod(from, to)`: two times of day, in ticks.
    Period,
    /// `queue(a, b, ...)`
    Queue,
    /// `chunk of <location>`
    ChunkOf,
    /// `chunk(x, z[, world])`
    ChunkXz,
    /// `world border of <world>`
    Border,
    /// `<text> parsed as <type>`
    ParseAs(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub op: Op,
    pub args: Vec<Expr>,
    pub ty: Ty,
}

fn node(op: Op, args: Vec<Expr>, ty: u8) -> Expr {
    Expr::T1(Box::new(Node {
        op,
        args,
        ty: Ty::T1(ty),
    }))
}

/// `<expr> parsed as <type>` for the types of this module.
pub fn parse_as(inner: Expr, ty: u8) -> Expr {
    node(Op::ParseAs(ty), vec![inner], ty)
}

fn num(v: &Value) -> Option<f64> {
    v.as_num()
}

/// Evaluate a node whose argument values are already computed.
pub fn apply(op: &Op, vals: &[Value]) -> Result<Value, String> {
    let n = |i: usize, what: &str| -> Result<f64, String> {
        vals.get(i)
            .and_then(num)
            .ok_or_else(|| format!("{what} needs a number"))
    };
    Ok(match op {
        Op::Const(v) => v.clone(),
        Op::Rgb => {
            let c = |i, w| n(i, w).map(|x| x.clamp(0.0, 255.0).round() as u8);
            Value::Color {
                r: c(0, "the red part")?,
                g: c(1, "the green part")?,
                b: c(2, "the blue part")?,
            }
        }
        Op::HexToColor => match vals.first() {
            Some(Value::Text(t)) => parse_color(t).map_or(Value::None, |(r, g, b)| Value::Color { r, g, b }),
            _ => Value::None,
        },
        Op::HexOf => match vals.first() {
            Some(Value::Color { r, g, b }) => Value::Text(format!("#{r:02x}{g:02x}{b:02x}")),
            Some(Value::Num(x)) => {
                let i = *x as i64;
                Value::Text(if i < 0 {
                    format!("-{:x}", -i)
                } else {
                    format!("{i:x}")
                })
            }
            _ => Value::None,
        },
        Op::Quat => Value::Quaternion {
            w: n(0, "w")?,
            x: n(1, "x")?,
            y: n(2, "y")?,
            z: n(3, "z")?,
        },
        Op::AxisAngle => {
            let (angle, x, y, z) = match vals.get(1) {
                Some(Value::Vector { x, y, z }) => (n(0, "the angle")?, *x, *y, *z),
                _ => (n(0, "the angle")?, n(1, "x")?, n(2, "y")?, n(3, "z")?),
            };
            crate::functions_aliases::axis_angle(angle, x, y, z)?
        }
        Op::Period => Value::Timeperiod {
            start: n(0, "the start")? as i64,
            end: n(1, "the end")? as i64,
        },
        Op::Queue => Value::Queue(
            vals.iter()
                .flat_map(|v| match v {
                    Value::List(items) => items.clone(),
                    Value::None => Vec::new(),
                    other => vec![other.clone()],
                })
                .collect(),
        ),
        Op::ChunkOf => match vals.first() {
            Some(Value::Location { x, z, world, .. }) => Value::Chunk {
                x: (x / 16.0).floor() as i32,
                z: (z / 16.0).floor() as i32,
                world: world.clone(),
            },
            _ => Value::None,
        },
        Op::ChunkXz => Value::Chunk {
            x: n(0, "the chunk x")? as i32,
            z: n(1, "the chunk z")? as i32,
            world: match vals.get(2) {
                Some(Value::World(w)) => Some(w.clone()),
                _ => None,
            },
        },
        Op::Border => match vals.first() {
            Some(Value::World(w)) => Value::Border(w.clone()),
            _ => Value::None,
        },
        Op::ParseAs(ty) => match vals.first() {
            Some(Value::Text(t)) => parse_text(*ty, t),
            Some(v @ (Value::Enum(_) | Value::Color { .. })) => {
                let mut s = String::new();
                v.push_to(&mut s);
                parse_text(*ty, &s)
            }
            _ => Value::None,
        },
    })
}

impl P<'_> {
    fn t1_lit(&self, off: usize, w: &str) -> bool {
        if self.pos + off >= self.toks.len() {
            self.hit_end.set(true);
        }
        matches!(self.toks.get(self.pos + off), Some(t) if t.is_lit(w))
    }

    /// `(a, b, c)` with the `(` ahead. Checks the count.
    fn t1_args(&mut self, name: &str, min: usize, max: usize) -> Result<Vec<Expr>, String> {
        self.pos += 1;
        let mut args = Vec::new();
        if !self.eat_sym(")") {
            loop {
                args.push(self.top()?);
                if self.eat_sym(")") {
                    break;
                }
                if !self.eat_sym(",") {
                    return Err(format!("expected ',' or ')' in {name}(...)"));
                }
            }
        }
        if args.len() < min || args.len() > max {
            let want = if min == max {
                min.to_string()
            } else {
                format!("{min} to {max}")
            };
            return Err(format!("{name}() takes {want} value(s), got {}", args.len()));
        }
        Ok(args)
    }

    /// Called with the head word already read. `Ok(None)`: not one of ours.
    pub(crate) fn t1_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let paren = matches!(self.peek(), Some(t) if t.is_lit("("));
        if paren && self.has_function(lw) {
            return Ok(None);
        }
        // `light blue wool` is an item, `light blue` a color.
        if matches!(lw, "light" | "dark")
            && matches!(self.toks.get(self.pos), Some(Tok::Word(_)))
            && let Some(item) = self.item_at_prev(lw)
        {
            return Ok(Some(item));
        }
        if lw == "light"
            && matches!(self.toks.get(self.pos), Some(Tok::Word(_) | Tok::Num(_)))
            && let Some((l, n, joined)) = self.t1_phrase(lw, 2)
        {
            return Ok(self.t1_take(l, n, &joined));
        }
        let e = match lw {
            "rgb" if paren => {
                let args = self.t1_args("rgb", 3, 3)?;
                node(Op::Rgb, args, T_COLOR)
            }
            "quaternion" if paren => {
                let args = self.t1_args("quaternion", 4, 4)?;
                node(Op::Quat, args, T_QUATERNION)
            }
            "axisangle" if paren => {
                let args = self.t1_args("axisAngle", 2, 4)?;
                if args.len() == 3 {
                    return Err("axisAngle(angle, x, y, z) or axisAngle(angle, vector)".into());
                }
                node(Op::AxisAngle, args, T_QUATERNION)
            }
            "identity" if self.t1_lit(0, "quaternion") => {
                self.pos += 1;
                node(
                    Op::Const(Value::Quaternion {
                        w: 1.0,
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    vec![],
                    T_QUATERNION,
                )
            }
            "timeperiod" if paren => {
                let args = self.t1_args("timeperiod", 2, 2)?;
                node(Op::Period, args, T_TIMEPERIOD)
            }
            "time"
                if self.t1_lit(0, "period")
                    && matches!(self.toks.get(self.pos + 1), Some(t) if t.is_lit("(")) =>
            {
                self.pos += 1;
                let args = self.t1_args("time period", 2, 2)?;
                node(Op::Period, args, T_TIMEPERIOD)
            }
            "queue" if paren => {
                let args = self.t1_args("queue", 0, usize::MAX)?;
                node(Op::Queue, args, T_QUEUE)
            }
            "chunk" if paren => {
                let args = self.t1_args("chunk", 2, 3)?;
                node(Op::ChunkXz, args, T_CHUNK)
            }
            "chunk" if self.t1_lit(0, "of") || self.t1_lit(0, "at") => {
                self.pos += 1;
                let loc = self.primary()?;
                if !matches!(loc.ty(self.env), Ty::Location | Ty::Player | Ty::Object) {
                    return Err("'chunk of' needs a location".into());
                }
                node(Op::ChunkOf, vec![loc], T_CHUNK)
            }
            "world" if self.t1_lit(0, "border") && self.t1_lit(1, "of") => {
                self.pos += 2;
                let w = self.primary()?;
                node(Op::Border, vec![w], T_BORDER)
            }
            "color" | "colour" if self.t1_lit(0, "from") && self.t1_lit(1, "hex") => {
                self.pos += 2;
                let t = self.primary()?;
                node(Op::HexToColor, vec![t], T_COLOR)
            }
            "hex" if self.t1_lit(0, "of") => {
                self.pos += 1;
                let at = self.pos;
                let Ok(c) = self.primary() else {
                    self.pos = at - 1;
                    return Ok(None);
                };
                if !matches!(c.ty(self.env), Ty::T1(T_COLOR) | Ty::Object) {
                    // `hex of 255` is the number form from the utilities page.
                    self.pos = at - 1;
                    return Ok(None);
                }
                Expr::T1(Box::new(Node {
                    op: Op::HexOf,
                    args: vec![c],
                    ty: Ty::Text,
                }))
            }
            _ => return Ok(None),
        };
        Ok(Some(e))
    }

    /// The longest word phrase from the head word `first` (already read) that
    /// names a value. Gives the value, the words used and the joined name.
    fn t1_phrase(&self, first: &str, min_words: usize) -> Option<(Lit, usize, String)> {
        let mut words = vec![first.to_ascii_lowercase()];
        for t in self.toks.get(self.pos..).unwrap_or(&[]) {
            if words.len() >= 4 {
                break;
            }
            match t {
                Tok::Word(w) => words.push(w.to_ascii_lowercase()),
                Tok::Num(n) if n.fract() == 0.0 && *n >= 0.0 && *n < 1000.0 => {
                    words.push(format!("{}", *n as u32))
                }
                _ => break,
            }
        }
        let idx = index();
        (min_words..=words.len()).rev().find_map(|n| {
            let joined = words.get(..n)?.join("_");
            idx.get(&joined).map(|l| (*l, n, joined))
        })
    }

    /// A bare word phrase that names a value (`left`, `full moon`, `red`).
    /// `first` is the word already read. Tried last, so any other meaning wins.
    pub(crate) fn t1_literal(&mut self, first: &str) -> Option<Expr> {
        let (l, n, joined) = self.t1_phrase(first, 1)?;
        self.t1_take(l, n, &joined)
    }

    fn t1_take(&mut self, l: Lit, n: usize, joined: &str) -> Option<Expr> {
        self.pos = self.pos - 1 + n;
        if self.pos >= self.toks.len() {
            let prefix = format!("{joined}_");
            if index().keys().any(|k| k.starts_with(&prefix)) {
                self.hit_end.set(true);
            }
        }
        let (value, ty) = lit_value(l);
        let Ty::T1(ty) = ty else { return None };
        Some(node(Op::Const(value), vec![], ty))
    }
}
