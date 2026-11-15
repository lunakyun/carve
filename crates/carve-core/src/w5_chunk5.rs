//! Potion effect values, particle values, queues and a set of smaller
//! expressions: raw names, enchantment levels, boss bar keys and progress,
//! client weather, redstone power, respawn anchor charges, inventory rows,
//! slots and viewers, player skulls, potion items, `result of function`,
//! `respawn reason` and `on-screen kick message`.
//!
//! Every expression is one boxed [`Node`] (`Expr::W5c5`), so the hot `Expr`
//! and `eval` paths stay small. The engine evaluates the arguments and calls
//! [`apply`]. Game facts come through one host method, [`Host::w5c5`], with a
//! [`Q`] that says what is wanted.

use crate::compile::{EffKind, Effect};
use crate::expr::{Expr, P, Part, Ty};
use crate::host::{Host, LogLevel};
use crate::lexer::Tok;
use crate::pattern::Capture;
use crate::value::{PlayerRef, Value, push_num};
use crate::world_effects::{FxPos, ParticleFx, effect_id};

// ---- values -------------------------------------------------------------------

/// A value of this module. Not saved in global variables.
#[derive(Debug, Clone, PartialEq)]
pub enum Obj {
    Potion(PotionVal),
    Particle(PartVal),
}

/// A potion effect: `potion effect of speed of tier 2 for 10 seconds`.
#[derive(Debug, Clone, PartialEq)]
pub struct PotionVal {
    /// Vanilla id, like `jump_boost`.
    pub effect: String,
    /// 0 for tier 1.
    pub amplifier: u8,
    /// [`INFINITE`] for infinite effects.
    pub ticks: u32,
    pub ambient: bool,
    pub particles: bool,
}

/// Ticks of an infinite effect. Pumpkin stores durations as `i32`, and -1
/// means infinite there.
pub const INFINITE: u32 = u32::MAX;

/// A particle with its count, offset and speed, ready to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct PartVal {
    pub name: String,
    /// 0: the offset is a velocity (directional particles).
    pub count: u32,
    pub offset: [f64; 3],
    pub speed: f64,
}

impl Obj {
    pub fn type_name(&self) -> &'static str {
        match self {
            Obj::Potion(_) => "potion effect",
            Obj::Particle(_) => "particle",
        }
    }

    pub fn push_to(&self, out: &mut String) {
        match self {
            Obj::Potion(p) => {
                if p.ambient {
                    out.push_str("ambient ");
                }
                if p.ticks == INFINITE {
                    out.push_str("infinite ");
                }
                out.push_str(&p.effect.replace('_', " "));
                out.push(' ');
                push_num(out, f64::from(p.amplifier) + 1.0);
                if p.ticks != INFINITE {
                    out.push_str(" for ");
                    crate::timespans::push_span(out, i64::from(p.ticks) * 50);
                }
                if !p.particles {
                    out.push_str(" without particles");
                }
            }
            Obj::Particle(p) => {
                out.push_str(&p.name.replace('_', " "));
                out.push_str(" particle (count ");
                push_num(out, f64::from(p.count));
                out.push_str(", offset ");
                for (i, n) in p.offset.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    push_num(out, *n);
                }
                out.push_str(", speed ");
                push_num(out, p.speed);
                out.push(')');
            }
        }
    }
}

fn potion_value(p: PotionVal) -> Value {
    Value::W5(Box::new(Obj::Potion(p)))
}

fn particle_value(p: PartVal) -> Value {
    Value::W5(Box::new(Obj::Particle(p)))
}

// ---- host queries ---------------------------------------------------------------

/// What [`Host::w5c5`] is asked. The answer is a [`Value`]; `Value::None`
/// means unknown or not supported.
#[derive(Debug)]
pub enum Q<'a> {
    /// Active potion effects of a player: a list of potion effect values.
    Effects(&'a PlayerRef),
    /// Give a player a potion effect. `Bool(true)` when done; otherwise the
    /// engine falls back to `Host::add_effect`.
    AddEffect(&'a PlayerRef, &'a PotionVal),
    /// Progress of a boss bar, 0 to 100.
    BarProgress(&'a str),
    /// Does a boss bar with this id exist? A boolean.
    BarExists(&'a str),
    /// Client weather of a player: `"clear"`, `"rain"` or none (no custom weather).
    Weather(&'a PlayerRef),
    /// Set (`Some(rain)`) or reset (`None`) the client weather of a player.
    SetWeather(&'a PlayerRef, Option<bool>),
    /// A block's state: a list of the block id, its state id and its default state id.
    BlockState(&'a crate::block_ops::BlockPos),
    /// Set the state id of a block.
    SetBlockState(&'a crate::block_ops::BlockPos, u16),
    /// Players who have this chest menu open: menu id (0 for a menu not
    /// built from a value) and, for those, the player it was opened for.
    Viewers(u64, Option<&'a str>),
}

// ---- tables ------------------------------------------------------------------------

const HARMFUL: &[&str] = &[
    "slowness",
    "mining_fatigue",
    "instant_damage",
    "nausea",
    "blindness",
    "hunger",
    "weakness",
    "poison",
    "wither",
    "levitation",
    "unluck",
    "darkness",
    "wind_charged",
    "weaving",
    "oozing",
    "infested",
];

const NEUTRAL: &[&str] = &["glowing", "bad_omen", "trial_omen", "raid_omen"];

/// `beneficial`, `harmful` or `neutral`, as in vanilla.
pub fn category(effect: &str) -> &'static str {
    if HARMFUL.contains(&effect) {
        "harmful"
    } else if NEUTRAL.contains(&effect) {
        "neutral"
    } else {
        "beneficial"
    }
}

/// Highest level of an enchantment in vanilla.
pub fn max_level(id: &str) -> Option<u32> {
    Some(match id {
        "protection"
        | "fire_protection"
        | "feather_falling"
        | "blast_protection"
        | "projectile_protection"
        | "piercing"
        | "breach" => 4,
        "sharpness" | "smite" | "bane_of_arthropods" | "efficiency" | "power" | "impaling" | "density" => 5,
        "respiration" | "thorns" | "depth_strider" | "soul_speed" | "swift_sneak" | "looting"
        | "sweeping_edge" | "unbreaking" | "fortune" | "luck_of_the_sea" | "lure" | "loyalty" | "riptide"
        | "quick_charge" | "wind_burst" | "lunge" => 3,
        "frost_walker" | "knockback" | "fire_aspect" | "punch" => 2,
        "aqua_affinity" | "binding_curse" | "silk_touch" | "flame" | "infinity" | "channeling"
        | "multishot" | "mending" | "vanishing_curse" => 1,
        _ => return None,
    })
}

/// The vanilla potion that gives this effect, for `potion of speed`.
fn base_potion(effect: &str) -> Option<&'static str> {
    Some(match effect {
        "speed" => "swiftness",
        "slowness" => "slowness",
        "strength" => "strength",
        "instant_health" => "healing",
        "instant_damage" => "harming",
        "jump_boost" => "leaping",
        "regeneration" => "regeneration",
        "fire_resistance" => "fire_resistance",
        "water_breathing" => "water_breathing",
        "invisibility" => "invisibility",
        "night_vision" => "night_vision",
        "weakness" => "weakness",
        "poison" => "poison",
        "luck" => "luck",
        "slow_falling" => "slow_falling",
        "wind_charged" => "wind_charged",
        "weaving" => "weaving",
        "oozing" => "oozing",
        "infested" => "infested",
        _ => return None,
    })
}

/// Redstone power read from a block state. `idx` is the state's index among
/// the block's states (vanilla order: the last property changes fastest).
pub fn redstone_power(block: &str, idx: i64) -> i64 {
    match block {
        // east, north, power, south, west (3 x 3 x 16 x 3 x 3)
        "redstone_wire" => (idx / 9).rem_euclid(16),
        "light_weighted_pressure_plate" | "heavy_weighted_pressure_plate" | "target" => idx.rem_euclid(16),
        // inverted, power
        "daylight_detector" => idx.rem_euclid(16),
        "redstone_block" => 15,
        // `lit` / `powered` is the last property, `true` first.
        "redstone_torch" | "redstone_wall_torch" | "lever" | "repeater" if idx.rem_euclid(2) == 0 => 15,
        b if b.ends_with("_button") && idx.rem_euclid(2) == 0 => 15,
        _ => 0,
    }
}

/// Index of the default state among the block's states, for blocks whose
/// default is not their first state.
fn default_index(block: &str) -> i64 {
    match block {
        // east=none, north=none, power=0, south=none, west=none
        "redstone_wire" => 1160,
        // inverted=false
        "daylight_detector" => 16,
        // face=wall, facing=north, powered=false
        "lever" => 9,
        b if b.ends_with("_button") => 9,
        // delay=1, facing=north, locked=false, powered=false
        "repeater" => 3,
        // lit=false for the wall torch's default? No: lit=true, facing=north.
        _ => 0,
    }
}

// ---- nodes -------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PField {
    Count,
    Offset,
    Distribution,
    Velocity,
    Speed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotField {
    /// 0 based.
    Amplifier,
    /// 1 based.
    Tier,
    Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// effect, tier, duration (a negative number: the default).
    NewPotion {
        ambient: bool,
        infinite: bool,
        hidden: bool,
    },
    PotionGet(PotField),
    /// potion, new value.
    PotionSet(PotField),
    /// player.
    EffectsOf,
    /// effect type, player.
    EffectOf,
    /// effect type, player: the tier, or none.
    TierOf,
    /// effect type.
    Category,
    /// 0 potion, 1 splash, 2 lingering: effect type or potion effect.
    PotionItem(u8),
    /// player or name.
    Skull,
    Particle(&'static str),
    PartGet(PField),
    /// particle, value.
    PartWith(PField),
    /// items.
    QueueNew,
    /// true: start.
    QueueEnd(bool),
    RawName,
    /// true: maximum.
    EnchLevel(bool),
    BarKey,
    BarProgress,
    Weather,
    Redstone,
    /// true: maximum.
    Charges(bool),
    InvRows,
    InvSlots,
    InvHolder,
    InvViewers,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub op: Op,
    pub args: Vec<Expr>,
    pub ty: Ty,
}

fn node(op: Op, args: Vec<Expr>, ty: Ty) -> Expr {
    Expr::W5c5(Box::new(Node { op, args, ty }))
}

fn lit_text(s: &str) -> Expr {
    Expr::Text(vec![Part::Lit(s.to_string())])
}

// ---- evaluation ----------------------------------------------------------------------

/// The vanilla effect id of a script value (`speed`, `"jump boost"`).
pub fn effect_of(v: &Value) -> Option<&'static str> {
    let s = match v {
        Value::Enum(e) => e.id().to_string(),
        Value::Text(t) => t.trim().to_ascii_lowercase().replace(' ', "_"),
        Value::W5(o) => match &**o {
            Obj::Potion(p) => p.effect.clone(),
            Obj::Particle(_) => return None,
        },
        _ => return None,
    };
    effect_id(s.strip_prefix("minecraft:").unwrap_or(&s))
}

fn num(v: &Value) -> Option<f64> {
    v.as_num().filter(|n| n.is_finite())
}

fn potion(v: &Value) -> Option<&PotionVal> {
    match v {
        Value::W5(o) => match &**o {
            Obj::Potion(p) => Some(p),
            Obj::Particle(_) => None,
        },
        _ => None,
    }
}

fn particle(v: &Value) -> Option<&PartVal> {
    match v {
        Value::W5(o) => match &**o {
            Obj::Particle(p) => Some(p),
            Obj::Potion(_) => None,
        },
        Value::Enum(e) if e.type_name() == "particle" => None,
        _ => None,
    }
}

/// A particle value from a particle value or a particle name.
fn particle_or_named(v: &Value) -> Option<PartVal> {
    if let Some(p) = particle(v) {
        return Some(p.clone());
    }
    let s = match v {
        Value::Enum(e) if e.type_name() == "particle" => e.id().to_string(),
        Value::Text(t) => t.trim().to_ascii_lowercase().replace(' ', "_"),
        _ => return None,
    };
    let name = particle_id(s.strip_prefix("minecraft:").unwrap_or(&s))?;
    Some(new_particle(name))
}

fn new_particle(name: &str) -> PartVal {
    PartVal {
        name: name.to_string(),
        count: 1,
        offset: [0.0; 3],
        speed: 0.0,
    }
}

pub fn particle_id(words: &str) -> Option<&'static str> {
    crate::fx_names::PARTICLES.iter().copied().find(|p| *p == words)
}

fn vector(v: &Value) -> Option<[f64; 3]> {
    match v {
        Value::Vector { x, y, z } => Some([*x, *y, *z]),
        Value::Num(n) => Some([*n; 3]),
        _ => None,
    }
    .filter(|a| a.iter().all(|n| n.is_finite()))
}

fn block_pos(v: &Value) -> Option<crate::block_ops::BlockPos> {
    match v {
        Value::Location { x, y, z, world } => Some(crate::block_ops::BlockPos {
            x: x.floor() as i32,
            y: y.floor() as i32,
            z: z.floor() as i32,
            world: world.clone(),
        }),
        _ => None,
    }
}

/// Block id, state index and default state id of a block.
fn block_state(v: &Value, host: &mut dyn Host) -> Option<(String, i64, i64)> {
    let pos = block_pos(v)?;
    let Value::List(parts) = host.w5c5(Q::BlockState(&pos)) else {
        return None;
    };
    let [Value::Text(name), Value::Num(state), Value::Num(default)] = parts.as_slice() else {
        return None;
    };
    let name = name.strip_prefix("minecraft:").unwrap_or(name).to_string();
    let first = *default as i64 - default_index(&name);
    Some((name, *state as i64 - first, *default as i64))
}

fn named(ty_name: &str, id: &str) -> Value {
    match crate::types_1::ty_from_name(ty_name) {
        Some(Ty::T1(t)) => match crate::types_1::parse_text(t, id) {
            Value::None => Value::Text(id.to_string()),
            v => v,
        },
        _ => Value::Text(id.to_string()),
    }
}

fn snbt_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        if c == '"' || c == '\\' {
            o.push('\\');
        }
        o.push(c);
    }
    o.push('"');
    o
}

fn item_with(id: &str, comp: &str, snbt: String) -> Value {
    let meta = crate::item_meta::ItemMeta {
        comps: vec![(comp.to_string(), snbt)],
        ..crate::item_meta::ItemMeta::default()
    };
    Value::Item {
        id: id.to_string(),
        count: 1,
        meta: Some(Box::new(meta)),
    }
}

fn duration_ticks(v: &Value) -> Option<u32> {
    match v {
        Value::Timespan { millis } => Some((millis / 50).clamp(0, i64::from(i32::MAX)) as u32),
        Value::Num(n) if n.is_finite() => Some(n.clamp(0.0, f64::from(i32::MAX)) as u32),
        _ => None,
    }
}

/// Default length of a new potion effect, like `apply speed to player`.
const DEFAULT_TICKS: u32 = 15 * 20;

pub fn apply(op: Op, vals: &[Value], host: &mut dyn Host) -> Result<Value, String> {
    let a = vals.first().unwrap_or(&Value::None);
    let b = vals.get(1).unwrap_or(&Value::None);
    Ok(match op {
        Op::NewPotion {
            ambient,
            infinite,
            hidden,
        } => {
            let Some(effect) = effect_of(a) else {
                return Err(format!("'{a}' is not a potion effect type"));
            };
            let tier = num(b).unwrap_or(1.0).clamp(1.0, 256.0);
            let ticks = if infinite {
                INFINITE
            } else {
                match vals.get(2) {
                    Some(Value::Num(n)) if *n < 0.0 => DEFAULT_TICKS,
                    Some(v) => duration_ticks(v).ok_or_else(|| format!("'{v}' is not a time span"))?,
                    None => DEFAULT_TICKS,
                }
            };
            potion_value(PotionVal {
                effect: effect.to_string(),
                amplifier: (tier as u32 - 1).min(255) as u8,
                ticks,
                ambient,
                particles: !hidden,
            })
        }
        Op::PotionGet(f) => match potion(a) {
            Some(p) => match f {
                PotField::Amplifier => Value::Num(f64::from(p.amplifier)),
                PotField::Tier => Value::Num(f64::from(p.amplifier) + 1.0),
                PotField::Duration if p.ticks == INFINITE => Value::None,
                PotField::Duration => Value::Timespan {
                    millis: i64::from(p.ticks) * 50,
                },
            },
            None => Value::None,
        },
        Op::PotionSet(f) => {
            let Some(p) = potion(a) else {
                return Err(format!("expected a potion effect, got {}", a.type_name()));
            };
            let mut p = p.clone();
            match f {
                PotField::Amplifier | PotField::Tier => {
                    let n = num(b).ok_or("the amplifier must be a number")?;
                    let n = if f == PotField::Tier { n - 1.0 } else { n };
                    p.amplifier = n.clamp(0.0, 255.0) as u8;
                }
                PotField::Duration => {
                    p.ticks = duration_ticks(b).ok_or("the duration must be a time span")?;
                }
            }
            potion_value(p)
        }
        Op::EffectsOf => match a {
            Value::Player(p) => host.w5c5(Q::Effects(p)),
            _ => Value::None,
        },
        Op::EffectOf | Op::TierOf => {
            let (Some(effect), Value::Player(p)) = (effect_of(a), b) else {
                return Ok(Value::None);
            };
            let Value::List(all) = host.w5c5(Q::Effects(p)) else {
                return Ok(Value::None);
            };
            let found = all
                .into_iter()
                .find(|v| potion(v).is_some_and(|x| x.effect == effect));
            match (op, found) {
                (Op::TierOf, Some(v)) => Value::Num(potion(&v).map_or(0.0, |x| f64::from(x.amplifier) + 1.0)),
                (_, Some(v)) => v,
                (_, None) => Value::None,
            }
        }
        Op::Category => match effect_of(a) {
            Some(e) => named("potion effect type category", category(e)),
            None => Value::None,
        },
        Op::PotionItem(kind) => {
            let id = ["potion", "splash_potion", "lingering_potion"][usize::from(kind.min(2))];
            if let Some(p) = potion(a) {
                let dur = if p.ticks == INFINITE { -1 } else { p.ticks as i64 };
                let snbt = format!(
                    "{{custom_effects:[{{id:\"minecraft:{}\",amplifier:{},duration:{dur},ambient:{}b,show_particles:{}b}}]}}",
                    p.effect,
                    p.amplifier,
                    u8::from(p.ambient),
                    u8::from(p.particles)
                );
                return Ok(item_with(id, "potion_contents", snbt));
            }
            let Some(effect) = effect_of(a) else {
                return Err(format!("'{a}' is not a potion effect type"));
            };
            let snbt = match base_potion(effect) {
                Some(base) => format!("{{potion:\"minecraft:{base}\"}}"),
                None => format!(
                    "{{custom_effects:[{{id:\"minecraft:{effect}\",amplifier:0,duration:{}}}]}}",
                    3600
                ),
            };
            item_with(id, "potion_contents", snbt)
        }
        Op::Skull => {
            let name = match a {
                Value::Player(p) => p.name.clone(),
                Value::Text(t) if !t.trim().is_empty() && t.len() <= 16 => t.trim().to_string(),
                _ => return Ok(Value::None),
            };
            item_with("player_head", "profile", snbt_str(&name))
        }
        Op::Particle(name) => particle_value(new_particle(name)),
        Op::PartGet(f) => match particle_or_named(a) {
            Some(p) => match f {
                PField::Count => Value::Num(f64::from(p.count)),
                PField::Speed => Value::Num(p.speed),
                PField::Offset | PField::Distribution | PField::Velocity => Value::Vector {
                    x: p.offset[0],
                    y: p.offset[1],
                    z: p.offset[2],
                },
            },
            None => Value::None,
        },
        Op::PartWith(f) => {
            let Some(mut p) = particle_or_named(a) else {
                return Err(format!("expected a particle, got {}", a.type_name()));
            };
            match f {
                PField::Count => {
                    p.count = num(b)
                        .ok_or("the particle count must be a number")?
                        .clamp(0.0, 1000.0) as u32;
                }
                PField::Speed => p.speed = num(b).ok_or("the particle speed must be a number")?,
                PField::Offset | PField::Distribution | PField::Velocity => {
                    p.offset = vector(b).ok_or("the particle offset must be a vector")?;
                    if f == PField::Velocity {
                        p.count = 0;
                    } else if p.count == 0 {
                        p.count = 1;
                    }
                }
            }
            particle_value(p)
        }
        Op::QueueNew => {
            let mut items = Vec::new();
            for v in vals {
                match v {
                    Value::List(l) => items.extend(l.iter().filter(|x| !matches!(x, Value::None)).cloned()),
                    Value::None => {}
                    v => items.push(v.clone()),
                }
            }
            Value::Queue(items)
        }
        Op::QueueEnd(start) => match a {
            Value::Queue(q) => {
                let it = if start { q.first() } else { q.last() };
                it.cloned().unwrap_or(Value::None)
            }
            _ => Value::None,
        },
        Op::RawName => match a {
            Value::Item { id, .. } => Value::Text(format!("minecraft:{id}")),
            _ => Value::None,
        },
        Op::EnchLevel(max) => match crate::item_meta::split_spec(&a.to_string()) {
            Some((id, _)) => match max_level(&id) {
                Some(m) => Value::Num(if max { f64::from(m) } else { 1.0 }),
                None => Value::None,
            },
            None => Value::None,
        },
        Op::BarKey => {
            let id = a.to_string();
            match host.w5c5(Q::BarExists(&id)) {
                Value::Bool(true) => Value::Text(id),
                _ => Value::None,
            }
        }
        Op::BarProgress => host.w5c5(Q::BarProgress(&a.to_string())),
        Op::Weather => match a {
            Value::Player(p) => match host.w5c5(Q::Weather(p)) {
                Value::Text(t) => named("weather type", &t),
                v => v,
            },
            _ => Value::None,
        },
        Op::Redstone => match block_state(a, host) {
            Some((name, idx, _)) => Value::Num(redstone_power(&name, idx) as f64),
            None => Value::None,
        },
        Op::Charges(max) => match block_state(a, host) {
            Some((name, idx, _)) if name == "respawn_anchor" => {
                Value::Num(if max { 4.0 } else { idx as f64 })
            }
            _ => Value::None,
        },
        Op::InvRows | Op::InvSlots | Op::InvHolder | Op::InvViewers => {
            let Value::Inventory(inv) = a else {
                return Ok(Value::None);
            };
            match op {
                Op::InvRows => Value::Num(f64::from(inv.rows)),
                Op::InvSlots => Value::Num(inv.size() as f64),
                // Chest inventories made by scripts have no holder.
                Op::InvHolder => Value::None,
                _ => host.w5c5(Q::Viewers(inv.id, inv.viewer.as_deref())),
            }
        }
    })
}

// ---- effects ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EOp {
    /// potion effect, player.
    Apply,
    /// particle, place.
    Draw,
    /// player, weather.
    SetWeather,
    /// player.
    ResetWeather,
    /// player: rain (true) or clear.
    SetWeatherTo(bool),
    /// block, number.
    SetCharges,
    /// variable, value: `set particle count of {_p} to 5`.
    SetPart(PField),
    SetPotion(PotField),
}

#[derive(Debug, Clone, PartialEq)]
pub struct W5Effect {
    pub op: EOp,
    pub args: Vec<Expr>,
}

use EffKind::W5c5 as K;

pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    ("apply %object% to %player%", K(EOp::Apply)),
    ("draw %object% at %object%", K(EOp::Draw)),
    ("play %object% at %object%", K(EOp::Draw)),
    ("show %object% at %object%", K(EOp::Draw)),
    (
        "set [the] (client|custom) weather of %player% to (clear|sun|sunny)",
        K(EOp::SetWeatherTo(false)),
    ),
    (
        "set [the] (client|custom) weather of %player% to (rain|rainy|thunder|storm)",
        K(EOp::SetWeatherTo(true)),
    ),
    (
        "set [the] (client|custom) weather of %player% to %object%",
        K(EOp::SetWeather),
    ),
    (
        "reset [the] [(client|custom)] weather of %player%",
        K(EOp::ResetWeather),
    ),
    ("set [the] charges of %object% to %number%", K(EOp::SetCharges)),
    ("set [the] charge of %object% to %number%", K(EOp::SetCharges)),
    (
        "set [the] particle count of %variable% to %number%",
        K(EOp::SetPart(PField::Count)),
    ),
    (
        "set [the] particle offset of %variable% to %object%",
        K(EOp::SetPart(PField::Offset)),
    ),
    (
        "set [the] particle distribution of %variable% to %object%",
        K(EOp::SetPart(PField::Distribution)),
    ),
    (
        "set [the] particle velocity of %variable% to %object%",
        K(EOp::SetPart(PField::Velocity)),
    ),
    (
        "set [the] particle speed [value] of %variable% to %number%",
        K(EOp::SetPart(PField::Speed)),
    ),
    (
        "set [the] extra value of %variable% to %number%",
        K(EOp::SetPart(PField::Speed)),
    ),
    (
        "set [the] [potion] amplifier of %variable% to %number%",
        K(EOp::SetPotion(PotField::Amplifier)),
    ),
    (
        "set [the] potion (tier|level) of %variable% to %number%",
        K(EOp::SetPotion(PotField::Tier)),
    ),
    (
        "set [the] potion (duration|length) of %variable% to %timespan%",
        K(EOp::SetPotion(PotField::Duration)),
    ),
];

pub fn build(op: EOp, caps: &mut [Option<Capture>]) -> Option<Effect> {
    let mut take = |i: usize| caps.get_mut(i).and_then(Option::take);
    match op {
        EOp::SetPart(_) | EOp::SetPotion(_) => {
            let Some(Capture::Var(v)) = take(0) else {
                return None;
            };
            let Some(Capture::Expr(val)) = take(1) else {
                return None;
            };
            let n = match op {
                EOp::SetPotion(p) => Op::PotionSet(p),
                EOp::SetPart(f) => Op::PartWith(f),
                _ => return None,
            };
            let new = node(n, vec![Expr::Var(v.clone()), val], Ty::Object);
            Some(Effect::Set(v, new))
        }
        _ => {
            let mut args = Vec::new();
            for i in 0..2 {
                match take(i) {
                    Some(Capture::Expr(e)) => args.push(e),
                    Some(Capture::Var(v)) => args.push(Expr::Var(v)),
                    None => {}
                }
            }
            Some(Effect::W5c5(Box::new(W5Effect { op, args })))
        }
    }
}

fn fx_pos(v: &Value, host: &mut dyn Host) -> Option<FxPos> {
    match v {
        Value::Location { x, y, z, world } => Some(FxPos {
            x: *x,
            y: *y,
            z: *z,
            world: world.clone(),
        }),
        Value::Player(p) => host.player_location(p).map(|(x, y, z, w)| FxPos {
            x,
            y,
            z,
            world: Some(w),
        }),
        _ => None,
    }
}

/// Weather words to rain (true) or clear (false).
fn rain_of(v: &Value) -> Option<bool> {
    let s = match v {
        Value::Enum(e) => e.id().to_string(),
        v => v.to_string().trim().to_ascii_lowercase(),
    };
    match s.as_str() {
        "clear" | "sun" | "sunny" => Some(false),
        "rain" | "rainy" | "raining" | "thunder" | "thundering" | "storm" | "downfall" => Some(true),
        _ => None,
    }
}

pub fn run_effect(op: EOp, vals: &[Value], host: &mut dyn Host) -> Result<(), String> {
    let a = vals.first().unwrap_or(&Value::None);
    let b = vals.get(1).unwrap_or(&Value::None);
    match op {
        EOp::Apply => {
            let Some(p) = potion(a) else {
                return Err(format!(
                    "expected a potion effect to apply, got {} ({a})",
                    a.type_name()
                ));
            };
            let Value::Player(who) = b else {
                return Ok(());
            };
            if host.w5c5(Q::AddEffect(who, p)) != Value::Bool(true) {
                host.add_effect(who, &p.effect, p.ticks.min(i32::MAX as u32), p.amplifier);
            }
        }
        EOp::Draw => {
            let Some(p) = particle_or_named(a) else {
                return Err(format!(
                    "expected a particle to draw, got {} ({a})",
                    a.type_name()
                ));
            };
            if crate::skbee_particles_display::needs_data(&p.name) {
                crate::skbee_particles_display::warn_no_data(host, &p.name);
                return Ok(());
            }
            let Some(at) = fx_pos(b, host) else {
                return Ok(());
            };
            host.spawn_particle(&ParticleFx {
                name: p.name,
                at,
                count: p.count,
                spread: 0.0,
                speed: p.speed,
                offset: Some(p.offset),
                force: false,
                data: crate::skbee_particles_display::ParticleData::None,
            });
        }
        EOp::SetWeather => {
            let Value::Player(p) = a else {
                return Ok(());
            };
            let Some(rain) = rain_of(b) else {
                return Err(format!("'{b}' is not a weather (clear or rain)"));
            };
            host.w5c5(Q::SetWeather(p, Some(rain)));
        }
        EOp::SetWeatherTo(rain) => {
            if let Value::Player(p) = a {
                host.w5c5(Q::SetWeather(p, Some(rain)));
            }
        }
        EOp::ResetWeather => {
            if let Value::Player(p) = a {
                host.w5c5(Q::SetWeather(p, None));
            }
        }
        EOp::SetCharges => {
            let Some((name, _, default)) = block_state(a, host) else {
                return Ok(());
            };
            if name != "respawn_anchor" {
                host.log(
                    LogLevel::Warn,
                    &format!("only respawn anchors have charges, not {name}"),
                );
                return Ok(());
            }
            let n = num(b).ok_or("charges must be a number")?.clamp(0.0, 4.0) as i64;
            if let (Some(pos), Ok(state)) = (block_pos(a), u16::try_from(default + n)) {
                host.w5c5(Q::SetBlockState(&pos, state));
            }
        }
        EOp::SetPart(_) | EOp::SetPotion(_) => {}
    }
    Ok(())
}

// ---- parsing ----------------------------------------------------------------------------

fn word(t: Option<&Tok>) -> Option<String> {
    match t {
        Some(Tok::Word(w)) => Some(w.to_ascii_lowercase()),
        _ => None,
    }
}

const STOP: &[&str] = &[
    "of", "for", "without", "with", "tier", "to", "on", "and", "or", "is",
];

impl P<'_> {
    fn c_lit(&self, off: usize, w: &str) -> bool {
        if self.pos + off >= self.toks.len() {
            self.hit_end.set(true);
        }
        matches!(self.toks.get(self.pos + off), Some(t) if t.is_lit(w))
    }

    fn c_any(&self, off: usize, ws: &[&str]) -> bool {
        ws.iter().any(|w| self.c_lit(off, w))
    }

    /// `of <operand>` with the `of` at `self.pos + off`.
    fn c_of(&mut self, off: usize) -> Result<Expr, String> {
        self.pos += off + 1;
        if self.pos >= self.toks.len() {
            self.hit_end.set(true);
        }
        self.primary()
    }

    /// Words from `self.pos` that name a potion effect: (id, words used).
    fn c_effect_words(&self, from: usize) -> Option<(&'static str, usize)> {
        let mut words = Vec::new();
        for t in self.toks.get(from..)?.iter().take(4) {
            match t {
                Tok::Word(w) if !STOP.contains(&w.to_ascii_lowercase().as_str()) => {
                    words.push(w.to_ascii_lowercase());
                }
                _ => break,
            }
        }
        (1..=words.len())
            .rev()
            .find_map(|n| effect_id(&words[..n].join("_")).map(|id| (id, n)))
    }

    /// An effect type: plain words (`jump boost`) or an expression.
    fn c_effect(&mut self) -> Result<Expr, String> {
        if let Some((id, n)) = self.c_effect_words(self.pos) {
            self.pos += n;
            return Ok(lit_text(id));
        }
        match self.peek() {
            Some(Tok::Word(w)) => {
                let w = w.clone();
                if !matches!(self.toks.get(self.pos + 1), Some(t) if t.is_lit("'s") || t.is_lit("of")) {
                    return Err(format!("unknown potion effect '{w}'"));
                }
                self.primary()
            }
            Some(_) => self.primary(),
            None => Err("expected a potion effect type".into()),
        }
    }

    /// After `potion effect of`: `<effect> [[of] tier N] [without particles] [for <timespan>]`.
    fn c_new_potion(&mut self, ambient: bool, infinite: bool) -> Result<Expr, String> {
        let effect = self.c_effect()?;
        let mut tier = Expr::Num(1.0);
        if self.c_lit(0, "of") && self.c_lit(1, "tier") {
            self.pos += 2;
            tier = self.sum()?;
        } else if self.c_lit(0, "tier") {
            self.pos += 1;
            tier = self.sum()?;
        } else if matches!(self.toks.get(self.pos), Some(Tok::Num(_))) {
            tier = self.sum()?;
        }
        let mut hidden = false;
        if self.c_lit(0, "without") && self.c_any(1, &["particles", "particle"]) {
            self.pos += 2;
            hidden = true;
        }
        let mut dur = Expr::Num(-1.0);
        if !infinite && self.c_lit(0, "for") {
            self.pos += 1;
            if self.pos >= self.toks.len() {
                self.hit_end.set(true);
            }
            dur = self.sum()?;
        }
        Ok(node(
            Op::NewPotion {
                ambient,
                infinite,
                hidden,
            },
            vec![effect, tier, dur],
            Ty::Object,
        ))
    }

    /// `<words> particle[s]` starting at the head word (`self.pos - 1`).
    fn c_particle_literal(&self) -> Option<(&'static str, usize)> {
        let start = self.pos.checked_sub(1)?;
        let mut words = Vec::new();
        for (i, t) in self.toks.get(start..)?.iter().take(6).enumerate() {
            let Tok::Word(w) = t else { return None };
            let w = w.to_ascii_lowercase();
            if w == "particle" || w == "particles" {
                if i == 0 {
                    return None;
                }
                return particle_id(&words.join("_")).map(|id| (id, i));
            }
            words.push(w);
        }
        None
    }

    /// `result of [running|executing] function "name" [with arg[ument]s a, b]`.
    fn c_result(&mut self) -> Result<Expr, String> {
        if self.c_any(0, &["running", "executing"]) {
            self.pos += 1;
        }
        if !self.c_lit(0, "function") {
            return Err("only 'result of function \"name\" with arguments ...' is supported".into());
        }
        self.pos += 1;
        let name = match self.peek() {
            Some(Tok::Str(s)) => s.clone(),
            Some(Tok::Word(w)) => w.clone(),
            _ => return Err("expected a function name, like \"double\"".into()),
        };
        self.pos += 1;
        let env = self.env;
        let Some(id) = env
            .functions
            .iter()
            .position(|s| s.name.eq_ignore_ascii_case(&name))
        else {
            return Err(format!("unknown function '{name}'"));
        };
        let mut args = Vec::new();
        if self.c_lit(0, "with") && self.c_any(1, &["arguments", "argument", "args", "arg"]) {
            self.pos += 2;
            loop {
                args.push(self.top()?);
                if !self.eat_sym(",")
                    && !(self.c_lit(0, "and") && {
                        self.pos += 1;
                        true
                    })
                {
                    break;
                }
            }
        }
        let sig = env.functions.get(id).ok_or("unknown function")?;
        let required = sig.params.iter().filter(|p| !p.2).count();
        if args.len() < required || args.len() > sig.params.len() {
            return Err(format!(
                "{}() takes {} argument(s), got {}",
                sig.name,
                sig.params.len(),
                args.len()
            ));
        }
        Ok(Expr::Call(id, args))
    }

    /// Called with the head word read. `Ok(None)`: not one of ours.
    pub(crate) fn w5c5_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let save = self.pos;
        let e = match lw {
            "a" | "an" => {
                let next = word(self.toks.get(self.pos));
                let ours = matches!(
                    next.as_deref(),
                    Some("potion" | "ambient" | "infinite" | "permanent" | "new" | "queue")
                );
                if !ours {
                    if self.pos >= self.toks.len() {
                        self.hit_end.set(true);
                    }
                    return Ok(None);
                }
                self.pos += 1;
                let next = next.unwrap_or_default();
                return match self.w5c5_head(&next)? {
                    Some(e) => Ok(Some(e)),
                    None => {
                        self.pos = save;
                        Ok(None)
                    }
                };
            }
            "new" if self.c_lit(0, "queue") => {
                self.pos += 1;
                self.c_queue()?
            }
            "queue" if !self.c_lit(0, "(") => self.c_queue()?,
            "start" | "end" if self.c_lit(0, "of") => {
                let q = self.c_of(0)?;
                node(Op::QueueEnd(lw == "start"), vec![q], Ty::Object)
            }
            "raw" | "minecraft" | "vanilla" if self.c_any(0, &["name", "names"]) && self.c_lit(1, "of") => {
                let it = self.c_of(1)?;
                node(Op::RawName, vec![it], Ty::Text)
            }
            "max" | "maximum" | "min" | "minimum" | "starting"
                if self.c_any(0, &["enchant", "enchantment"]) && self.c_lit(1, "level") =>
            {
                self.pos += 2;
                if !self.c_lit(0, "of") {
                    return Err("expected 'of <enchantment>'".into());
                }
                self.pos += 1;
                let ench = match crate::item_meta::enchant_at(self.toks, self.pos) {
                    Some((id, _, used)) => {
                        self.pos += used;
                        lit_text(id)
                    }
                    None => self.primary()?,
                };
                node(Op::EnchLevel(lw.starts_with("max")), vec![ench], Ty::Number)
            }
            "max" | "maximum" if self.c_any(0, &["charge", "charges"]) && self.c_lit(1, "of") => {
                let b = self.c_of(1)?;
                node(Op::Charges(true), vec![b], Ty::Number)
            }
            "charge" | "charges" if self.c_lit(0, "of") => {
                let b = self.c_of(0)?;
                node(Op::Charges(false), vec![b], Ty::Number)
            }
            "redstone" if self.c_lit(0, "power") && self.c_lit(1, "of") => {
                let b = self.c_of(1)?;
                node(Op::Redstone, vec![b], Ty::Number)
            }
            "boss" if self.c_lit(0, "bar") && self.c_any(1, &["key", "id"]) && self.c_lit(2, "of") => {
                self.pos += 1;
                self.c_bar(Op::BarKey, 1)?
            }
            "bossbar" if self.c_any(0, &["key", "id"]) && self.c_lit(1, "of") => self.c_bar(Op::BarKey, 1)?,
            "progress" if self.c_lit(0, "of") && self.c_any(1, &["bossbar", "boss"]) => {
                self.c_bar(Op::BarProgress, 0)?
            }
            "client" | "custom" if self.c_lit(0, "weather") && self.c_lit(1, "of") => {
                let p = self.c_of(1)?;
                node(Op::Weather, vec![p], Ty::Object)
            }
            "rows" | "slots" if self.c_lit(0, "of") => {
                let i = self.c_of(0)?;
                let op = if lw == "rows" { Op::InvRows } else { Op::InvSlots };
                node(op, vec![i], Ty::Number)
            }
            "amount" | "number"
                if self.c_lit(0, "of") && self.c_any(1, &["rows", "slots"]) && self.c_lit(2, "of") =>
            {
                let op = if self.c_lit(1, "rows") {
                    Op::InvRows
                } else {
                    Op::InvSlots
                };
                let i = self.c_of(2)?;
                node(op, vec![i], Ty::Number)
            }
            "holder" | "holders" if self.c_lit(0, "of") => {
                let i = self.c_of(0)?;
                node(Op::InvHolder, vec![i], Ty::Object)
            }
            "viewers" if self.c_lit(0, "of") => {
                let i = self.c_of(0)?;
                node(Op::InvViewers, vec![i], Ty::List)
            }
            "particle" => {
                let (f, n) = if self.c_lit(0, "count") {
                    (PField::Count, 1)
                } else if self.c_lit(0, "offset") {
                    (PField::Offset, 1)
                } else if self.c_lit(0, "distribution") {
                    (PField::Distribution, 1)
                } else if self.c_lit(0, "velocity") {
                    (PField::Velocity, 1)
                } else if self.c_lit(0, "speed") {
                    (PField::Speed, if self.c_lit(1, "value") { 2 } else { 1 })
                } else {
                    return Ok(None);
                };
                if !self.c_lit(n, "of") {
                    return Ok(None);
                }
                let p = self.c_of(n)?;
                let ty = if matches!(f, PField::Count | PField::Speed) {
                    Ty::Number
                } else {
                    Ty::Vector
                };
                node(Op::PartGet(f), vec![p], ty)
            }
            "extra" if self.c_lit(0, "value") && self.c_lit(1, "of") => {
                let p = self.c_of(1)?;
                node(Op::PartGet(PField::Speed), vec![p], Ty::Number)
            }
            "potion" => match self.c_potion()? {
                Some(e) => e,
                None => {
                    self.pos = save;
                    return Ok(None);
                }
            },
            "ambient" if self.c_lit(0, "potion") && self.c_lit(1, "effect") && self.c_lit(2, "of") => {
                self.pos += 3;
                self.c_new_potion(true, false)?
            }
            "infinite" | "permanent" => {
                let ambient = self.c_lit(0, "ambient");
                let skip = usize::from(ambient);
                if !(self.c_lit(skip, "potion")
                    && self.c_lit(skip + 1, "effect")
                    && self.c_lit(skip + 2, "of"))
                {
                    return Ok(None);
                }
                self.pos += skip + 3;
                self.c_new_potion(ambient, true)?
            }
            "amplifier" | "amplifiers" if self.c_lit(0, "of") => {
                let p = self.c_of(0)?;
                node(Op::PotionGet(PotField::Amplifier), vec![p], Ty::Number)
            }
            "tier" if self.c_lit(0, "of") => self.c_tier()?,
            "active" if self.c_lit(0, "potion") && self.c_lit(1, "effects") && self.c_lit(2, "of") => {
                let p = self.c_of(2)?;
                node(Op::EffectsOf, vec![p], Ty::List)
            }
            "skull" if self.c_lit(0, "of") => {
                let p = self.c_of(0)?;
                node(Op::Skull, vec![p], Ty::Item)
            }
            "splash" | "lingering" if self.c_lit(0, "potion") => {
                self.pos += 1;
                if self.c_lit(0, "item") {
                    self.pos += 1;
                }
                if !self.c_lit(0, "of") {
                    return Err(format!("expected '{lw} potion of <effect>'"));
                }
                self.pos += 1;
                let e = self.c_effect()?;
                node(
                    Op::PotionItem(if lw == "splash" { 1 } else { 2 }),
                    vec![e],
                    Ty::Item,
                )
            }
            "respawn" | "respawning" if self.c_lit(0, "reason") => {
                if self.env.in_function || self.env.trigger_name != "'on respawn'" {
                    return Err("'respawn reason' only exists in 'on respawn'".into());
                }
                self.pos += 1;
                match crate::types_1::ty_from_name("respawn reason") {
                    Some(Ty::T1(t)) => crate::types_1::parse_as(Expr::Message, t),
                    _ => Expr::Message,
                }
            }
            "on-screen" | "on" => {
                // The lexer may split `on-screen` at the dash.
                let skip = if lw == "on" {
                    if !(self.c_lit(0, "-") && self.c_lit(1, "screen")) {
                        return Ok(None);
                    }
                    2
                } else {
                    0
                };
                if !(self.c_lit(skip, "kick") && self.c_any(skip + 1, &["message", "reason"])) {
                    return Ok(None);
                }
                if self.env.in_function || self.env.trigger_name != "'on kick'" {
                    return Err("'on-screen kick message' only exists in 'on kick'".into());
                }
                self.pos += skip + 2;
                Expr::Message
            }
            "result" | "results"
                if self.c_lit(0, "of")
                    && (self.c_lit(1, "function")
                        || (self.c_any(1, &["running", "executing"]) && self.c_lit(2, "function"))) =>
            {
                self.pos += 1;
                self.c_result()?
            }
            _ => {
                // Cheap pre-check: both forms below need one of these words soon.
                let near = self.toks.get(save..).unwrap_or_default().iter().take(5);
                if !near
                    .clone()
                    .any(|t| t.is_lit("particle") || t.is_lit("particles"))
                    && !near
                        .into_iter()
                        .any(|t| t.is_lit("effect") || t.is_lit("effects"))
                {
                    return Ok(None);
                }
                if let Some((id, used)) = self.c_particle_literal() {
                    self.pos = save - 1 + used + 1;
                    return Ok(Some(node(Op::Particle(id), Vec::new(), Ty::Object)));
                }
                // `speed potion effect of player`, `jump boost effect of player`
                if let Some((id, n)) = self.c_effect_words(save - 1) {
                    let at = save - 1 + n;
                    let skip = usize::from(matches!(self.toks.get(at), Some(t) if t.is_lit("potion")));
                    let eff = self.toks.get(at + skip);
                    let of = self.toks.get(at + skip + 1);
                    if matches!(eff, Some(t) if t.is_lit("effect") || t.is_lit("effects"))
                        && matches!(of, Some(t) if t.is_lit("of"))
                    {
                        self.pos = at + skip + 1;
                        let p = self.c_of(0)?;
                        return Ok(Some(node(Op::EffectOf, vec![lit_text(id), p], Ty::Object)));
                    }
                }
                return Ok(None);
            }
        };
        Ok(Some(e))
    }

    /// `bossbar key of [bossbar] "id"`, `progress of bossbar "id"`: `off` is where the `of` is.
    fn c_bar(&mut self, op: Op, off: usize) -> Result<Expr, String> {
        self.pos += off + 1;
        if self.c_lit(0, "bossbar") {
            self.pos += 1;
        } else if self.c_lit(0, "boss") && self.c_lit(1, "bar") {
            self.pos += 2;
        }
        if self.pos >= self.toks.len() {
            self.hit_end.set(true);
        }
        let id = self.primary()?;
        let ty = if op == Op::BarKey { Ty::Text } else { Ty::Number };
        Ok(node(op, vec![id], ty))
    }

    /// `[a] [new] queue [(of|with) a, b, c]`, after `queue`.
    fn c_queue(&mut self) -> Result<Expr, String> {
        let mut items = Vec::new();
        if self.pos < self.toks.len() && self.c_any(0, &["of", "with"]) {
            self.pos += 1;
            loop {
                if matches!(self.toks.get(self.pos), Some(Tok::Var(v)) if v.ends_with("::*")) {
                    let (src, end) = crate::list_ops::parse_src_at(self.toks, self.pos, self.env, false)?;
                    self.pos = end;
                    items.push(Expr::Items(Box::new(src)));
                } else {
                    items.push(self.sum()?);
                }
                if !self.eat_sym(",")
                    && !(self.toks.get(self.pos).is_some_and(|t| t.is_lit("and")) && {
                        self.pos += 1;
                        true
                    })
                {
                    break;
                }
            }
        }
        Ok(node(Op::QueueNew, items, Ty::T1(crate::types_1::T_QUEUE)))
    }

    /// After `tier`, at `of`: `tier of <effect> (of|for|on) <player>` or `tier of <potion effect>`.
    fn c_tier(&mut self) -> Result<Expr, String> {
        if let Some((id, n)) = self.c_effect_words(self.pos + 1)
            && self
                .toks
                .get(self.pos + 1 + n)
                .is_some_and(|t| t.is_lit("of") || t.is_lit("for") || t.is_lit("on"))
        {
            self.pos += 1 + n;
            let p = self.c_of(0)?;
            return Ok(node(Op::TierOf, vec![lit_text(id), p], Ty::Number));
        }
        let p = self.c_of(0)?;
        Ok(node(Op::PotionGet(PotField::Tier), vec![p], Ty::Number))
    }

    /// After `potion`. `Ok(None)`: not one of ours (the caller resets `pos`).
    fn c_potion(&mut self) -> Result<Option<Expr>, String> {
        let e = if self.c_lit(0, "effect") && self.c_lit(1, "of") {
            self.pos += 2;
            self.c_new_potion(false, false)?
        } else if self.c_lit(0, "effects") && self.c_lit(1, "of") {
            let p = self.c_of(1)?;
            node(Op::EffectsOf, vec![p], Ty::List)
        } else if let Some(n) = [
            &["category", "of"][..],
            &["effect", "category", "of"],
            &["effect", "type", "category", "of"],
        ]
        .iter()
        .find(|ws| ws.iter().enumerate().all(|(i, w)| self.c_lit(i, w)))
        .map(|ws| ws.len())
        {
            let t = self.c_of(n - 1)?;
            node(Op::Category, vec![t], Ty::Object)
        } else if self.c_any(0, &["amplifier", "amplifiers"]) && self.c_lit(1, "of") {
            let p = self.c_of(1)?;
            node(Op::PotionGet(PotField::Amplifier), vec![p], Ty::Number)
        } else if self.c_lit(0, "tier") && self.c_lit(1, "of") {
            self.pos += 1;
            self.c_tier()?
        } else if self.c_lit(0, "level") && self.c_lit(1, "of") {
            let p = self.c_of(1)?;
            node(Op::PotionGet(PotField::Tier), vec![p], Ty::Number)
        } else if self.c_any(0, &["duration", "length"]) && self.c_lit(1, "of") {
            let p = self.c_of(1)?;
            node(Op::PotionGet(PotField::Duration), vec![p], Ty::Timespan)
        } else if self.c_lit(0, "of") || (self.c_lit(0, "item") && self.c_lit(1, "of")) {
            self.pos += if self.c_lit(0, "of") { 1 } else { 2 };
            let e = if matches!(self.toks.get(self.pos), Some(Tok::Var(_))) {
                self.primary()?
            } else {
                self.c_effect()?
            };
            node(Op::PotionItem(0), vec![e], Ty::Item)
        } else {
            return Ok(None);
        };
        Ok(Some(e))
    }

    /// `<particle> with offset <vector>`, `with distribution`, `with velocity`,
    /// `with particle speed <n>`, `with extra value <n>`.
    pub(crate) fn w5c5_post(&mut self, mut e: Expr) -> Result<Expr, String> {
        loop {
            if !matches!(self.toks.get(self.pos), Some(t) if t.is_lit("with")) {
                return Ok(e);
            }
            let is_part = match &e {
                Expr::W5c5(n) => matches!(n.op, Op::Particle(_) | Op::PartWith(_)),
                Expr::Var(_) => true,
                _ => false,
            };
            if !is_part {
                return Ok(e);
            }
            let mut i = self.pos + 1;
            if matches!(self.toks.get(i), Some(t) if t.is_lit("a") || t.is_lit("an")) {
                i += 1;
            }
            let w = word(self.toks.get(i));
            let (f, used) = match w.as_deref() {
                Some("offset") => (PField::Offset, 1),
                Some("distribution") => (PField::Distribution, 1),
                Some("velocity") => (PField::Velocity, 1),
                Some("particle") if matches!(self.toks.get(i + 1), Some(t) if t.is_lit("speed") || t.is_lit("count")) =>
                {
                    let f = if matches!(self.toks.get(i + 1), Some(t) if t.is_lit("count")) {
                        PField::Count
                    } else {
                        PField::Speed
                    };
                    let v = usize::from(matches!(self.toks.get(i + 2), Some(t) if t.is_lit("value")));
                    (f, 2 + v)
                }
                Some("extra") if matches!(self.toks.get(i + 1), Some(t) if t.is_lit("value")) => {
                    (PField::Speed, 2)
                }
                _ => return Ok(e),
            };
            self.pos = i + used;
            if matches!(self.toks.get(self.pos), Some(t) if t.is_lit("of")) {
                self.pos += 1;
            }
            if self.pos >= self.toks.len() {
                self.hit_end.set(true);
            }
            let v = self.primary()?;
            e = node(Op::PartWith(f), vec![e, v], Ty::Object);
        }
    }
}

/// Does this token run start one of our list expressions (`potion effects of`)?
pub fn is_list_expr(toks: &[Tok]) -> bool {
    let lit = |i: usize, w: &str| toks.get(i).is_some_and(|t| t.is_lit(w));
    let skip = usize::from(lit(0, "the")) + usize::from(lit(0, "active") || lit(1, "active"));
    (lit(skip, "potion") && lit(skip + 1, "effects") && lit(skip + 2, "of"))
        || (lit(0, "viewers") && lit(1, "of"))
}
