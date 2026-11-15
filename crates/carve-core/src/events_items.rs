//! Item, mob and level events: `on drop`, `on pick up`, `on consume`,
//! `on projectile hit`, `on mob spawn`, `on gamemode change`, `on craft` and
//! `on level change`, with their event values (`event-item`, `event-entity`...).
//!
//! The values live in one boxed `EventExtra` on the event context, so the
//! hot `EventCtx` only grows by one pointer. The expression is one small
//! `Copy` enum that is evaluated on the rare path of the engine.

use crate::compile::{CmpOp, Cond, EventKind};
use crate::engine::EventCtx;
use crate::expr::{Expr, Part, Ty};
use crate::lexer::Tok;
use crate::value::{PlayerRef, Value};

/// A point the host reports (a spawn or a projectile hit).
#[derive(Debug, Clone, PartialEq)]
pub struct EvPos {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub world: Option<String>,
}

/// Everything these events can carry. Fields the host does not know stay `None`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventExtra {
    /// Item id without `minecraft:`, and the amount.
    pub item: Option<(String, u32)>,
    /// Entity type id without `minecraft:` (`zombie`).
    pub entity: Option<String>,
    pub location: Option<EvPos>,
    pub reason: Option<String>,
    pub old_mode: Option<String>,
    pub new_mode: Option<String>,
    pub old_level: Option<i64>,
    pub new_level: Option<i64>,
    pub recipe: Option<String>,
    /// The player a projectile hit, when it was a player.
    pub hit_player: Option<PlayerRef>,
}

/// One event value. Each event allows only some of them (see `mask`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventVal {
    Item,
    Entity,
    Location,
    Reason,
    OldMode,
    NewMode,
    OldLevel,
    NewLevel,
    Recipe,
    HitPlayer,
}

impl EventVal {
    fn bit(self) -> u16 {
        1 << (self as u16)
    }

    pub fn ty(self) -> Ty {
        match self {
            EventVal::Item => Ty::Item,
            EventVal::Entity
            | EventVal::Reason
            | EventVal::OldMode
            | EventVal::NewMode
            | EventVal::Recipe => Ty::Text,
            EventVal::Location => Ty::Location,
            EventVal::OldLevel | EventVal::NewLevel => Ty::Number,
            EventVal::HitPlayer => Ty::Player,
        }
    }
}

/// Which event values `on <kind>` offers, as a bit set.
pub fn mask(k: EventKind) -> u16 {
    use EventVal::*;
    let bits: &[EventVal] = match k {
        EventKind::Drop | EventKind::PickUp | EventKind::Consume => &[Item],
        EventKind::ProjectileHit => &[Location, HitPlayer],
        EventKind::MobSpawn => &[Entity, Location, Reason],
        EventKind::GamemodeChange => &[OldMode, NewMode],
        EventKind::Craft => &[Recipe, Item],
        EventKind::LevelChange => &[OldLevel, NewLevel],
        _ => &[],
    };
    bits.iter().fold(0, |m, v| m | v.bit())
}

/// Event names for `EventKind::parse`.
pub fn parse_kind(s: &str) -> Option<EventKind> {
    Some(match s {
        "drop" | "item drop" | "player drop" | "drop item" | "player drop item" => EventKind::Drop,
        "pick up" | "pickup" | "item pickup" | "item pick up" | "player pickup" | "player pick up"
        | "pick up item" => EventKind::PickUp,
        "consume" | "eat" | "drink" | "item consume" | "player consume" => EventKind::Consume,
        "projectile hit" | "projectile land" => EventKind::ProjectileHit,
        "mob spawn" | "spawn" | "creature spawn" | "entity spawn" => EventKind::MobSpawn,
        "gamemode change" | "game mode change" | "player gamemode change" | "player game mode change" => {
            EventKind::GamemodeChange
        }
        "craft" | "item craft" | "craft item" | "player craft" => EventKind::Craft,
        "level change" | "player level change" => EventKind::LevelChange,
        _ => return None,
    })
}

/// The `on <name>` names, for the "unknown event" error text.
pub const NAMES: &str =
    "drop, pick up, consume, projectile hit, mob spawn, gamemode change, craft, level change";

/// Splits `spawn of zombie` and `gamemode change to creative` into the event
/// name and a filter `(what, text)`. Other names come back unchanged.
pub fn split_filter(ev: &str) -> (&str, Option<(&'static str, String)>) {
    for (what, marker) in [("entity", " of "), ("mode", " to "), ("victim", " of ")] {
        let Some(i) = ev.find(marker) else { continue };
        let base = ev.get(..i).unwrap_or(ev).trim();
        let rest = ev.get(i + marker.len()..).unwrap_or("").trim();
        let right_base = match what {
            "entity" => base.ends_with("spawn"),
            "victim" => base == "damage" || base == "death",
            _ => base.ends_with("change") && base.contains("game"),
        };
        if right_base && !rest.is_empty() {
            return (base, Some((what, rest.to_string())));
        }
    }
    (ev, None)
}

/// Whether `on <kind> <filter>` is allowed for this filter kind.
pub fn filter_ok(k: EventKind, what: &str) -> bool {
    match what {
        "with" | "on" => matches!(k, EventKind::Click | EventKind::RightClick | EventKind::LeftClick),
        "entity" => k == EventKind::MobSpawn,
        "victim" => matches!(k, EventKind::Damage | EventKind::Death),
        "mode" => k == EventKind::GamemodeChange,
        "prefix" => k == EventKind::Ext4(crate::w2_events_4::ExtEvent::PlaceholderRequest),
        "command" => k == EventKind::Mv(crate::w4_events_movement::MvEvent::Command),
        "stepon" => k == EventKind::Ext4(crate::w2_events_4::ExtEvent::StepOn),
        _ => false,
    }
}

/// Entity type or mode text of a filter as the text it is compared with.
pub fn filter_rhs(what: &str, text: &str) -> Option<Expr> {
    let t = text.trim();
    let norm = match what {
        "entity" => {
            let t = t
                .strip_prefix("a ")
                .or_else(|| t.strip_prefix("an "))
                .unwrap_or(t);
            let t = t.strip_prefix("minecraft:").unwrap_or(t);
            t.replace(' ', "_")
        }
        "mode" => crate::gamemode::normalize(t)?.to_string(),
        "victim" => {
            let t = t
                .strip_prefix("a ")
                .or_else(|| t.strip_prefix("an "))
                .unwrap_or(t);
            let t = t.strip_prefix("minecraft:").unwrap_or(t);
            crate::entities::kind_of(&t.replace(' ', "_"))?.to_string()
        }
        "prefix" => t.to_ascii_lowercase(),
        _ => return None,
    };
    Some(Expr::Text(vec![Part::Lit(norm)]))
}

/// The value a filter compares against the event.
pub fn filter_lhs(what: &str) -> Option<Expr> {
    match what {
        "entity" => Some(Expr::EventVal(EventVal::Entity)),
        "victim" => Some(crate::entities::victim_type()),
        "mode" => Some(Expr::EventVal(EventVal::NewMode)),
        "prefix" => Some(Expr::Ev4Val(2)),
        _ => None,
    }
}

/// Reads an event value name at the start of `first` + `rest`. Returns the
/// value and how many tokens of `rest` it used.
pub fn parse_val(allowed: u16, first: &str, rest: &[Tok]) -> Option<(EventVal, usize)> {
    let w = |i: usize, s: &str| matches!(rest.get(i), Some(t) if t.is_lit(s));
    let (v, n) = match first {
        "event-item" => (EventVal::Item, 0),
        "event-entity" => (EventVal::Entity, 0),
        "event-location" => (EventVal::Location, 0),
        "event-recipe" => (EventVal::Recipe, 0),
        "recipe" if w(0, "id") => (EventVal::Recipe, 1),
        "spawn" if w(0, "reason") => (EventVal::Reason, 1),
        "hit" if w(0, "player") => (EventVal::HitPlayer, 1),
        "old" | "previous" | "new" => {
            let old = first != "new";
            if w(0, "gamemode") {
                (if old { EventVal::OldMode } else { EventVal::NewMode }, 1)
            } else if w(0, "game") && w(1, "mode") {
                (if old { EventVal::OldMode } else { EventVal::NewMode }, 2)
            } else if w(0, "level") {
                (
                    if old {
                        EventVal::OldLevel
                    } else {
                        EventVal::NewLevel
                    },
                    1,
                )
            } else {
                return None;
            }
        }
        _ => return None,
    };
    (allowed & v.bit() != 0).then_some((v, n))
}

/// `event-entity is [a|an] zombie` and its negations: the entity type is a
/// bare name, which is not an item, so this is parsed by hand.
pub fn parse_entity_cond(toks: &[Tok], env: &crate::expr::Env) -> Option<Cond> {
    if env.event_vals & EventVal::Entity.bit() == 0 {
        return None;
    }
    let (first, rest) = toks.split_first()?;
    if !first.is_lit("event-entity") {
        return None;
    }
    let (eq, rest) = match rest {
        [a, rest @ ..] if a.is_lit("is") => match rest {
            [n, rest @ ..] if n.is_lit("not") => (false, rest),
            _ => (true, rest),
        },
        [a, rest @ ..] if a.is_lit("isn't") => (false, rest),
        _ => return None,
    };
    let rest = match rest {
        [a, rest @ ..] if a.is_lit("a") || a.is_lit("an") => rest,
        _ => rest,
    };
    let mut name = String::new();
    for t in rest {
        let Tok::Word(w) = t else { return None };
        if !name.is_empty() {
            name.push('_');
        }
        name.push_str(&w.to_ascii_lowercase());
    }
    if name.is_empty() {
        return None;
    }
    let op = if eq { CmpOp::Eq } else { CmpOp::Ne };
    Some(Cond::Cmp(
        Expr::EventVal(EventVal::Entity),
        op,
        Expr::Text(vec![Part::Lit(name)]),
    ))
}

/// Value of an event value in this event; `<none>` when the host gave none.
#[inline(never)]
pub fn eval(v: EventVal, ctx: &EventCtx) -> Value {
    let Some(x) = ctx.extra.as_deref() else {
        return Value::None;
    };
    let text = |s: &Option<String>| s.clone().map(Value::Text).unwrap_or(Value::None);
    let num = |n: Option<i64>| n.map(|n| Value::Num(n as f64)).unwrap_or(Value::None);
    match v {
        EventVal::Item => match &x.item {
            Some((id, count)) => Value::Item {
                id: id.clone(),
                count: *count,
                meta: None,
            },
            None => Value::None,
        },
        EventVal::Entity => text(&x.entity),
        EventVal::Location => match &x.location {
            Some(p) => Value::Location {
                x: p.x,
                y: p.y,
                z: p.z,
                world: p.world.clone(),
            },
            None => Value::None,
        },
        EventVal::Reason => text(&x.reason),
        EventVal::OldMode => text(&x.old_mode),
        EventVal::NewMode => text(&x.new_mode),
        EventVal::OldLevel => num(x.old_level),
        EventVal::NewLevel => num(x.new_level),
        EventVal::Recipe => text(&x.recipe),
        EventVal::HitPlayer => x.hit_player.clone().map(Value::Player).unwrap_or(Value::None),
    }
}

impl EventCtx {
    /// A context with a player and event values.
    pub fn with_extra(p: PlayerRef, extra: EventExtra) -> Self {
        let mut c = EventCtx::player(p);
        c.extra = Some(Box::new(extra));
        c
    }

    /// A context without a player (a mob spawn, an unknown pick-up entity).
    pub fn console_with_extra(extra: EventExtra) -> Self {
        let mut c = EventCtx::console();
        c.extra = Some(Box::new(extra));
        c
    }
}
