//! Experience and levels: `level of player`, `set level of player to 5`,
//! `add 3 to level of player`, `give player 10 xp`, `level progress of player`.

use crate::host::Host;
use crate::value::PlayerRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XpField {
    /// Whole levels.
    Level,
    /// Total experience points.
    Points,
    /// Progress to the next level, 0 to 1.
    Progress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XpMode {
    Set,
    Add,
    Remove,
}

/// What an xp effect pattern does. `num_first` is true when the pattern's
/// first slot is the number and the second is the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XpOp {
    pub field: XpField,
    pub mode: XpMode,
    pub num_first: bool,
}

const fn op(field: XpField, mode: XpMode, num_first: bool) -> XpOp {
    XpOp { field, mode, num_first }
}

use XpField::{Level, Points, Progress};
use XpMode::{Add, Remove, Set};

/// Effect patterns for experience. Tried before the core ones.
pub const XP_EFFECT_PATTERNS: &[(&str, XpOp)] = &[
    ("set level progress of %player% to %number%", op(Progress, Set, false)),
    ("set %player%'s level progress to %number%", op(Progress, Set, false)),
    ("set (level|levels) of %player% to %number%", op(Level, Set, false)),
    ("set %player%'s (level|levels) to %number%", op(Level, Set, false)),
    (
        "set (xp|exp|experience|total experience) of %player% to %number%",
        op(Points, Set, false),
    ),
    (
        "set %player%'s (xp|exp|experience|total experience) to %number%",
        op(Points, Set, false),
    ),
    ("add %number% to level progress of %player%", op(Progress, Add, true)),
    ("add %number% to %player%'s level progress", op(Progress, Add, true)),
    ("remove %number% from level progress of %player%", op(Progress, Remove, true)),
    ("remove %number% from %player%'s level progress", op(Progress, Remove, true)),
    ("add %number% to (level|levels) of %player%", op(Level, Add, true)),
    ("add %number% to %player%'s (level|levels)", op(Level, Add, true)),
    ("add %number% (level|levels) to %player%", op(Level, Add, true)),
    ("remove %number% from (level|levels) of %player%", op(Level, Remove, true)),
    ("remove %number% from %player%'s (level|levels)", op(Level, Remove, true)),
    ("remove %number% (level|levels) from %player%", op(Level, Remove, true)),
    ("give %number% (level|levels) to %player%", op(Level, Add, true)),
    ("give %player% %number% (level|levels)", op(Level, Add, false)),
    (
        "add %number% to (xp|exp|experience|total experience) of %player%",
        op(Points, Add, true),
    ),
    (
        "add %number% to %player%'s (xp|exp|experience|total experience)",
        op(Points, Add, true),
    ),
    (
        "add %number% (xp|exp|experience) [points] to %player%",
        op(Points, Add, true),
    ),
    (
        "remove %number% from (xp|exp|experience|total experience) of %player%",
        op(Points, Remove, true),
    ),
    (
        "remove %number% from %player%'s (xp|exp|experience|total experience)",
        op(Points, Remove, true),
    ),
    (
        "remove %number% (xp|exp|experience) [points] from %player%",
        op(Points, Remove, true),
    ),
    (
        "give %number% (xp|exp|experience) [points] to %player%",
        op(Points, Add, true),
    ),
    (
        "give %player% %number% (xp|exp|experience) [points]",
        op(Points, Add, false),
    ),
];

/// Apply one xp effect. Add and remove read the current value first so the
/// result never goes below zero (progress is kept between 0 and 1).
pub fn apply(host: &mut dyn Host, p: &PlayerRef, field: XpField, mode: XpMode, n: f64) {
    if !n.is_finite() {
        return;
    }
    let delta = match mode {
        Set => None,
        Add => Some(n),
        Remove => Some(-n),
    };
    let max = if field == Progress { 1.0 } else { f64::from(i32::MAX) };
    let clamp = |v: f64| {
        let v = v.clamp(0.0, max);
        if field == Progress { v } else { v.floor() }
    };
    match delta {
        None => host.xp_set(p, field, clamp(n)),
        Some(d) => match host.xp_get(p, field) {
            Some(cur) => host.xp_set(p, field, clamp(cur + d)),
            None if field == Progress => {}
            None => host.xp_add(p, field, d.trunc()),
        },
    }
}

/// Two-word properties: `level progress`, `total experience`.
pub fn two_word(first: &str, second: &str) -> Option<crate::expr::Prop> {
    use crate::expr::Prop;
    match (first, second) {
        ("level", "progress") => Some(Prop::LevelProgress),
        ("total", "experience") => Some(Prop::Xp),
        _ => None,
    }
}
