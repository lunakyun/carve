//! Wave 2, expressions 5: number constants, `numbers from a to b`,
//! `x% of y`, midpoints, stack sizes, moon phase, player list name,
//! metadata and persistent data, `parse error`, `plain <item>`.
//!
//! Constants fold into plain numbers at compile time. Everything else is one
//! boxed [`W5`] expression, so the hot expression enum does not grow.

use crate::compile::{EffKind, Effect};
use crate::expr::{Env, Expr, Ty, parse_sum_at};
use crate::lexer::Tok;
use crate::pattern::Capture;

/// Most numbers `numbers from a to b` may produce.
pub const MAX_NUMBERS: f64 = 1_000_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharKind {
    Upper,
    Lower,
    Digits,
    Spaces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataKind {
    /// Lives in memory until the server stops.
    Metadata,
    /// Saved with the variables.
    Persistent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum W5 {
    /// `numbers from a to b` (false) / `integers from a to b` (true).
    Numbers(Box<Expr>, Box<Expr>, bool),
    /// `10% of x`: percent, value.
    Percent(Box<Expr>, Box<Expr>),
    /// `number of uppercase letters in x`.
    CountChars(CharKind, Box<Expr>),
    /// `midpoint of a and b`.
    Midpoint(Box<Expr>, Box<Expr>),
    /// `middle of loc`: the centre of the block.
    MiddleOf(Box<Expr>),
    /// `max stack size of item`.
    MaxStack(Box<Expr>),
    /// `moon phase of world`.
    MoonPhase(Box<Expr>),
    /// `player list name of p`.
    ListName(Box<Expr>),
    /// `metadata value "k" of p`: key, target.
    Data(DataKind, Box<Expr>, Box<Expr>),
    /// `parse error`.
    ParseError,
    /// `plain <item>`.
    Plain(Box<Expr>),
}

impl W5 {
    pub fn ty(&self) -> Ty {
        match self {
            W5::Numbers(..) => Ty::List,
            W5::Percent(..) | W5::CountChars(..) | W5::MaxStack(_) => Ty::Number,
            W5::Midpoint(..) | W5::MiddleOf(_) => Ty::Location,
            W5::MoonPhase(_) | W5::ListName(_) | W5::ParseError => Ty::Text,
            W5::Data(..) => Ty::Object,
            W5::Plain(_) => Ty::Item,
        }
    }
}

fn w(toks: &[Tok], i: usize) -> Option<String> {
    match toks.get(i) {
        Some(Tok::Word(x)) => Some(x.to_ascii_lowercase()),
        _ => None,
    }
}

fn is(toks: &[Tok], i: usize, lit: &str) -> bool {
    toks.get(i).is_some_and(|t| t.is_lit(lit))
}

/// `loop numbers from 1 to 5` / `loop all integers between ...`.
pub fn is_numbers_loop(toks: &[Tok]) -> bool {
    let first = w(toks, 0);
    match first.as_deref() {
        Some("numbers" | "integers") => true,
        Some("all") => is(toks, 1, "numbers") || is(toks, 1, "integers"),
        _ => false,
    }
}

/// Words that begin a multi-word expression of ours. When the tokens end
/// before the guard words, the pattern matcher must try a longer span.
pub fn may_continue(lw: &str) -> bool {
    matches!(
        lw,
        "moon"
            | "player"
            | "tab"
            | "persistent"
            | "number"
            | "amount"
            | "count"
            | "midpoint"
            | "middle"
            | "center"
            | "centre"
            | "max"
            | "maximum"
            | "min"
            | "minimum"
            | "all"
            | "positive"
            | "negative"
            | "parse"
            | "last"
    )
}

pub(crate) fn boxed(e: W5) -> Expr {
    Expr::W5(Box::new(e))
}

/// A number constant like `max integer value`, from the words after it.
fn limit(max: bool, kind: &str) -> Option<f64> {
    Some(match (max, kind) {
        (true, "int" | "integer") => f64::from(i32::MAX),
        (false, "int" | "integer") => f64::from(i32::MIN),
        (true, "long") => i64::MAX as f64,
        (false, "long") => i64::MIN as f64,
        (true, "double") => f64::MAX,
        (false, "double") => -f64::MAX,
        (true, "float") => f64::from(f32::MAX),
        (false, "float") => -f64::from(f32::MAX),
        _ => return None,
    })
}

/// Words that start one of these expressions. `pos` is the index after the
/// word `lw` (already lowercased). Returns the expression and the index after
/// it, or `None` when the words are not ours.
pub fn parse_word(toks: &[Tok], pos: usize, env: &Env, lw: &str) -> Result<Option<(Expr, usize)>, String> {
    let sub = |at: usize| parse_sum_at(toks, at, env);
    Ok(Some(match lw {
        "pi" => (Expr::Num(std::f64::consts::PI), pos),
        "nan" => (Expr::Num(f64::NAN), pos),
        "infinity" => (Expr::Num(f64::INFINITY), pos),
        "positive" if is(toks, pos, "infinity") => (Expr::Num(f64::INFINITY), pos + 1),
        "negative" if is(toks, pos, "infinity") => (Expr::Num(f64::NEG_INFINITY), pos + 1),
        "max" | "maximum" | "min" | "minimum" => {
            let max = lw.starts_with("max");
            if let Some(kind) = w(toks, pos)
                && let Some(v) = limit(max, &kind)
            {
                let end = if is(toks, pos + 1, "value") {
                    pos + 2
                } else {
                    pos + 1
                };
                return Ok(Some((Expr::Num(v), end)));
            }
            if max && is(toks, pos, "stack") && is(toks, pos + 1, "size") && is(toks, pos + 2, "of") {
                let (e, end) = sub(pos + 3)?;
                (boxed(W5::MaxStack(Box::new(e))), end)
            } else {
                return Ok(None);
            }
        }
        "all" | "numbers" | "integers" => {
            let mut p = pos;
            if lw == "all" {
                if !(is(toks, p, "numbers") || is(toks, p, "integers")) {
                    return Ok(None);
                }
                p += 1;
            }
            let ints = lw == "integers" || (lw == "all" && is(toks, p - 1, "integers"));
            if !(is(toks, p, "from") || is(toks, p, "between")) {
                return Ok(None);
            }
            let (a, p) = sub(p + 1)?;
            if !(is(toks, p, "to") || is(toks, p, "and")) {
                return Err("expected 'numbers from <a> to <b>'".into());
            }
            let (b, p) = sub(p + 1)?;
            (boxed(W5::Numbers(Box::new(a), Box::new(b), ints)), p)
        }
        "number" | "amount" | "count" if is(toks, pos, "of") => {
            let kind = match w(toks, pos + 1).as_deref() {
                Some("uppercase" | "upper") => CharKind::Upper,
                Some("lowercase" | "lower") => CharKind::Lower,
                Some("digits") => CharKind::Digits,
                Some("spaces" | "whitespace") => CharKind::Spaces,
                _ => return Ok(None),
            };
            let mut p = pos + 2;
            if is(toks, p, "letters") || is(toks, p, "characters") {
                p += 1;
            }
            if !(is(toks, p, "in") || is(toks, p, "of")) {
                return Err(format!(
                    "expected '{lw} of {} in <text>'",
                    w(toks, pos + 1).unwrap_or_default()
                ));
            }
            let (t, p) = sub(p + 1)?;
            (boxed(W5::CountChars(kind, Box::new(t))), p)
        }
        "midpoint" | "middle" | "center" | "centre" if is(toks, pos, "of") || is(toks, pos, "between") => {
            let (a, p) = sub(pos + 1)?;
            if is(toks, p, "and") {
                let (b, p) = sub(p + 1)?;
                (boxed(W5::Midpoint(Box::new(a), Box::new(b))), p)
            } else if lw == "midpoint" {
                return Err("expected 'midpoint of <a> and <b>'".into());
            } else {
                (boxed(W5::MiddleOf(Box::new(a))), p)
            }
        }
        "moon" if is(toks, pos, "phase") && is(toks, pos + 1, "of") => {
            let (a, p) = sub(pos + 2)?;
            (boxed(W5::MoonPhase(Box::new(a))), p)
        }
        "player" | "tab" if is(toks, pos, "list") && is(toks, pos + 1, "name") && is(toks, pos + 2, "of") => {
            let (a, p) = sub(pos + 3)?;
            (boxed(W5::ListName(Box::new(a))), p)
        }
        "metadata" | "persistent" => {
            let (kind, at) = if lw == "metadata" {
                (DataKind::Metadata, pos)
            } else if is(toks, pos, "data") {
                (DataKind::Persistent, pos + 1)
            } else {
                return Ok(None);
            };
            if !is(toks, at, "value") {
                return Ok(None);
            }
            let (key, p) = sub(at + 1)?;
            if !is(toks, p, "of") {
                return Err(format!("expected '{lw} value <key> of <player>'"));
            }
            let (target, p) = sub(p + 1)?;
            (boxed(W5::Data(kind, Box::new(key), Box::new(target))), p)
        }
        "parse" if is(toks, pos, "error") => (boxed(W5::ParseError), pos + 1),
        "last" if is(toks, pos, "parse") && is(toks, pos + 1, "error") => (boxed(W5::ParseError), pos + 2),
        "plain" if toks.get(pos).is_some() => {
            let (e, p) = sub(pos)?;
            if !matches!(e.ty(env), Ty::Item | Ty::Object) {
                return Err("'plain' needs an item".into());
            }
            (boxed(W5::Plain(Box::new(e))), p)
        }
        _ => return Ok(None),
    }))
}

/// `<player>'s player list name`: `pos` is the word after `'s`.
pub fn possessive(toks: &[Tok], pos: usize, base: &Expr) -> Option<(Expr, usize)> {
    let head = w(toks, pos)?;
    if matches!(head.as_str(), "player" | "tab") && is(toks, pos + 1, "list") && is(toks, pos + 2, "name") {
        return Some((boxed(W5::ListName(Box::new(base.clone()))), pos + 3));
    }
    None
}

/// `10% of x` / `10 percent of x`: `pos` is at `%` or `percent`.
pub fn percent_follows(toks: &[Tok], pos: usize) -> bool {
    (is(toks, pos, "%") || is(toks, pos, "percent")) && is(toks, pos + 1, "of")
}

pub fn percent(pct: Expr, of: Expr) -> Expr {
    boxed(W5::Percent(Box::new(pct), Box::new(of)))
}

// ---- pure helpers ----

/// Names for moon phase 0 to 7, as in vanilla (0 is a full moon).
pub const MOON_PHASES: [&str; 8] = [
    "full moon",
    "waning gibbous",
    "last quarter",
    "waning crescent",
    "new moon",
    "waxing crescent",
    "first quarter",
    "waxing gibbous",
];

pub fn moon_phase(day_time_ticks: u64) -> &'static str {
    MOON_PHASES[((day_time_ticks / 24000) % 8) as usize]
}

pub fn count_chars(kind: CharKind, s: &str) -> usize {
    s.chars()
        .filter(|c| match kind {
            CharKind::Upper => c.is_uppercase(),
            CharKind::Lower => c.is_lowercase(),
            CharKind::Digits => c.is_ascii_digit(),
            CharKind::Spaces => c.is_whitespace(),
        })
        .count()
}

/// Vanilla stack size for an item id (without `minecraft:`).
pub fn default_stack_size(id: &str) -> u32 {
    const ONE_SUFFIX: [&str; 19] = [
        "_sword",
        "_pickaxe",
        "_axe",
        "_shovel",
        "_hoe",
        "_helmet",
        "_chestplate",
        "_leggings",
        "_boots",
        "_horse_armor",
        "_boat",
        "_raft",
        "_minecart",
        "_bed",
        "_stew",
        "_shulker_box",
        "_disc",
        "_chest_boat",
        "_potion",
    ];
    const ONE: [&str; 27] = [
        "bow",
        "crossbow",
        "trident",
        "mace",
        "shield",
        "elytra",
        "fishing_rod",
        "carrot_on_a_stick",
        "warped_fungus_on_a_stick",
        "shears",
        "flint_and_steel",
        "brush",
        "spyglass",
        "saddle",
        "totem_of_undying",
        "enchanted_book",
        "writable_book",
        "potion",
        "cake",
        "minecart",
        "water_bucket",
        "lava_bucket",
        "milk_bucket",
        "powder_snow_bucket",
        "wolf_armor",
        "bundle",
        "compass",
    ];
    const SIXTEEN: [&str; 7] = [
        "ender_pearl",
        "snowball",
        "egg",
        "bucket",
        "honey_bottle",
        "armor_stand",
        "painting",
    ];
    if SIXTEEN.contains(&id)
        || id.ends_with("_sign")
        || id.ends_with("_hanging_sign")
        || id.ends_with("_banner")
    {
        16
    } else if ONE.contains(&id) || ONE_SUFFIX.iter().any(|s| id.ends_with(s)) {
        1
    } else {
        64
    }
}

/// What a script can attach data to: a player, a world, or an item type.
pub fn target_id(v: &crate::value::Value) -> Option<String> {
    use crate::value::Value;
    match v {
        Value::Player(p) => Some(format!("p:{}", p.uuid)),
        Value::World(w) => Some(format!("w:{w}")),
        Value::Item { id, .. } => Some(format!("i:{id}")),
        _ => None,
    }
}

/// Engine state for this module.
#[derive(Debug, Default)]
pub struct State {
    /// Metadata values: in memory only, gone when the server stops.
    pub metadata: std::collections::HashMap<String, crate::value::Value>,
    /// Message of the last failed `parsed as number`.
    pub last_parse_error: Option<String>,
}

/// Key for a stored value: scripts can attach data to players and worlds.
pub fn data_key(kind: DataKind, target_id: &str, key: &str) -> String {
    let tag = match kind {
        DataKind::Metadata => "md",
        DataKind::Persistent => "pd",
    };
    format!("\u{1}{tag}:{target_id}:{}", key.to_ascii_lowercase())
}

// ---- effects ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum W5Op {
    SetListName,
    ResetListName,
    SetListPriority,
    SetData(DataKind),
    DeleteData(DataKind),
}

#[derive(Debug, Clone, PartialEq)]
pub struct W5Effect {
    pub op: W5Op,
    /// The player or other target.
    pub target: Expr,
    /// The key (data effects) or the new value.
    pub a: Option<Expr>,
    /// The value to store (data effects).
    pub b: Option<Expr>,
}

use EffKind::W5 as K;
use W5Op::{DeleteData, ResetListName, SetData, SetListName, SetListPriority};

pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    ("set (player|tab) list name of %player% to %text%", K(SetListName)),
    ("set %player%'s (player|tab) list name to %text%", K(SetListName)),
    (
        "(reset|clear) (player|tab) list name of %player%",
        K(ResetListName),
    ),
    (
        "(reset|clear) %player%'s (player|tab) list name",
        K(ResetListName),
    ),
    (
        "set (player|tab) list priority of %player% to %number%",
        K(SetListPriority),
    ),
    (
        "set %player%'s (player|tab) list priority to %number%",
        K(SetListPriority),
    ),
    (
        "set metadata value %text% of %object% to %object%",
        K(SetData(DataKind::Metadata)),
    ),
    (
        "set persistent data value %text% of %object% to %object%",
        K(SetData(DataKind::Persistent)),
    ),
    (
        "(delete|clear|reset) metadata value %text% of %object%",
        K(DeleteData(DataKind::Metadata)),
    ),
    (
        "(delete|clear|reset) persistent data value %text% of %object%",
        K(DeleteData(DataKind::Persistent)),
    ),
];

fn cap(caps: &mut [Option<Capture>], i: usize) -> Option<Expr> {
    match caps.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    }
}

/// Build an effect from pattern captures. List name and priority patterns
/// capture the player first, then the value. Data patterns capture key,
/// target, then value.
pub fn build(op: W5Op, caps: &mut [Option<Capture>]) -> Option<Effect> {
    let eff = match op {
        SetListName | SetListPriority => W5Effect {
            op,
            target: cap(caps, 0)?,
            a: cap(caps, 1),
            b: None,
        },
        ResetListName => W5Effect {
            op,
            target: cap(caps, 0)?,
            a: None,
            b: None,
        },
        SetData(_) => W5Effect {
            op,
            a: cap(caps, 0),
            target: cap(caps, 1)?,
            b: cap(caps, 2),
        },
        DeleteData(_) => W5Effect {
            op,
            a: cap(caps, 0),
            target: cap(caps, 1)?,
            b: None,
        },
    };
    Some(Effect::W5(Box::new(eff)))
}
