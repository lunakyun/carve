//! Wave 2 expressions, part 2: character codes, `characters between`,
//! `difference between`, directions (`3 blocks above loc`), chunk numbers,
//! compass target, `death message`, `damage cause` and `chat recipients`.
//!
//! All of them live behind one boxed `Expr::W2` variant so the hot `Expr`
//! enum does not grow. The pure helpers are here; running them is in
//! `engine/w2_eval.rs`.

use crate::compile::{EffKind, Effect};
use crate::expr::{Expr, Ty};
use crate::pattern::Capture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum W2Op {
    /// `codepoint of "a"`: args: text.
    Codepoint,
    /// `character from codepoint 65`: args: number.
    FromCodepoint,
    /// `characters between "a" and "e"`: args: from, to.
    CharsBetween { alnum: bool },
    /// `difference between a and b`.
    Difference,
    /// `3 blocks above loc`: args: amount, location. The unit step is x, y, z.
    Offset(i8, i8, i8),
    /// `chunk x of loc`.
    ChunkX,
    /// `chunk z of loc`.
    ChunkZ,
    /// `compass target of player`.
    CompassTarget,
    /// `death message` in `on death`.
    DeathMessage,
    /// `damage cause` in `on damage`.
    DamageCause,
    /// `chat recipients` in `on chat`.
    ChatRecipients,
}

#[derive(Debug, Clone, PartialEq)]
pub struct W2 {
    pub op: W2Op,
    pub args: Vec<Expr>,
}

impl W2Op {
    /// Static type, used for checks at load time.
    pub fn ty(self, args: &[Expr], env: &crate::expr::Env) -> Ty {
        match self {
            W2Op::Codepoint | W2Op::ChunkX | W2Op::ChunkZ => Ty::Number,
            W2Op::FromCodepoint | W2Op::DeathMessage | W2Op::DamageCause => Ty::Text,
            W2Op::CharsBetween { .. } | W2Op::ChatRecipients => Ty::List,
            W2Op::Offset(..) | W2Op::CompassTarget => Ty::Location,
            W2Op::Difference => match (args.first().map(|a| a.ty(env)), args.get(1).map(|a| a.ty(env))) {
                (Some(Ty::Number), Some(Ty::Number)) => Ty::Number,
                (Some(Ty::Date | Ty::Timespan), Some(Ty::Date | Ty::Timespan)) => Ty::Timespan,
                _ => Ty::Object,
            },
        }
    }
}

/// Do these tokens start a list made by this module (`characters between ...`,
/// `chat recipients`)? Used so `loop`, `set {x::*} to` and friends take them.
pub fn starts_list(toks: &[crate::lexer::Tok]) -> bool {
    let w = |i: usize| match toks.get(i) {
        Some(crate::lexer::Tok::Word(w)) => Some(w.to_ascii_lowercase()),
        _ => None,
    };
    let mut i = usize::from(w(0).as_deref() == Some("all"));
    if w(i).as_deref() == Some("alphanumeric") {
        i += 1;
    }
    (w(i).as_deref() == Some("characters") && w(i + 1).as_deref() == Some("between"))
        || (w(0).as_deref() == Some("chat") && w(1).as_deref() == Some("recipients"))
}

/// Unicode code point of the first character.
pub fn codepoint(s: &str) -> Option<f64> {
    s.chars().next().map(|c| f64::from(u32::from(c)))
}

/// The character with this code point; `None` for bad numbers and surrogates.
pub fn from_codepoint(n: f64) -> Option<String> {
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 || n > f64::from(u32::MAX) {
        return None;
    }
    char::from_u32(n as u32).map(String::from)
}

/// Every character from `a` to `b` inclusive, counting down when `a > b`.
/// Only the first character of each end counts. Capped to keep scripts safe.
pub fn chars_between(a: &str, b: &str, alnum: bool) -> Vec<String> {
    const CAP: u32 = 65_536;
    let (Some(x), Some(y)) = (a.chars().next(), b.chars().next()) else {
        return Vec::new();
    };
    let (x, y) = (u32::from(x), u32::from(y));
    let keep = |c: &char| !alnum || c.is_alphanumeric();
    if x <= y {
        (x..=y.min(x.saturating_add(CAP)))
            .filter_map(char::from_u32)
            .filter(keep)
            .map(String::from)
            .collect()
    } else {
        (y.max(x.saturating_sub(CAP))..=x)
            .rev()
            .filter_map(char::from_u32)
            .filter(keep)
            .map(String::from)
            .collect()
    }
}

/// Chunk number of a block coordinate (16 blocks per chunk, rounds down).
pub fn chunk_of(coord: f64) -> f64 {
    (coord / 16.0).floor()
}

/// Direction words: `above`, `below`, `north`, `south`, `east`, `west`.
/// North is negative z, east is positive x, like Minecraft.
pub fn direction(word: &str) -> Option<(i8, i8, i8)> {
    Some(match word {
        "above" | "up" => (0, 1, 0),
        "below" | "down" | "beneath" | "under" => (0, -1, 0),
        "north" => (0, 0, -1),
        "south" => (0, 0, 1),
        "east" => (1, 0, 0),
        "west" => (-1, 0, 0),
        _ => return None,
    })
}

/// Plain name of a damage type: `PlayerAttack` becomes `player_attack`.
pub fn snake_case(debug_name: &str) -> String {
    let mut out = String::with_capacity(debug_name.len() + 4);
    for (i, c) in debug_name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Effects of this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum W2Eff {
    /// `set compass target of %player% to %location%`: player, location.
    SetCompass,
    /// `reset compass target of %player%`: points the compass at the world origin.
    ResetCompass,
}

pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    ("set compass target of %player% to %location%", EffKind::W2(W2Eff::SetCompass)),
    ("set %player%'s compass target to %location%", EffKind::W2(W2Eff::SetCompass)),
    ("reset compass target of %player%", EffKind::W2(W2Eff::ResetCompass)),
    ("reset %player%'s compass target", EffKind::W2(W2Eff::ResetCompass)),
];

fn cap(caps: &mut [Option<Capture>], i: usize) -> Option<Expr> {
    match caps.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    }
}

pub fn build(kind: W2Eff, caps: &mut [Option<Capture>]) -> Option<Effect> {
    Some(Effect::W2(
        kind,
        Box::new((cap(caps, 0)?, cap(caps, 1).unwrap_or(Expr::Num(0.0)))),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codepoints() {
        assert_eq!(codepoint("A"), Some(65.0));
        assert_eq!(codepoint(""), None);
        assert_eq!(from_codepoint(97.0).as_deref(), Some("a"));
        assert_eq!(from_codepoint(-1.0), None);
        assert_eq!(from_codepoint(55296.0), None);
    }

    #[test]
    fn between() {
        assert_eq!(chars_between("a", "c", false), ["a", "b", "c"]);
        assert_eq!(chars_between("c", "a", false), ["c", "b", "a"]);
        assert_eq!(chars_between("8", "B", true), ["8", "9", "A", "B"]);
    }

    #[test]
    fn chunks_round_down() {
        assert_eq!(chunk_of(15.9), 0.0);
        assert_eq!(chunk_of(16.0), 1.0);
        assert_eq!(chunk_of(-0.5), -1.0);
    }

    #[test]
    fn snake() {
        assert_eq!(snake_case("PlayerAttack"), "player_attack");
        assert_eq!(snake_case("Fall"), "fall");
    }
}
