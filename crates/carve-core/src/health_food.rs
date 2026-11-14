//! Health and food: `max health`, `food level`, `saturation`, add/remove on
//! them, `damage`, `heal by`, and `feed`. The parts that are not hot live here
//! so the shared files stay small.

use crate::compile::{EffKind, Effect};
use crate::expr::{Expr, Prop};
use crate::host::Host;
use crate::lexer::Tok;
use crate::pattern::Capture;
use crate::value::PlayerRef;

/// A number stored on a player that scripts can read and change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vital {
    Health,
    MaxHealth,
    /// Food level, 0 to 20.
    Food,
    Saturation,
}

impl Vital {
    pub fn of_prop(p: Prop) -> Option<Vital> {
        Some(match p {
            Prop::Health => Vital::Health,
            Prop::MaxHealth => Vital::MaxHealth,
            Prop::Food => Vital::Food,
            Prop::Saturation => Vital::Saturation,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HfOp {
    Set(Vital),
    Add(Vital),
    Remove(Vital),
    /// `heal player by 4`
    HealBy,
    /// `damage player by 4`
    Damage,
    /// `feed player [by 4]`
    Feed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HfEffect {
    pub op: HfOp,
    pub player: Expr,
    pub amount: Option<Expr>,
}

/// Property words starting at `toks[i]`: the property and how many tokens it
/// used. Only the multi-word and new names; plain `health` is handled elsewhere.
pub fn words_from(toks: &[Tok], i: usize) -> Option<(Prop, usize)> {
    let word = |k: usize| match toks.get(k) {
        Some(Tok::Word(w)) => Some(w.to_ascii_lowercase()),
        _ => None,
    };
    let w0 = word(i)?;
    let w1 = word(i + 1);
    Some(match (w0.as_str(), w1.as_deref()) {
        ("max", Some("health")) => (Prop::MaxHealth, 2),
        ("food", Some("level")) => (Prop::Food, 2),
        ("hunger", Some("level")) => (Prop::Food, 2),
        ("hunger", _) => (Prop::Food, 1),
        ("saturation", Some("level")) => (Prop::Saturation, 2),
        ("saturation", _) => (Prop::Saturation, 1),
        _ => return None,
    })
}

use EffKind::Hf;
use HfOp::{Add, Remove, Set};
use Vital::{Food, Health, MaxHealth, Saturation};

/// Extra effect patterns, tried after the main list.
pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    ("set max health of %player% to %number%", Hf(Set(MaxHealth))),
    ("set %player%'s max health to %number%", Hf(Set(MaxHealth))),
    ("add %number% to max health of %player%", Hf(Add(MaxHealth))),
    ("add %number% to %player%'s max health", Hf(Add(MaxHealth))),
    (
        "remove %number% from max health of %player%",
        Hf(Remove(MaxHealth)),
    ),
    (
        "remove %number% from %player%'s max health",
        Hf(Remove(MaxHealth)),
    ),
    ("set %player%'s health to %number%", Hf(Set(Health))),
    ("add %number% to health of %player%", Hf(Add(Health))),
    ("add %number% to %player%'s health", Hf(Add(Health))),
    ("remove %number% from health of %player%", Hf(Remove(Health))),
    ("remove %number% from %player%'s health", Hf(Remove(Health))),
    ("set (food level|hunger) of %player% to %number%", Hf(Set(Food))),
    ("set %player%'s (food level|hunger) to %number%", Hf(Set(Food))),
    ("add %number% to (food level|hunger) of %player%", Hf(Add(Food))),
    ("add %number% to %player%'s (food level|hunger)", Hf(Add(Food))),
    (
        "remove %number% from (food level|hunger) of %player%",
        Hf(Remove(Food)),
    ),
    (
        "remove %number% from %player%'s (food level|hunger)",
        Hf(Remove(Food)),
    ),
    ("set saturation of %player% to %number%", Hf(Set(Saturation))),
    ("set %player%'s saturation to %number%", Hf(Set(Saturation))),
    ("add %number% to saturation of %player%", Hf(Add(Saturation))),
    ("add %number% to %player%'s saturation", Hf(Add(Saturation))),
    (
        "remove %number% from saturation of %player%",
        Hf(Remove(Saturation)),
    ),
    (
        "remove %number% from %player%'s saturation",
        Hf(Remove(Saturation)),
    ),
    ("heal %player% by %number%", Hf(HfOp::HealBy)),
    ("damage %player% by %number%", Hf(HfOp::Damage)),
    ("feed %player% [by %number%]", Hf(HfOp::Feed)),
];

fn cap(caps: &mut [Option<Capture>], i: usize) -> Option<Expr> {
    match caps.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    }
}

/// Build the effect from the pattern captures. `add` and `remove` capture the
/// number first and the player second; everything else is player, number.
pub fn build(op: HfOp, caps: &mut [Option<Capture>]) -> Option<Effect> {
    let (p, n) = match op {
        Add(_) | Remove(_) => (1, 0),
        _ => (0, 1),
    };
    Some(Effect::Hf(Box::new(HfEffect {
        op,
        player: cap(caps, p)?,
        amount: cap(caps, n),
    })))
}

fn clamp(v: Vital, x: f64, max_health: f64) -> f64 {
    let x = if x.is_finite() { x } else { 0.0 };
    match v {
        Health => x.clamp(0.0, max_health),
        MaxHealth => x.clamp(1.0, 1024.0),
        Food | Saturation => x.clamp(0.0, 20.0),
    }
}

/// Write a vital with limits applied. Health at zero kills.
fn store(host: &mut dyn Host, p: &PlayerRef, v: Vital, x: f64) {
    let max = host.vital(p, MaxHealth).unwrap_or(20.0);
    let x = clamp(v, x, max);
    if v == Health && x <= 0.0 {
        host.kill(p);
        return;
    }
    host.set_vital(p, v, x);
    // A lower maximum pulls current health down with it.
    if v == MaxHealth && host.vital(p, Health).is_some_and(|h| h > x) {
        host.set_vital(p, Health, x);
    }
}

/// Run one of these effects on an online player.
pub fn run(eff: &HfEffect, amount: f64, p: &PlayerRef, host: &mut dyn Host) {
    match eff.op {
        Set(v) => store(host, p, v, amount),
        Add(v) | Remove(v) => {
            let Some(cur) = host.vital(p, v) else { return };
            let d = if matches!(eff.op, Add(_)) { amount } else { -amount };
            store(host, p, v, cur + d);
        }
        HfOp::HealBy => {
            if let Some(cur) = host.vital(p, Health) {
                store(host, p, Health, cur + amount.max(0.0));
            }
        }
        HfOp::Damage => host.damage(p, amount.max(0.0)),
        HfOp::Feed => {
            if eff.amount.is_some() {
                let cur = host.vital(p, Food).unwrap_or(0.0);
                store(host, p, Food, cur + amount.max(0.0));
            } else {
                store(host, p, Food, 20.0);
                store(host, p, Saturation, 20.0);
            }
        }
    }
}
