//! Wave 2 effects, part 1: `connect`, `force respawn`, `hide`/`reveal`
//! players, `equip`, and `copy ... into`.

use crate::compile::{EffKind, Effect};
use crate::expr::{Expr, VarRef};
use crate::pattern::Capture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityOp {
    /// `connect %player% to server %text% [on port %number%]`
    Connect,
    /// `force %player% to respawn`
    Respawn,
    /// `hide %player% from %player%`
    Hide,
    /// `reveal %player% to %player%`
    Reveal,
    /// `equip %player% with %item%`
    Equip,
    /// `copy %variable% into %variable%`
    Copy,
}

/// One effect of this module. Boxed inside `Effect` so the hot enum stays small.
#[derive(Debug, Clone, PartialEq)]
pub struct VisibilityEffect {
    pub op: VisibilityOp,
    pub a: Option<Expr>,
    pub b: Option<Expr>,
    pub c: Option<Expr>,
    pub from: Option<VarRef>,
    pub to: Option<VarRef>,
}

use EffKind::PlayerVisibility as K;

pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    (
        "connect %player% to [the] [server] %text% [on port %number%]",
        K(VisibilityOp::Connect),
    ),
    ("force %player% to respawn", K(VisibilityOp::Respawn)),
    ("force [the] respawn of %player%", K(VisibilityOp::Respawn)),
    ("hide %player% from %player%", K(VisibilityOp::Hide)),
    ("reveal %player% to %player%", K(VisibilityOp::Reveal)),
    ("equip %player% with %item%", K(VisibilityOp::Equip)),
    ("copy %variable% (into|to) %variable%", K(VisibilityOp::Copy)),
];

fn expr(caps: &mut [Option<Capture>], i: usize) -> Option<Expr> {
    match caps.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    }
}

fn var(caps: &mut [Option<Capture>], i: usize) -> Option<VarRef> {
    match caps.get_mut(i).and_then(Option::take) {
        Some(Capture::Var(v)) => Some(v),
        _ => None,
    }
}

pub fn build(op: VisibilityOp, caps: &mut [Option<Capture>]) -> Option<Effect> {
    let mut e = VisibilityEffect {
        op,
        a: None,
        b: None,
        c: None,
        from: None,
        to: None,
    };
    if op == VisibilityOp::Copy {
        e.from = Some(var(caps, 0)?);
        e.to = Some(var(caps, 1)?);
    } else {
        e.a = Some(expr(caps, 0)?);
        e.b = expr(caps, 1);
        e.c = expr(caps, 2);
        if matches!(
            op,
            VisibilityOp::Hide | VisibilityOp::Reveal | VisibilityOp::Equip | VisibilityOp::Connect
        ) && e.b.is_none()
        {
            return None;
        }
    }
    Some(Effect::PlayerVisibility(Box::new(e)))
}

/// Split `host:port` text. A bare host gets the default port 25565.
pub fn split_host(text: &str, port: Option<f64>) -> Option<(String, u16)> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let (host, inline) = match text.rsplit_once(':') {
        Some((h, p)) if !h.contains(':') => (h, p.parse::<u16>().ok()),
        _ => (text, None),
    };
    let port = match port {
        Some(p) if p.is_finite() && (1.0..=65535.0).contains(&p) => p as u16,
        Some(_) => return None,
        None => inline.unwrap_or(25565),
    };
    if host.is_empty() || host.contains(char::is_whitespace) {
        return None;
    }
    Some((host.to_string(), port))
}

/// Slot name for an item id: `armor.head`, `armor.chest`, ... Shields go to
/// the off hand and everything else to the main hand.
pub fn equip_slot(id: &str) -> &'static str {
    let id = id.strip_prefix("minecraft:").unwrap_or(id);
    if id.ends_with("_helmet") || id == "carved_pumpkin" || id.ends_with("_head") || id.ends_with("_skull") {
        "armor.head"
    } else if id.ends_with("_chestplate") || id == "elytra" {
        "armor.chest"
    } else if id.ends_with("_leggings") {
        "armor.legs"
    } else if id.ends_with("_boots") {
        "armor.feet"
    } else if id == "shield" {
        "weapon.offhand"
    } else {
        "weapon.mainhand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_split() {
        assert_eq!(split_host("play.x.net", None), Some(("play.x.net".into(), 25565)));
        assert_eq!(
            split_host("play.x.net:25570", None),
            Some(("play.x.net".into(), 25570))
        );
        assert_eq!(split_host("a.b", Some(1234.0)), Some(("a.b".into(), 1234)));
        assert_eq!(split_host("a.b", Some(0.0)), None);
        assert_eq!(split_host("", None), None);
    }

    #[test]
    fn slots() {
        assert_eq!(equip_slot("diamond_helmet"), "armor.head");
        assert_eq!(equip_slot("minecraft:iron_boots"), "armor.feet");
        assert_eq!(equip_slot("shield"), "weapon.offhand");
        assert_eq!(equip_slot("stick"), "weapon.mainhand");
    }
}
