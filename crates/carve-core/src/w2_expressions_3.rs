//! Player and misc expressions: `exhaustion of player`, `fall distance`,
//! `burn duration`, `freeze time`, `gliding state`, `glowing state`, `gravity`,
//! `hotbar slot`, `facing`, `head location`, `hidden players`,
//! `first empty slot`, `free memory`, `infinity`, `x hashed with MD5`,
//! `player from uuid`, `highest solid block at`, `index of value`, `except`,
//! `hex code of`, `id of`, `exact item of` and `x formatted`.
//!
//! All of these are one `Expr::Ext(ExtFn, args)` so the hot expression enum
//! does not grow. Evaluation lives in `engine/ext_eval.rs`.

use crate::compile::EffKind;
use crate::expr::{Expr, Ty};
use crate::hashing::{HashAlg, digest_hex};
use crate::host::Host;
use crate::lexer::Tok;
use crate::pattern::Capture;
use crate::value::{PlayerRef, Value};

/// A number or yes/no state stored on a player that the host can read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PStat {
    Exhaustion,
    FallDistance,
    /// Fire ticks left (a timespan in scripts).
    FireTicks,
    /// Freeze ticks (a timespan in scripts).
    FreezeTicks,
    Gliding,
    Glowing,
    Gravity,
    /// Selected hotbar slot, 0 to 8.
    HotbarSlot,
    Yaw,
    Pitch,
}

impl PStat {
    fn is_bool(self) -> bool {
        matches!(self, PStat::Gliding | PStat::Glowing | PStat::Gravity)
    }

    fn is_ticks(self) -> bool {
        matches!(self, PStat::FireTicks | PStat::FreezeTicks)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemKind {
    Free,
    Max,
    Total,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexSel {
    First,
    Last,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtFn {
    Stat(PStat),
    Facing,
    HeadLocation,
    HiddenPlayers,
    FirstEmptySlot,
    Memory(MemKind),
    /// `infinity` (false) or `negative infinity` (true).
    Infinity(bool),
    Hash(HashAlg),
    /// `<timespan> formatted`.
    Clock,
    PlayerFromUuid,
    /// `entity from uuid`, `entity(uuid)`: a player or any live entity.
    EntityFromUuid,
    /// `highest solid block at <location>`.
    Highest,
    IndexOf(IndexSel),
    Except,
    ExactItem,
    ItemId,
    HexCode,
}

pub fn ty(f: ExtFn) -> Ty {
    match f {
        ExtFn::Stat(s) if s.is_bool() => Ty::Boolean,
        ExtFn::Stat(s) if s.is_ticks() => Ty::Timespan,
        ExtFn::Stat(_) | ExtFn::FirstEmptySlot | ExtFn::Memory(_) | ExtFn::Infinity(_) => Ty::Number,
        ExtFn::Facing | ExtFn::Hash(_) | ExtFn::Clock | ExtFn::ItemId | ExtFn::HexCode => Ty::Text,
        ExtFn::HeadLocation | ExtFn::Highest => Ty::Location,
        ExtFn::HiddenPlayers | ExtFn::Except | ExtFn::IndexOf(IndexSel::All) => Ty::List,
        ExtFn::IndexOf(_) => Ty::Object,
        ExtFn::PlayerFromUuid => Ty::Player,
        ExtFn::EntityFromUuid => Ty::Object,
        ExtFn::ExactItem => Ty::Item,
    }
}

/// Does the first argument have to be an online player?
pub fn takes_player(f: ExtFn) -> bool {
    matches!(
        f,
        ExtFn::Stat(_) | ExtFn::Facing | ExtFn::HeadLocation | ExtFn::HiddenPlayers | ExtFn::FirstEmptySlot
    )
}

// ---- parsing helpers (the parser in expr.rs calls these) ----

/// `<words> of <player>` names, longest first.
const OF_NAMES: &[(&[&str], ExtFn)] = &[
    (&["burning", "duration"], ExtFn::Stat(PStat::FireTicks)),
    (&["burning", "time"], ExtFn::Stat(PStat::FireTicks)),
    (&["burn", "duration"], ExtFn::Stat(PStat::FireTicks)),
    (&["burn", "time"], ExtFn::Stat(PStat::FireTicks)),
    (&["fire", "duration"], ExtFn::Stat(PStat::FireTicks)),
    (&["freeze", "duration"], ExtFn::Stat(PStat::FreezeTicks)),
    (&["freeze", "time"], ExtFn::Stat(PStat::FreezeTicks)),
    (&["fall", "distance"], ExtFn::Stat(PStat::FallDistance)),
    (&["gliding", "state"], ExtFn::Stat(PStat::Gliding)),
    (&["glide", "state"], ExtFn::Stat(PStat::Gliding)),
    (&["glowing", "state"], ExtFn::Stat(PStat::Glowing)),
    (&["gravity", "state"], ExtFn::Stat(PStat::Gravity)),
    (&["hotbar", "slot"], ExtFn::Stat(PStat::HotbarSlot)),
    (&["head", "location"], ExtFn::HeadLocation),
    (&["eye", "location"], ExtFn::HeadLocation),
    (&["hidden", "players"], ExtFn::HiddenPlayers),
    (&["exhaustion"], ExtFn::Stat(PStat::Exhaustion)),
    (&["glowing"], ExtFn::Stat(PStat::Glowing)),
    (&["gravity"], ExtFn::Stat(PStat::Gravity)),
    (&["facing"], ExtFn::Facing),
];

/// Property words at `toks[i..]` (case-insensitive) and how many tokens they use.
pub fn of_head(toks: &[Tok], i: usize) -> Option<(ExtFn, usize)> {
    let first = match toks.get(i) {
        Some(Tok::Word(w)) => w.to_ascii_lowercase(),
        _ => return None,
    };
    let second = match toks.get(i + 1) {
        Some(Tok::Word(w)) => Some(w.to_ascii_lowercase()),
        _ => None,
    };
    OF_NAMES.iter().find_map(|(words, f)| match words {
        [a] if *a == first => Some((*f, 1)),
        [a, b] if *a == first && second.as_deref() == Some(*b) => Some((*f, 2)),
        _ => None,
    })
}

/// Hash algorithm words at `toks[i..]`: `MD5`, `SHA-256`, `SHA 512`.
pub fn alg_at(toks: &[Tok], i: usize) -> Option<(HashAlg, usize)> {
    let Some(Tok::Word(w)) = toks.get(i) else {
        return None;
    };
    if let Some(a) = crate::hashing::alg_by_name(w) {
        return Some((a, 1));
    }
    if w.eq_ignore_ascii_case("sha") {
        let mut j = i + 1;
        if matches!(toks.get(j), Some(t) if t.is_lit("-")) {
            j += 1;
        }
        if let Some(Tok::Num(n)) = toks.get(j) {
            let a = crate::hashing::alg_by_name(&format!("sha{}", *n as u32))?;
            return Some((a, j + 1 - i));
        }
    }
    None
}

// ---- evaluation ----

fn player_of(v: &Value) -> Option<&PlayerRef> {
    match v {
        Value::Player(p) => Some(p),
        _ => None,
    }
}

fn stat_value(s: PStat, x: f64) -> Value {
    if s.is_bool() {
        Value::Bool(x != 0.0)
    } else if s.is_ticks() {
        Value::Timespan {
            millis: (x * 50.0).round() as i64,
        }
    } else {
        Value::Num(x)
    }
}

/// Compass word for a yaw and pitch (Minecraft yaw: 0 south, 90 west).
pub fn facing_word(yaw: f64, pitch: f64) -> &'static str {
    if pitch <= -60.0 {
        return "up";
    }
    if pitch >= 60.0 {
        return "down";
    }
    let y = yaw.rem_euclid(360.0);
    match ((y + 45.0) / 90.0) as u32 % 4 {
        0 => "south",
        1 => "west",
        2 => "north",
        _ => "east",
    }
}

/// `1:05:03`, `5:03`, or `2:01:05:03` with days.
pub fn clock_text(millis: i64) -> String {
    let neg = millis < 0;
    let total = millis.unsigned_abs() / 1000;
    let (d, h, m, s) = (total / 86_400, total / 3600 % 24, total / 60 % 60, total % 60);
    let body = if d > 0 {
        format!("{d}:{h:02}:{m:02}:{s:02}")
    } else if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    };
    if neg { format!("-{body}") } else { body }
}

fn text_of(v: &Value) -> String {
    match v {
        Value::Text(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Evaluate every function except `IndexOf` (the engine does that one, since
/// it needs list keys). Arguments are already evaluated; a player argument
/// of a player function is already resolved to an online player.
pub fn apply(f: ExtFn, a: &[Value], host: &mut dyn Host) -> Value {
    let arg = a.first().unwrap_or(&Value::None);
    match f {
        ExtFn::Stat(s) => match player_of(arg).and_then(|p| host.pstat(p, s)) {
            Some(x) => stat_value(s, x),
            None => Value::None,
        },
        ExtFn::Facing => match player_of(arg) {
            Some(p) => {
                let yaw = host.pstat(p, PStat::Yaw).unwrap_or(0.0);
                let pitch = host.pstat(p, PStat::Pitch).unwrap_or(0.0);
                Value::Text(facing_word(yaw, pitch).to_string())
            }
            None => Value::None,
        },
        ExtFn::HeadLocation => match player_of(arg).and_then(|p| host.eye_location(p)) {
            Some((x, y, z, w)) => Value::Location {
                x,
                y,
                z,
                world: Some(w),
            },
            None => Value::None,
        },
        ExtFn::HiddenPlayers => match player_of(arg) {
            Some(p) => {
                let others = host.online_players();
                Value::List(
                    others
                        .into_iter()
                        .filter(|o| o.uuid != p.uuid && !host.can_see(p, o))
                        .map(Value::Player)
                        .collect(),
                )
            }
            None => Value::None,
        },
        ExtFn::FirstEmptySlot => match player_of(arg).and_then(|p| host.first_empty_slot(p)) {
            Some(n) => Value::Num(f64::from(n)),
            None => Value::None,
        },
        ExtFn::Memory(k) => match host.system_memory() {
            Some((total, used)) => {
                let bytes = match k {
                    MemKind::Free => total.saturating_sub(used),
                    MemKind::Max | MemKind::Total => total,
                };
                Value::Num((bytes / 1_048_576) as f64)
            }
            None => Value::None,
        },
        ExtFn::Infinity(neg) => Value::Num(if neg { f64::NEG_INFINITY } else { f64::INFINITY }),
        ExtFn::Hash(alg) => match arg {
            Value::None => Value::None,
            v => Value::Text(digest_hex(alg, text_of(v).as_bytes())),
        },
        ExtFn::Clock => match arg {
            Value::Timespan { millis } => Value::Text(clock_text(*millis)),
            _ => Value::None,
        },
        ExtFn::PlayerFromUuid => {
            let u = text_of(arg);
            let u = u.trim();
            match host.resolve_player(u) {
                Some(name) => Value::Player(PlayerRef {
                    uuid: u.to_string(),
                    name,
                }),
                None => Value::None,
            }
        }
        ExtFn::EntityFromUuid => crate::w6_chunk7::entity_from_uuid(arg, host),
        ExtFn::Highest => {
            let (x, z, world) = match arg {
                Value::Location { x, z, world, .. } => (*x, *z, world.clone()),
                Value::Player(p) => match host.player_location(p) {
                    Some((x, _, z, w)) => (x, z, Some(w)),
                    None => return Value::None,
                },
                _ => return Value::None,
            };
            let (bx, bz) = (x.floor(), z.floor());
            match host.top_block_y(world.as_deref(), bx as i32, bz as i32) {
                Some(y) => Value::Location {
                    x: bx,
                    y: f64::from(y),
                    z: bz,
                    world,
                },
                None => Value::None,
            }
        }
        ExtFn::Except => {
            let keep = |v: &Value| match a.get(1) {
                Some(Value::List(ex)) => !ex.iter().any(|e| e.loose_eq(v)),
                Some(Value::None) | None => true,
                Some(one) => !one.loose_eq(v),
            };
            match arg {
                Value::List(items) => Value::List(items.iter().filter(|v| keep(v)).cloned().collect()),
                Value::None => Value::List(Vec::new()),
                one => Value::List(if keep(one) { vec![one.clone()] } else { Vec::new() }),
            }
        }
        ExtFn::ExactItem => match arg {
            Value::Item { .. } => arg.clone(),
            _ => Value::None,
        },
        ExtFn::ItemId => match arg {
            Value::Item { id, .. } => Value::Text(format!("minecraft:{id}")),
            _ => Value::None,
        },
        ExtFn::HexCode => match crate::colors::name_rgb(&text_of(arg)) {
            Some((r, g, b)) => Value::Text(format!("{r:02X}{g:02X}{b:02X}")),
            None => Value::None,
        },
        ExtFn::IndexOf(_) => Value::None,
    }
}

/// A list key as a value: a number when it looks like one.
pub fn key_value(k: &str) -> Value {
    match k.parse::<f64>() {
        Ok(n) => Value::Num(n),
        Err(_) => Value::Text(k.to_string()),
    }
}

// ---- effects: set / add / remove on the stats above ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Set,
    Add,
    Remove,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatEffect {
    pub stat: PStat,
    pub mode: Mode,
    pub player: Expr,
    pub value: Expr,
}

macro_rules! set_of {
    ($n:literal, $t:literal, $s:expr) => {
        (
            concat!("set ", $n, " of %player% to %", $t, "%"),
            EffKind::W2c($s, Mode::Set),
        )
    };
}
macro_rules! set_poss {
    ($n:literal, $t:literal, $s:expr) => {
        (
            concat!("set %player%'s ", $n, " to %", $t, "%"),
            EffKind::W2c($s, Mode::Set),
        )
    };
}
macro_rules! add_of {
    ($n:literal, $t:literal, $s:expr) => {
        (
            concat!("add %", $t, "% to ", $n, " of %player%"),
            EffKind::W2c($s, Mode::Add),
        )
    };
}
macro_rules! add_poss {
    ($n:literal, $t:literal, $s:expr) => {
        (
            concat!("add %", $t, "% to %player%'s ", $n),
            EffKind::W2c($s, Mode::Add),
        )
    };
}
macro_rules! rem_of {
    ($n:literal, $t:literal, $s:expr) => {
        (
            concat!("remove %", $t, "% from ", $n, " of %player%"),
            EffKind::W2c($s, Mode::Remove),
        )
    };
}
macro_rules! rem_poss {
    ($n:literal, $t:literal, $s:expr) => {
        (
            concat!("remove %", $t, "% from %player%'s ", $n),
            EffKind::W2c($s, Mode::Remove),
        )
    };
}

use PStat::{Exhaustion, FallDistance, FireTicks, FreezeTicks, Glowing, Gravity};

/// Effect patterns, tried after the main list.
pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    set_of!("exhaustion", "number", Exhaustion),
    set_poss!("exhaustion", "number", Exhaustion),
    add_of!("exhaustion", "number", Exhaustion),
    add_poss!("exhaustion", "number", Exhaustion),
    rem_of!("exhaustion", "number", Exhaustion),
    rem_poss!("exhaustion", "number", Exhaustion),
    set_of!("fall distance", "number", FallDistance),
    set_poss!("fall distance", "number", FallDistance),
    add_of!("fall distance", "number", FallDistance),
    add_poss!("fall distance", "number", FallDistance),
    rem_of!("fall distance", "number", FallDistance),
    rem_poss!("fall distance", "number", FallDistance),
    set_of!("(burn|burning|fire) (duration|time)", "timespan", FireTicks),
    set_poss!("(burn|burning|fire) (duration|time)", "timespan", FireTicks),
    add_of!("(burn|burning|fire) (duration|time)", "timespan", FireTicks),
    add_poss!("(burn|burning|fire) (duration|time)", "timespan", FireTicks),
    rem_of!("(burn|burning|fire) (duration|time)", "timespan", FireTicks),
    rem_poss!("(burn|burning|fire) (duration|time)", "timespan", FireTicks),
    set_of!("freeze (duration|time)", "timespan", FreezeTicks),
    set_poss!("freeze (duration|time)", "timespan", FreezeTicks),
    add_of!("freeze (duration|time)", "timespan", FreezeTicks),
    add_poss!("freeze (duration|time)", "timespan", FreezeTicks),
    rem_of!("freeze (duration|time)", "timespan", FreezeTicks),
    rem_poss!("freeze (duration|time)", "timespan", FreezeTicks),
    set_of!("glowing [state]", "boolean", Glowing),
    set_poss!("glowing [state]", "boolean", Glowing),
    set_of!("gravity [state]", "boolean", Gravity),
    set_poss!("gravity [state]", "boolean", Gravity),
];

/// Build the effect from pattern captures. `set` captures player then value;
/// `add` and `remove` capture value then player.
pub fn build(stat: PStat, mode: Mode, caps: &mut [Option<Capture>]) -> Option<StatEffect> {
    let take = |caps: &mut [Option<Capture>], i: usize| match caps.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    };
    let (pi, vi) = if mode == Mode::Set { (0, 1) } else { (1, 0) };
    Some(StatEffect {
        stat,
        mode,
        player: take(caps, pi)?,
        value: take(caps, vi)?,
    })
}

/// Apply one effect to an online player.
pub fn run(eff: &StatEffect, v: &Value, p: &PlayerRef, host: &mut dyn Host) {
    let s = eff.stat;
    let amount = match v {
        Value::Num(n) => *n,
        Value::Timespan { millis } => *millis as f64 / 50.0,
        Value::Bool(b) => f64::from(u8::from(*b)),
        _ => return,
    };
    if !amount.is_finite() {
        return;
    }
    let new = match eff.mode {
        Mode::Set => amount,
        Mode::Add => host.pstat(p, s).unwrap_or(0.0) + amount,
        Mode::Remove => host.pstat(p, s).unwrap_or(0.0) - amount,
    };
    let new = match s {
        PStat::Exhaustion => new.clamp(0.0, 40.0),
        PStat::FallDistance => new.max(0.0),
        PStat::FireTicks | PStat::FreezeTicks => new.max(0.0).round(),
        PStat::Gliding | PStat::Glowing | PStat::Gravity => f64::from(u8::from(new != 0.0)),
        _ => return,
    };
    host.set_pstat(p, s, new);
}
