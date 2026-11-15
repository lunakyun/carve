//! A mixed batch of expressions: `not x`, number limits, item amount,
//! durability, item cooldown, `last color of`, `language of`, last damage
//! facts, last death location, join message, resource pack response and
//! highest/lowest solid block.
//!
//! They are rare, so they share one boxed `Expr::W4` and one boxed
//! `Effect::W4`. The hot enums in `engine.rs` do not grow.

use crate::compile::{EffKind, Effect};
use crate::expr::{Expr, Ty};
use crate::host::Host;
use crate::pattern::Capture;
use crate::value::{PlayerRef, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum W4Op {
    /// `not x`
    Not,
    MaxDouble,
    MaxFloat,
    /// `item amount of <item>`
    ItemAmount,
    /// `last color of <text>`
    LastColor,
    /// `max durability of <item>`
    MaxDurability,
    /// `max item use time of <item>`
    MaxUseTime,
    /// `key of <item>`
    Key,
    /// `language of <player>`
    Language,
    /// `cooldown of <item> for <player>`
    Cooldown,
    /// `max freeze time of <entity>`
    MaxFreeze,
    /// `last death location of <player>`
    LastDeathLoc,
    /// `last attacker of <player>`
    LastAttacker,
    /// `last damage of <player>`
    LastDamage,
    /// `last damage cause of <player>`
    LastDamageCause,
    /// `join message`
    JoinMessage,
    /// `[last] resource pack response [of <player>]`
    PackResponse,
    /// `light level at <location>`, `block light level at`, `sky light level at`
    Light,
    BlockLight,
    SkyLight,
    /// `highest block at <location>`
    Highest,
    /// `lowest block at <location>`
    Lowest,
}

#[derive(Debug, Clone, PartialEq)]
pub struct W4 {
    pub op: W4Op,
    pub args: Vec<Expr>,
}

impl W4 {
    pub fn ty(&self) -> Ty {
        use W4Op::*;
        match self.op {
            Not => Ty::Boolean,
            MaxDouble | MaxFloat | ItemAmount | MaxDurability | LastDamage | Light | BlockLight
            | SkyLight => Ty::Number,
            LastColor | Key | Language | LastDamageCause | JoinMessage | PackResponse => Ty::Text,
            MaxUseTime | Cooldown | MaxFreeze => Ty::Timespan,
            LastDeathLoc | Highest | Lowest => Ty::Location,
            LastAttacker => Ty::Player,
        }
    }
}

/// What the host remembers about the last hit a player took.
#[derive(Debug, Clone, PartialEq)]
pub struct DamageInfo {
    pub amount: f64,
    /// Lowercase damage type name such as `fall` or `player_attack`.
    pub cause: String,
    /// The attacking player, when a player did it.
    pub attacker: Option<PlayerRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum W4Eff {
    /// `set cooldown of <item> for <player> to <timespan>`
    SetCooldown,
    /// `reset cooldown of <item> for <player>`
    ResetCooldown,
    /// `set join message to <text>`
    SetJoinMessage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct W4Effect {
    pub op: W4Eff,
    pub args: Vec<Expr>,
}

pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    (
        "set [the] [item] cooldown of %item% for %player% to %timespan%",
        EffKind::W4(W4Eff::SetCooldown),
    ),
    (
        "reset [the] [item] cooldown of %item% for %player%",
        EffKind::W4(W4Eff::ResetCooldown),
    ),
    (
        "set [the] join message to %text%",
        EffKind::W4(W4Eff::SetJoinMessage),
    ),
];

/// Build the effect from the pattern captures, in pattern order.
pub fn build(op: W4Eff, caps: &mut [Option<Capture>]) -> Option<Effect> {
    let mut args = Vec::new();
    for c in caps.iter_mut() {
        if let Some(Capture::Expr(e)) = c.take() {
            args.push(e);
        }
    }
    Some(Effect::W4(Box::new(W4Effect { op, args })))
}

fn item_id(v: &Value) -> Option<String> {
    match v {
        Value::Item { id, .. } => Some(id.clone()),
        Value::Text(t) => {
            let t = t.trim().to_ascii_lowercase();
            let t = t.strip_prefix("minecraft:").unwrap_or(&t);
            Some(t.replace(' ', "_"))
        }
        _ => None,
    }
}

/// Run an effect. Arguments are already evaluated; players are resolved.
pub fn run(op: W4Eff, args: &[Value], host: &mut dyn Host) {
    match (op, args) {
        (W4Eff::SetCooldown, [item, Value::Player(p), Value::Timespan { millis }]) => {
            if let Some(id) = item_id(item) {
                host.set_item_cooldown(p, &id, (*millis / 50).max(0));
            }
        }
        (W4Eff::ResetCooldown, [item, Value::Player(p)]) => {
            if let Some(id) = item_id(item) {
                host.set_item_cooldown(p, &id, 0);
            }
        }
        (W4Eff::SetJoinMessage, [Value::Text(t)]) => host.set_join_message(t),
        _ => {}
    }
}

fn span_ticks(ticks: i64) -> Value {
    Value::Timespan { millis: ticks * 50 }
}

/// Evaluate an expression from its already evaluated arguments.
pub fn apply(op: W4Op, args: &[Value], host: &mut dyn Host) -> Value {
    use W4Op::*;
    let a0 = args.first().unwrap_or(&Value::None);
    let player = match a0 {
        Value::Player(p) => Some(p),
        _ => None,
    };
    match op {
        Not => match a0 {
            Value::Bool(b) => Value::Bool(!b),
            Value::None => Value::Bool(true),
            _ => Value::None,
        },
        MaxDouble => Value::Num(f64::MAX),
        MaxFloat => Value::Num(f64::from(f32::MAX)),
        ItemAmount => match a0 {
            Value::Item { count, .. } => Value::Num(f64::from(*count)),
            _ => Value::None,
        },
        LastColor => match a0 {
            Value::Text(t) => last_color(t).map_or(Value::None, Value::Text),
            _ => Value::None,
        },
        MaxDurability => item_id(a0)
            .and_then(|id| max_durability(&id))
            .map_or(Value::None, |n| Value::Num(f64::from(n))),
        MaxUseTime => item_id(a0)
            .and_then(|id| max_use_ticks(&id))
            .map_or(Value::None, |t| span_ticks(i64::from(t))),
        Key => crate::parsing_loot_components::key_of(a0),
        MaxFreeze => match a0 {
            Value::None => Value::None,
            _ => span_ticks(140),
        },
        Language => player
            .and_then(|p| host.player_info(p, crate::skbee_utilities::Info::Locale))
            .map_or(Value::None, Value::Text),
        Cooldown => match (item_id(a0), args.get(1)) {
            (Some(id), Some(Value::Player(p))) => span_ticks(host.item_cooldown(p, &id).unwrap_or(0).max(0)),
            _ => Value::None,
        },
        LastDeathLoc => {
            player
                .and_then(|p| host.last_death_location(p))
                .map_or(Value::None, |(x, y, z, w)| Value::Location {
                    x,
                    y,
                    z,
                    world: Some(w),
                })
        }
        LastAttacker => player
            .and_then(|p| host.last_damage(p))
            .and_then(|d| d.attacker)
            .map_or(Value::None, Value::Player),
        LastDamage => player
            .and_then(|p| host.last_damage(p))
            .map_or(Value::None, |d| Value::Num(d.amount)),
        LastDamageCause => player
            .and_then(|p| host.last_damage(p))
            .map_or(Value::None, |d| Value::Text(d.cause)),
        JoinMessage => host.join_message().map_or(Value::None, Value::Text),
        PackResponse => player
            .and_then(|p| host.resource_pack_status(p))
            .map_or(Value::None, Value::Text),
        Light | BlockLight | SkyLight => match a0 {
            Value::Location { x, y, z, world } => {
                let kind = match op {
                    BlockLight => 0,
                    SkyLight => 1,
                    _ => 2,
                };
                // `location(x, y, z)` has no world: use the default one.
                world
                    .clone()
                    .or_else(|| host.default_world())
                    .and_then(|w| {
                        host.light_level(&w, x.floor() as i32, y.floor() as i32, z.floor() as i32, kind)
                    })
                    .map_or(Value::None, |l| Value::Num(f64::from(l)))
            }
            _ => Value::None,
        },
        Highest | Lowest => match a0 {
            Value::Location { x, z, world, .. } => {
                let Some(w) = world.clone().or_else(|| host.default_world()) else {
                    return Value::None;
                };
                let (bx, bz) = (x.floor() as i32, z.floor() as i32);
                host.solid_block_y(&w, bx, bz, op == Highest)
                    .map_or(Value::None, |y| Value::Location {
                        x: f64::from(bx),
                        y: f64::from(y),
                        z: f64::from(bz),
                        world: Some(w),
                    })
            }
            _ => Value::None,
        },
    }
}

const COLOR_NAMES: [(&str, (u8, u8, u8)); 16] = [
    ("black", (0x00, 0x00, 0x00)),
    ("dark_blue", (0x00, 0x00, 0xAA)),
    ("dark_green", (0x00, 0xAA, 0x00)),
    ("dark_aqua", (0x00, 0xAA, 0xAA)),
    ("dark_red", (0xAA, 0x00, 0x00)),
    ("dark_purple", (0xAA, 0x00, 0xAA)),
    ("gold", (0xFF, 0xAA, 0x00)),
    ("gray", (0xAA, 0xAA, 0xAA)),
    ("dark_gray", (0x55, 0x55, 0x55)),
    ("blue", (0x55, 0x55, 0xFF)),
    ("green", (0x55, 0xFF, 0x55)),
    ("aqua", (0x55, 0xFF, 0xFF)),
    ("red", (0xFF, 0x55, 0x55)),
    ("light_purple", (0xFF, 0x55, 0xFF)),
    ("yellow", (0xFF, 0xFF, 0x55)),
    ("white", (0xFF, 0xFF, 0xFF)),
];

/// The color that is active at the end of a text, as a color name (or
/// `#rrggbb` for custom colors). `None` when the text has no color.
pub fn last_color(text: &str) -> Option<String> {
    let spans = crate::colors::parse(text);
    let rgb = spans.iter().rev().find_map(|s| s.style.color)?;
    Some(match COLOR_NAMES.iter().find(|(_, c)| *c == rgb) {
        Some((n, _)) => (*n).to_string(),
        None => format!("#{:02x}{:02x}{:02x}", rgb.0, rgb.1, rgb.2),
    })
}

/// Maximum durability of a damageable item. `None` for everything else.
pub fn max_durability(id: &str) -> Option<u32> {
    const TOOLS: [(&str, u32); 7] = [
        ("wooden", 59),
        ("stone", 131),
        ("copper", 190),
        ("iron", 250),
        ("golden", 32),
        ("diamond", 1561),
        ("netherite", 2031),
    ];
    const ARMOR: [(&str, u32); 6] = [
        ("leather", 5),
        ("golden", 7),
        ("chainmail", 15),
        ("iron", 15),
        ("diamond", 33),
        ("netherite", 37),
    ];
    for part in ["sword", "pickaxe", "axe", "shovel", "hoe"] {
        if let Some(tier) = id.strip_suffix(part).and_then(|t| t.strip_suffix('_')) {
            return TOOLS.iter().find(|(n, _)| *n == tier).map(|(_, d)| *d);
        }
    }
    for (part, base) in [
        ("helmet", 11u32),
        ("chestplate", 16),
        ("leggings", 15),
        ("boots", 13),
    ] {
        if let Some(tier) = id.strip_suffix(part).and_then(|t| t.strip_suffix('_')) {
            if tier == "turtle" && part == "helmet" {
                return Some(275);
            }
            if tier == "copper" {
                return Some(base * 11);
            }
            return ARMOR.iter().find(|(n, _)| *n == tier).map(|(_, m)| base * m);
        }
    }
    Some(match id {
        "bow" => 384,
        "crossbow" => 465,
        "trident" => 250,
        "shield" => 336,
        "fishing_rod" => 64,
        "flint_and_steel" => 64,
        "shears" => 238,
        "elytra" => 432,
        "carrot_on_a_stick" => 25,
        "warped_fungus_on_a_stick" => 100,
        "brush" => 64,
        "mace" => 500,
        "wolf_armor" => 64,
        _ => return None,
    })
}

/// Ticks an item can be used for in one go. `None` for items you cannot hold down.
pub fn max_use_ticks(id: &str) -> Option<u32> {
    Some(match id {
        "bow" | "trident" | "shield" => 72000,
        "crossbow" => 25,
        "spyglass" => 1200,
        "goat_horn" => 140,
        "honey_bottle" => 40,
        "dried_kelp" => 16,
        "potion" | "milk_bucket" | "ominous_bottle" => 32,
        "brush" => 200,
        _ if is_food(id) => 32,
        _ => return None,
    })
}

fn is_food(id: &str) -> bool {
    matches!(
        id,
        "apple"
            | "golden_apple"
            | "enchanted_golden_apple"
            | "bread"
            | "cooked_beef"
            | "beef"
            | "cooked_porkchop"
            | "porkchop"
            | "cooked_chicken"
            | "chicken"
            | "cooked_mutton"
            | "mutton"
            | "cooked_rabbit"
            | "rabbit"
            | "cooked_cod"
            | "cod"
            | "cooked_salmon"
            | "salmon"
            | "carrot"
            | "golden_carrot"
            | "potato"
            | "baked_potato"
            | "poisonous_potato"
            | "beetroot"
            | "melon_slice"
            | "sweet_berries"
            | "glow_berries"
            | "cookie"
            | "pumpkin_pie"
            | "mushroom_stew"
            | "rabbit_stew"
            | "beetroot_soup"
            | "suspicious_stew"
            | "rotten_flesh"
            | "spider_eye"
            | "pufferfish"
            | "tropical_fish"
            | "chorus_fruit"
    )
}
