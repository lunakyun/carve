//! More conditions: dates in the past or future, divisibility, empty values,
//! online state, alive state, worlds, item traits (edible, fuel, ...),
//! `is of type`, and script commands.
//!
//! These are rare checks. They live behind one boxed `Cond::Rare` so the hot
//! condition enum does not grow.

use crate::compile::CondKind;
use crate::expr::Expr;
use crate::host::Host;
use crate::value::{PlayerRef, Value};
use crate::w2_conditions_3::norm_id;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum W2Kind {
    InPast,
    InFuture,
    Divisible,
    Empty,
    Online,
    Alive,
    InWorld,
    Edible,
    Fuel,
    FireResistant,
    Flammable,
    OfType,
    ScriptCommand,
}

/// One rare condition: what to check, its subject, and an optional second value.
#[derive(Debug, Clone, PartialEq)]
pub struct Rare {
    pub kind: W2Kind,
    pub negate: bool,
    pub a: Expr,
    pub b: Option<Expr>,
}

const fn k(kind: W2Kind, negate: bool) -> CondKind {
    CondKind::W2(kind, negate)
}

/// Patterns. Tried before the plain `is` comparison.
pub const CONDITION_PATTERNS: &[(&str, CondKind)] = &[
    ("%date% is in [the] past", k(W2Kind::InPast, false)),
    ("%date% (is not|isn't) in [the] past", k(W2Kind::InPast, true)),
    ("%date% is in [the] future", k(W2Kind::InFuture, false)),
    ("%date% (is not|isn't) in [the] future", k(W2Kind::InFuture, true)),
    (
        "%number% is [evenly] divisible by %number%",
        k(W2Kind::Divisible, false),
    ),
    (
        "%number% (is not|isn't) [evenly] divisible by %number%",
        k(W2Kind::Divisible, true),
    ),
    ("%object% is empty", k(W2Kind::Empty, false)),
    ("%object% (is not|isn't) empty", k(W2Kind::Empty, true)),
    ("%object% is online", k(W2Kind::Online, false)),
    ("%object% (is not|isn't) online", k(W2Kind::Online, true)),
    ("%object% is offline", k(W2Kind::Online, true)),
    ("%object% (is not|isn't) offline", k(W2Kind::Online, false)),
    ("%player% is alive", k(W2Kind::Alive, false)),
    ("%player% (is not|isn't) alive", k(W2Kind::Alive, true)),
    ("%player% is dead", k(W2Kind::Alive, true)),
    ("%player% (is not|isn't) dead", k(W2Kind::Alive, false)),
    ("%object% is in [the] world %object%", k(W2Kind::InWorld, false)),
    (
        "%object% (is not|isn't) in [the] world %object%",
        k(W2Kind::InWorld, true),
    ),
    ("%object% is edible", k(W2Kind::Edible, false)),
    ("%object% (is not|isn't) edible", k(W2Kind::Edible, true)),
    ("%object% is [a] fuel", k(W2Kind::Fuel, false)),
    ("%object% (is not|isn't) [a] fuel", k(W2Kind::Fuel, true)),
    ("%object% is fire resistant", k(W2Kind::FireResistant, false)),
    (
        "%object% (is not|isn't) fire resistant",
        k(W2Kind::FireResistant, true),
    ),
    ("%object% is flammable", k(W2Kind::Flammable, false)),
    ("%object% (is not|isn't) flammable", k(W2Kind::Flammable, true)),
    ("%object% is of type %object%", k(W2Kind::OfType, false)),
    (
        "%object% (is not|isn't) of type %object%",
        k(W2Kind::OfType, true),
    ),
    (
        "[the] command %text% is a [script] command",
        k(W2Kind::ScriptCommand, false),
    ),
    (
        "[the] command %text% (is not|isn't) a [script] command",
        k(W2Kind::ScriptCommand, true),
    ),
];

pub fn build(kind: W2Kind, negate: bool, a: Expr, b: Option<Expr>) -> Rare {
    Rare { kind, negate, a, b }
}

fn online_player(v: &Value, host: &mut dyn Host) -> Option<PlayerRef> {
    match v {
        Value::Player(p) => host.resolve_player(&p.uuid).map(|name| PlayerRef {
            uuid: p.uuid.clone(),
            name,
        }),
        Value::Text(name) => host.find_player(name),
        _ => None,
    }
}

/// Lowercase an item id: no `minecraft:` prefix, spaces become underscores.
fn item_id(v: &Value) -> Option<String> {
    let raw = match v {
        Value::Item { id, .. } => id.as_str(),
        Value::Text(t) => t.as_str(),
        _ => return None,
    };
    let raw = raw.trim().to_lowercase();
    let raw = raw.strip_prefix("minecraft:").unwrap_or(&raw);
    Some(raw.replace(' ', "_"))
}

/// Check a rare condition (before negation). `b` is the evaluated second value.
/// `ScriptCommand` is answered by the engine, which owns the command table.
pub fn check(kind: W2Kind, a: &Value, b: Option<&Value>, host: &mut dyn Host) -> bool {
    match kind {
        W2Kind::InPast | W2Kind::InFuture => {
            let Value::Date(ms) = a else { return false };
            let now = host.wall_clock_millis();
            if kind == W2Kind::InPast {
                *ms < now
            } else {
                *ms > now
            }
        }
        W2Kind::Divisible => {
            let (Some(x), Some(y)) = (a.as_num(), b.and_then(Value::as_num)) else {
                return false;
            };
            if y == 0.0 || !x.is_finite() || !y.is_finite() {
                return false;
            }
            let r = (x % y).abs();
            r < 1e-9 || (y.abs() - r).abs() < 1e-9
        }
        W2Kind::Empty => match a {
            Value::None => true,
            Value::Text(t) => t.is_empty(),
            Value::List(l) => l.is_empty(),
            _ => false,
        },
        W2Kind::Online => online_player(a, host).is_some(),
        W2Kind::Alive => match online_player(a, host) {
            Some(p) => host.health(&p).is_none_or(|h| h > 0.0),
            None => false,
        },
        W2Kind::InWorld => {
            let want = match b {
                Some(Value::World(w)) | Some(Value::Text(w)) => w.as_str(),
                _ => return false,
            };
            let have = match a {
                Value::Location { world: Some(w), .. } | Value::World(w) => Some(w.clone()),
                Value::Player(_) | Value::Text(_) => online_player(a, host)
                    .and_then(|p| host.player_location(&p))
                    .map(|l| l.3),
                _ => None,
            };
            have.is_some_and(|w| w.eq_ignore_ascii_case(want))
        }
        W2Kind::Edible => item_id(a).is_some_and(|id| is_edible(&id)),
        W2Kind::Fuel => item_id(a).is_some_and(|id| is_fuel(&id)),
        W2Kind::FireResistant => item_id(a).is_some_and(|id| is_fire_resistant(&id)),
        W2Kind::Flammable => item_id(a).is_some_and(|id| is_flammable(&id)),
        W2Kind::OfType => {
            let Some(b) = b else { return false };
            match a {
                Value::Player(_) => item_id(b).is_some_and(|w| w == "player"),
                Value::Console => item_id(b).is_some_and(|w| w == "console"),
                Value::Entity(e) => item_id(b).is_some_and(|w| w == norm_id(&e.kind)),
                Value::Location { .. } => crate::w6_chunk3::block_id_at(a, host)
                    .is_some_and(|id| item_id(b).is_some_and(|w| w == id)),
                _ => matches!((item_id(a), item_id(b)), (Some(x), Some(y)) if x == y),
            }
        }
        W2Kind::ScriptCommand => false,
    }
}

/// Items a player eats or drinks (a food or consumable component in the game data).
pub fn is_edible(id: &str) -> bool {
    crate::w6_chunk3_data::FOOD.binary_search(&id).is_ok()
}

/// Items a furnace burns (a fuel component in the game data).
fn is_fuel(id: &str) -> bool {
    crate::w6_chunk3_data::FUEL.binary_search(&id).is_ok()
}

fn is_fire_resistant(id: &str) -> bool {
    crate::w6_chunk3_data::FIRE_RESISTANT.binary_search(&id).is_ok()
}

/// Blocks fire can burn, and the items that place them.
fn is_flammable(id: &str) -> bool {
    crate::w6_chunk3_data::FLAMMABLE_BLOCKS.binary_search(&id).is_ok()
}

use crate::player_flags::PlayerFlag;

const fn f(flag: PlayerFlag, negate: bool) -> CondKind {
    CondKind::Flag(flag, negate)
}

/// Player state checks that ask the host (see `Host::player_flag`).
pub const FLAG_PATTERNS: &[(&str, CondKind)] = &[
    ("%player% is (burning|on fire)", f(PlayerFlag::Burning, false)),
    (
        "%player% (is not|isn't) (burning|on fire)",
        f(PlayerFlag::Burning, true),
    ),
    ("%player% is invisible", f(PlayerFlag::Invisible, false)),
    (
        "%player% (is not|isn't) invisible",
        f(PlayerFlag::Invisible, true),
    ),
    ("%player% is invulnerable", f(PlayerFlag::Invulnerable, false)),
    (
        "%player% (is not|isn't) invulnerable",
        f(PlayerFlag::Invulnerable, true),
    ),
    ("%player% is gliding", f(PlayerFlag::Gliding, false)),
    ("%player% (is not|isn't) gliding", f(PlayerFlag::Gliding, true)),
    ("%player% is frozen", f(PlayerFlag::Frozen, false)),
    ("%player% (is not|isn't) frozen", f(PlayerFlag::Frozen, true)),
    (
        "%player%'s custom name is visible",
        f(PlayerFlag::CustomNameVisible, false),
    ),
    (
        "%player%'s custom name (is not|isn't) visible",
        f(PlayerFlag::CustomNameVisible, true),
    ),
    (
        "%player% has [a] resource pack [loaded]",
        f(PlayerFlag::ResourcePack, false),
    ),
    (
        "%player% (does not|doesn't) have [a] resource pack [loaded]",
        f(PlayerFlag::ResourcePack, true),
    ),
];
