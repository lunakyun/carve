//! Wave 5, chunk 3: mob and player checks (`is tamed`, `is saddled`,
//! `is wearing`, `is riptiding`, ...), item and block checks (`is stackable`,
//! `lid of ... is open`, glint override), potion effect flags, display flags,
//! `resource pack was ...`, `x is any of ...`, the operator and ban lists,
//! beacon effects, applied enchantments, and bartering drops.
//!
//! Everything sits behind boxed variants (`Cond::W5c3`, `Expr::W5c3`,
//! `Effect::W5c3`), so the hot enums do not grow. The game answers through a
//! single host call, [`Host::w5c3`](crate::host::Host::w5c3).

use crate::compile::{CondKind, EffKind, Effect};
use crate::engine::{Engine, Exec, online};
use crate::expr::{Expr, P, Ty};
use crate::host::Host;
use crate::lexer::Tok;
use crate::pattern::Capture;
use crate::value::{PlayerRef, Value};

/// A yes/no question about an entity or player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Check {
    /// The pose is the trident spin attack.
    Riptiding,
    /// Something sits in the saddle slot.
    Saddled,
    /// An enderman is screaming.
    Screaming,
    /// A wolf or cat has an owner.
    Tamed,
    /// A text display draws its text with a shadow.
    TextShadow,
    /// A text display can be seen through blocks.
    SeeThrough,
}

/// A flag of an active potion effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffFlag {
    Icon,
    Particles,
    Ambient,
}

/// What the core asks the game.
#[derive(Debug, Clone, Copy)]
pub enum Q<'a> {
    /// `Bool` answer.
    Check(Check, &'a Value),
    /// Item ids (without `minecraft:`) worn in the armor, body and saddle slots,
    /// as a list of text.
    Worn(&'a Value),
    /// `Bool`: a flag of the player's active effect (an effect id like `speed`).
    /// `None` when the player does not have that effect.
    Effect(&'a PlayerRef, &'a str, EffFlag),
    /// `Bool`: the lid of the chest or barrel at a block is open.
    Lid(&'a str, i32, i32, i32),
    /// The operators, as a list of players.
    Ops,
    /// Banned players, as a list of players.
    BannedPlayers,
    /// Banned IP addresses, as a list of text.
    BannedIps,
    /// `Text`: the primary (`true`) or secondary effect of a beacon.
    Beacon(&'a str, i32, i32, i32, bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Host(Check),
    Spawnable,
    Stackable,
    Tameable,
    Wearing,
    Glint,
    /// `true`: open, `false`: closed.
    Lid(bool),
    Effect(EffFlag),
    /// Like `Effect`, with the player first (`player's speed effect ...`).
    EffectP(EffFlag),
    Pack,
    /// `x is any of list`.
    AnyOf,
    /// `any of list is x`.
    AnyOfRev,
}

#[derive(Debug, Clone, PartialEq)]
pub struct W5Cond {
    pub kind: Kind,
    pub negate: bool,
    pub a: Expr,
    pub b: Option<Expr>,
}

const fn k(kind: Kind, negate: bool) -> CondKind {
    CondKind::W5c3(kind, negate)
}

const fn h(c: Check, negate: bool) -> CondKind {
    CondKind::W5c3(Kind::Host(c), negate)
}

/// Condition patterns. Tried before the plain `is` comparison.
pub const CONDITION_PATTERNS: &[(&str, CondKind)] = &[
    ("%object% (is|are) riptiding", h(Check::Riptiding, false)),
    (
        "%object% (isn't|is not|aren't|are not) riptiding",
        h(Check::Riptiding, true),
    ),
    ("%object% (is|are) [properly] saddled", h(Check::Saddled, false)),
    (
        "%object% (isn't|is not|aren't|are not) [properly] saddled",
        h(Check::Saddled, true),
    ),
    ("%object% (is|are) screaming", h(Check::Screaming, false)),
    (
        "%object% (isn't|is not|aren't|are not) screaming",
        h(Check::Screaming, true),
    ),
    ("%object% (is|are) tamed", h(Check::Tamed, false)),
    (
        "%object% (isn't|is not|aren't|are not) tamed",
        h(Check::Tamed, true),
    ),
    ("%object% (is|are) tameable", k(Kind::Tameable, false)),
    (
        "%object% (isn't|is not|aren't|are not) tameable",
        k(Kind::Tameable, true),
    ),
    (
        "%object% (is|are) spawnable [in %world%]",
        k(Kind::Spawnable, false),
    ),
    (
        "%object% (isn't|is not|aren't|are not) spawnable [in %world%]",
        k(Kind::Spawnable, true),
    ),
    ("%object% (is|are) stackable", k(Kind::Stackable, false)),
    (
        "%object% (isn't|is not|aren't|are not) stackable",
        k(Kind::Stackable, true),
    ),
    ("%object% (is|are) wearing %object%", k(Kind::Wearing, false)),
    (
        "%object% (isn't|is not|aren't|are not) wearing %object%",
        k(Kind::Wearing, true),
    ),
    (
        "%player%'s (tool|held item) (has|have) [an] (enchantment glint (overridden|override)|overridden enchantment glint)",
        k(Kind::Glint, false),
    ),
    (
        "%player%'s (tool|held item) (doesn't|does not|don't|do not) have [an] (enchantment glint (overridden|override)|overridden enchantment glint)",
        k(Kind::Glint, true),
    ),
    (
        "[the] (tool|held item) of %player% (has|have) [an] (enchantment glint (overridden|override)|overridden enchantment glint)",
        k(Kind::Glint, false),
    ),
    (
        "[the] (tool|held item) of %player% (doesn't|does not|don't|do not) have [an] (enchantment glint (overridden|override)|overridden enchantment glint)",
        k(Kind::Glint, true),
    ),
    (
        "[the] (lid|lids) of %object% (is|are) (open|opened)",
        k(Kind::Lid(true), false),
    ),
    (
        "[the] (lid|lids) of %object% (isn't|is not|aren't|are not) (open|opened)",
        k(Kind::Lid(true), true),
    ),
    (
        "[the] (lid|lids) of %object% (is|are) (closed|close)",
        k(Kind::Lid(false), false),
    ),
    (
        "[the] (lid|lids) of %object% (isn't|is not|aren't|are not) (closed|close)",
        k(Kind::Lid(false), true),
    ),
    (
        "%object%'s (lid|lids) (is|are) (open|opened)",
        k(Kind::Lid(true), false),
    ),
    (
        "%object%'s (lid|lids) (isn't|is not|aren't|are not) (open|opened)",
        k(Kind::Lid(true), true),
    ),
    (
        "%object%'s (lid|lids) (is|are) (closed|close)",
        k(Kind::Lid(false), false),
    ),
    (
        "%object%'s (lid|lids) (isn't|is not|aren't|are not) (closed|close)",
        k(Kind::Lid(false), true),
    ),
    (
        "[the] [potion] effect of %object% (of|on) %player% (has|have) [an] icon",
        k(Kind::Effect(EffFlag::Icon), false),
    ),
    (
        "[the] [potion] effect of %object% (of|on) %player% (doesn't|does not|don't|do not) have [an] icon",
        k(Kind::Effect(EffFlag::Icon), true),
    ),
    (
        "[the] [potion] effect of %object% (of|on) %player% (has|have) particles",
        k(Kind::Effect(EffFlag::Particles), false),
    ),
    (
        "[the] [potion] effect of %object% (of|on) %player% (doesn't|does not|don't|do not) have particles",
        k(Kind::Effect(EffFlag::Particles), true),
    ),
    (
        "[the] [potion] effect of %object% (of|on) %player% (is|are) ambient",
        k(Kind::Effect(EffFlag::Ambient), false),
    ),
    (
        "[the] [potion] effect of %object% (of|on) %player% (isn't|is not|aren't|are not) ambient",
        k(Kind::Effect(EffFlag::Ambient), true),
    ),
    (
        "%player%'s %object% [potion] effect (has|have) [an] icon",
        k(Kind::EffectP(EffFlag::Icon), false),
    ),
    (
        "%player%'s %object% [potion] effect (doesn't|does not|don't|do not) have [an] icon",
        k(Kind::EffectP(EffFlag::Icon), true),
    ),
    (
        "%player%'s %object% [potion] effect (has|have) particles",
        k(Kind::EffectP(EffFlag::Particles), false),
    ),
    (
        "%player%'s %object% [potion] effect (doesn't|does not|don't|do not) have particles",
        k(Kind::EffectP(EffFlag::Particles), true),
    ),
    (
        "%player%'s %object% [potion] effect (is|are) ambient",
        k(Kind::EffectP(EffFlag::Ambient), false),
    ),
    (
        "%player%'s %object% [potion] effect (isn't|is not|aren't|are not) ambient",
        k(Kind::EffectP(EffFlag::Ambient), true),
    ),
    (
        "[the] text of %object% (has|have) [a] (drop|text) shadow",
        h(Check::TextShadow, false),
    ),
    (
        "[the] text of %object% (doesn't|does not|don't|do not) have [a] (drop|text) shadow",
        h(Check::TextShadow, true),
    ),
    (
        "%object% (has|have) [a] (drop|text) shadow",
        h(Check::TextShadow, false),
    ),
    (
        "%object% (doesn't|does not|don't|do not) have [a] (drop|text) shadow",
        h(Check::TextShadow, true),
    ),
    (
        "[the] text of %object% (is|are) visible through (blocks|walls)",
        h(Check::SeeThrough, false),
    ),
    (
        "[the] text of %object% (isn't|is not|aren't|are not) visible through (blocks|walls)",
        h(Check::SeeThrough, true),
    ),
    (
        "%object% (is|are) visible through (blocks|walls)",
        h(Check::SeeThrough, false),
    ),
    (
        "%object% (isn't|is not|aren't|are not) visible through (blocks|walls)",
        h(Check::SeeThrough, true),
    ),
    (
        "[the] resource pack (was|is|has) [been] %object%",
        k(Kind::Pack, false),
    ),
    (
        "[the] resource pack (wasn't|was not|isn't|is not|hasn't|has not) [been] %object%",
        k(Kind::Pack, true),
    ),
    (
        "%object% (is|are|=) (any|any one|one) of %object%",
        k(Kind::AnyOf, false),
    ),
    (
        "%object% (isn't|is not|aren't|are not|!=) (any|any one|one) of %object%",
        k(Kind::AnyOf, true),
    ),
    (
        "(any|any one|one) of %object% (is|are|=) %object%",
        k(Kind::AnyOfRev, false),
    ),
    (
        "(any|any one|one) of %object% (isn't|is not|aren't|are not|!=) %object%",
        k(Kind::AnyOfRev, true),
    ),
];

/// Words one of the patterns above needs. A line without any of them skips
/// the whole list, which keeps compiling fast.
const KEY_WORDS: &[&str] = &[
    "riptiding",
    "saddled",
    "screaming",
    "tamed",
    "tameable",
    "spawnable",
    "stackable",
    "wearing",
    "glint",
    "lid",
    "lids",
    "icon",
    "particles",
    "ambient",
    "shadow",
    "visible",
    "resource",
    "any",
    "one",
];

/// Cheap pre-check before trying [`CONDITION_PATTERNS`].
pub fn line_may_match(toks: &[Tok]) -> bool {
    toks.iter().any(|t| KEY_WORDS.iter().any(|w| t.is_lit(w)))
}

/// Build a condition from pattern captures.
pub fn build_cond(kind: Kind, negate: bool, mut c: Vec<Option<Capture>>) -> W5Cond {
    let mut take = |i: usize| match c.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    };
    let (a, b) = (take(0).unwrap_or(Expr::Num(0.0)), take(1));
    match kind {
        Kind::EffectP(f) => W5Cond {
            kind: Kind::Effect(f),
            negate,
            a: b.unwrap_or(Expr::Num(0.0)),
            b: Some(a),
        },
        _ => W5Cond { kind, negate, a, b },
    }
}

// ---------------------------------------------------------------- helpers

fn norm(s: &str) -> String {
    crate::w2_conditions_3::norm_id(s)
}

/// Entity types that can be tamed.
const TAMEABLE: &[&str] = &[
    "wolf",
    "cat",
    "parrot",
    "horse",
    "donkey",
    "mule",
    "llama",
    "trader_llama",
    "skeleton_horse",
    "zombie_horse",
    "camel",
];

/// Entity types a script can never spawn.
const NOT_SPAWNABLE: &[&str] = &["player", "fishing_bobber"];

/// The entity type id named by a value: an entity, a player, a type name.
fn entity_kind(v: &Value) -> Option<String> {
    match v {
        Value::Entity(e) => Some(norm(&e.kind)),
        Value::Player(_) => Some("player".into()),
        Value::Text(t) => Some(norm(t)),
        Value::Enum(e) => Some(norm(e.id())),
        _ => None,
    }
}

fn known_entity(id: &str) -> bool {
    crate::entity_names::ENTITY_TYPES.contains(&id)
}

fn item_id(v: &Value) -> Option<String> {
    match v {
        Value::Item { id, .. } => Some(norm(id)),
        Value::Text(t) => Some(norm(t)),
        Value::Enum(e) => Some(norm(e.id())),
        _ => None,
    }
}

/// Every value of a list, or the value itself.
fn each(v: &Value) -> Vec<&Value> {
    match v {
        Value::List(l) => l.iter().collect(),
        Value::None => Vec::new(),
        v => vec![v],
    }
}

fn block_of(v: &Value, host: &mut dyn Host) -> Option<(String, i32, i32, i32)> {
    let Value::Location { x, y, z, world } = v else {
        return None;
    };
    let w = match world {
        Some(w) => w.clone(),
        None => host.default_world()?,
    };
    Some((w, x.floor() as i32, y.floor() as i32, z.floor() as i32))
}

/// Skript's resource pack state names for Pumpkin's status text.
pub fn pack_state(raw: &str) -> String {
    match raw {
        "DownloadSuccess" => "successfully_loaded".into(),
        "Declined" => "declined".into(),
        "DownloadFail" => "failed_download".into(),
        "Accepted" => "accepted".into(),
        "Downloaded" => "downloaded".into(),
        "InvalidUrl" => "invalid_url".into(),
        "ReloadFailed" => "failed_reload".into(),
        "Discarded" => "discarded".into(),
        other => norm(other),
    }
}

/// `loaded` and `successfully loaded` both mean the pack is in use.
fn state_word(v: &Value) -> Option<String> {
    let s = match v {
        Value::Text(t) => norm(t),
        Value::Enum(e) => norm(e.id()),
        _ => return None,
    };
    Some(match s.as_str() {
        "loaded" | "successfully_loaded" | "success" => "successfully_loaded".into(),
        "failed" | "download_failed" | "failed_download" => "failed_download".into(),
        "reload_failed" | "failed_reload" => "failed_reload".into(),
        _ => s,
    })
}

/// Status effects by their registry id (beacon block entities store ids).
pub const EFFECT_IDS: &[&str] = &[
    "speed",
    "slowness",
    "haste",
    "mining_fatigue",
    "strength",
    "instant_health",
    "instant_damage",
    "jump_boost",
    "nausea",
    "regeneration",
    "resistance",
    "fire_resistance",
    "water_breathing",
    "invisibility",
    "blindness",
    "night_vision",
    "hunger",
    "weakness",
    "poison",
    "wither",
    "health_boost",
    "absorption",
    "saturation",
    "glowing",
    "levitation",
    "luck",
    "unluck",
    "slow_falling",
    "conduit_power",
    "dolphins_grace",
    "bad_omen",
    "hero_of_the_village",
    "darkness",
    "trial_omen",
    "raid_omen",
    "wind_charged",
    "weaving",
    "oozing",
    "infested",
    "breath_of_the_nautilus",
];

fn effect_name(v: &Value) -> Option<String> {
    let raw = match v {
        Value::Text(t) => t.clone(),
        Value::Enum(e) => e.id().to_string(),
        _ => return None,
    };
    let n = norm(&raw);
    let n = n.strip_suffix("_effect").unwrap_or(&n).to_string();
    crate::world_effects::effect_id(&n.replace('_', " "))
        .map(str::to_string)
        .or(Some(n))
}

fn as_bool(v: Option<Value>) -> Option<bool> {
    match v? {
        Value::Bool(b) => Some(b),
        _ => None,
    }
}

impl Engine {
    /// A value, with a list variable read as all of its items.
    fn c3_eval(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        match e {
            Expr::Var(v) if v.list => Ok(Value::List(
                self.list_items(v, ex, host)?
                    .into_iter()
                    .map(|(_, v)| v)
                    .collect(),
            )),
            e => self.eval(e, ex, host),
        }
    }

    /// Run one of these conditions. `None` from the game counts as false,
    /// negated or not.
    #[cold]
    pub(crate) fn cond_w5c3(
        &mut self,
        c: &W5Cond,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<bool, String> {
        let a = self.c3_eval(&c.a, ex, host)?;
        let b = match &c.b {
            Some(b) => Some(self.c3_eval(b, ex, host)?),
            None => None,
        };
        let hit = match c.kind {
            Kind::AnyOf | Kind::AnyOfRev => {
                let (one, list) = if c.kind == Kind::AnyOf {
                    (&a, b.as_ref().unwrap_or(&Value::None))
                } else {
                    (b.as_ref().unwrap_or(&Value::None), &a)
                };
                Some(each(list).iter().any(|x| one.loose_eq(x)))
            }
            Kind::Pack => {
                let got = ex.ctx.message.as_deref().map(pack_state);
                match (got, state_word(&a)) {
                    (Some(g), Some(w)) => Some(g == w),
                    _ => None,
                }
            }
            Kind::Effect(flag) | Kind::EffectP(flag) => {
                let p = b.as_ref().and_then(|p| online(p, &mut ex.seen, host));
                match (p, effect_name(&a)) {
                    (Some(p), Some(e)) => as_bool(host.w5c3(Q::Effect(&p, &e, flag))),
                    _ => None,
                }
            }
            Kind::Glint => match online(&a, &mut ex.seen, host) {
                Some(p) => host
                    .held_component(&p, crate::skbee_nbt::ItemComp::GlintOverride)
                    .map(|_| true)
                    .or(Some(false)),
                None => None,
            },
            _ => {
                // Every value of a list subject must pass.
                let mut all = None;
                for v in each(&a) {
                    let one = check_one(c.kind, v, b.as_ref(), host);
                    match one {
                        Some(true) => all = Some(all.unwrap_or(true)),
                        Some(false) => {
                            all = Some(false);
                            break;
                        }
                        None => {
                            all = None;
                            break;
                        }
                    }
                }
                all
            }
        };
        Ok(hit.is_some_and(|h| h != c.negate))
    }
}

/// One subject of a check (before negation).
fn check_one(kind: Kind, v: &Value, b: Option<&Value>, host: &mut dyn Host) -> Option<bool> {
    match kind {
        Kind::Host(c) => as_bool(host.w5c3(Q::Check(c, v))),
        Kind::Tameable => {
            let id = entity_kind(v)?;
            Some(TAMEABLE.contains(&id.as_str()))
        }
        Kind::Spawnable => {
            let id = entity_kind(v)?;
            Some(known_entity(&id) && !NOT_SPAWNABLE.contains(&id.as_str()))
        }
        Kind::Stackable => {
            let id = item_id(v)?;
            Some(crate::w4_conditions_1::max_stack(&id) > 1)
        }
        Kind::Wearing => {
            let worn = match host.w5c3(Q::Worn(v))? {
                Value::List(l) => l,
                _ => return None,
            };
            let want = each(b?);
            if want.is_empty() {
                return None;
            }
            Some(want.iter().all(|w| {
                item_id(w).is_some_and(|id| worn.iter().any(|x| matches!(x, Value::Text(t) if norm(t) == id)))
            }))
        }
        Kind::Lid(open) => {
            let (w, x, y, z) = block_of(v, host)?;
            as_bool(host.w5c3(Q::Lid(&w, x, y, z))).map(|is_open| is_open == open)
        }
        Kind::Glint | Kind::Effect(_) | Kind::EffectP(_) | Kind::Pack | Kind::AnyOf | Kind::AnyOfRev => None,
    }
}

// ---------------------------------------------------------------- expressions

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Ops,
    BannedPlayers,
    BannedIps,
    /// `any of <list>`: the list itself; `is` checks treat it as "or".
    AnyOf,
    /// `primary beacon effect of <block>` (`true`) or secondary.
    Beacon(bool),
    /// `applied enchantments` in `on enchant`.
    Applied,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub op: Op,
    pub args: Vec<Expr>,
}

impl Node {
    pub fn ty(&self) -> Ty {
        match self.op {
            Op::Ops | Op::BannedPlayers | Op::BannedIps | Op::Applied => Ty::List,
            Op::AnyOf => Ty::Object,
            Op::Beacon(_) => Ty::Text,
        }
    }
}

fn node(op: Op, args: Vec<Expr>) -> Expr {
    Expr::W5c3(Box::new(Node { op, args }))
}

impl Engine {
    #[cold]
    pub(crate) fn eval_w5c3(
        &mut self,
        n: &Node,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        Ok(match n.op {
            Op::Ops => host.w5c3(Q::Ops).unwrap_or(Value::List(Vec::new())),
            Op::BannedPlayers => host.w5c3(Q::BannedPlayers).unwrap_or(Value::List(Vec::new())),
            Op::BannedIps => host.w5c3(Q::BannedIps).unwrap_or(Value::List(Vec::new())),
            Op::AnyOf => match n.args.first() {
                Some(a) => self.c3_eval(a, ex, host)?,
                None => Value::None,
            },
            Op::Beacon(primary) => {
                let at = match n.args.first() {
                    Some(a) => self.eval(a, ex, host)?,
                    None => return Ok(Value::None),
                };
                match block_of(&at, host) {
                    Some((w, x, y, z)) => host.w5c3(Q::Beacon(&w, x, y, z, primary)).unwrap_or(Value::None),
                    None => Value::None,
                }
            }
            Op::Applied => {
                let texts = ex.ctx.ext.as_deref().map(|v| v.texts.as_slice()).unwrap_or(&[]);
                // After the event's own texts (the item, the offered enchantment).
                let own = usize::from(crate::w2_events_2::spec(crate::w2_events_2::Ev::Enchant).counts.1);
                Value::List(texts.iter().skip(own).map(|t| Value::Text(t.clone())).collect())
            }
        })
    }
}

/// Does this token run start one of our list expressions? (`loop banned
/// players:`, `set {_x::*} to all operators`.)
pub fn is_list_expr(toks: &[Tok]) -> bool {
    let lit = |i: usize, w: &str| toks.get(i).is_some_and(|t| t.is_lit(w));
    let list_at = |i: usize| {
        (lit(i, "banned") && (lit(i + 1, "players") || lit(i + 1, "ips") || lit(i + 1, "ip")))
            || ((lit(i, "ops") || lit(i, "operators")) && toks.len() == i + 1)
    };
    if lit(0, "all") {
        let skip = if lit(1, "of") && lit(2, "the") {
            3
        } else {
            1 + usize::from(lit(1, "the"))
        };
        return list_at(skip);
    }
    list_at(0)
        || (lit(0, "applied") && (lit(1, "enchantments") || lit(1, "enchants")))
        || ((lit(0, "barter") || lit(0, "bartering")) && lit(1, "drops"))
        || ((lit(0, "any") || lit(0, "one")) && (lit(1, "of") || lit(2, "of")))
}

/// Is the trigger `on <name>`?
fn in_event(p: &P<'_>, name: &str) -> bool {
    !p.env.in_function
        && p.env.trigger_name.len() == name.len() + 5
        && p.env.trigger_name[4..].starts_with(name)
}

impl P<'_> {
    fn c3_lit(&self, off: usize, w: &str) -> bool {
        if self.pos + off >= self.toks.len() {
            self.hit_end.set(true);
        }
        matches!(self.toks.get(self.pos + off), Some(t) if t.is_lit(w))
    }

    /// `[all [[of] the]] banned (players|ips)` and `[all [[of] the]] op[erator]s`,
    /// after `all`: how many words to skip before the list name.
    fn c3_all_skip(&self) -> usize {
        match (self.c3_lit(0, "of"), self.c3_lit(1, "the")) {
            (true, true) => 2,
            _ if self.c3_lit(0, "the") => 1,
            _ => 0,
        }
    }

    fn c3_lists(&mut self, skip: usize) -> Option<Expr> {
        if self.c3_lit(skip, "banned") {
            let op = if self.c3_lit(skip + 1, "players") {
                Op::BannedPlayers
            } else if self.c3_lit(skip + 1, "ips")
                || self.c3_lit(skip + 1, "ip") && self.c3_lit(skip + 2, "addresses")
            {
                Op::BannedIps
            } else {
                return None;
            };
            let extra = usize::from(op == Op::BannedIps && self.c3_lit(skip + 1, "ip"));
            self.pos += skip + 2 + extra;
            return Some(node(op, Vec::new()));
        }
        if self.c3_lit(skip, "ops") || self.c3_lit(skip, "operators") {
            self.pos += skip + 1;
            return Some(node(Op::Ops, Vec::new()));
        }
        None
    }

    /// Expressions of this module, by their first word. `Ok(None)`: not ours.
    pub(crate) fn w5c3_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let e = match lw {
            "all" => {
                let skip = self.c3_all_skip();
                match self.c3_lists(skip) {
                    Some(e) => e,
                    None => return Ok(None),
                }
            }
            "banned" => {
                self.pos -= 1;
                match self.c3_lists(0) {
                    Some(e) => e,
                    None => {
                        self.pos += 1;
                        return Ok(None);
                    }
                }
            }
            "ops" | "operators" if !matches!(self.toks.get(self.pos), Some(Tok::Sym("("))) => {
                node(Op::Ops, Vec::new())
            }
            "any" | "one" => {
                let skip = usize::from(lw == "any" && self.c3_lit(0, "one"));
                if !self.c3_lit(skip, "of") {
                    return Ok(None);
                }
                self.pos += skip + 1;
                if self.c3_lit(0, "the") {
                    self.pos += 1;
                }
                let list = self.primary()?;
                node(Op::AnyOf, vec![list])
            }
            "primary" | "secondary" => {
                let skip = usize::from(self.c3_lit(0, "beacon"));
                if !self.c3_lit(skip, "effect") {
                    return Ok(None);
                }
                self.pos += skip + 1;
                if !self.c3_lit(0, "of") {
                    return Err(format!("use '{lw} beacon effect of <block>'"));
                }
                self.pos += 1;
                let at = self.primary()?;
                node(Op::Beacon(lw == "primary"), vec![at])
            }
            "beacon" if self.c3_lit(0, "primary") || self.c3_lit(0, "secondary") => {
                let primary = self.c3_lit(0, "primary");
                if !(self.c3_lit(1, "effect") && self.c3_lit(2, "of")) {
                    return Ok(None);
                }
                self.pos += 3;
                let at = self.primary()?;
                node(Op::Beacon(primary), vec![at])
            }
            "applied" if self.c3_lit(0, "enchantments") || self.c3_lit(0, "enchants") => {
                if self.env.in_function || self.env.ext_event != Some(crate::w2_events_2::Ev::Enchant) {
                    return Err("'applied enchantments' only exists in 'on enchant'".into());
                }
                self.pos += 1;
                node(Op::Applied, Vec::new())
            }
            "anvil" => {
                // `anvil text input`, `anvil [item] repair cost`
                let (slot, used) = if self.c3_lit(0, "text") && self.c3_lit(1, "input") {
                    (0, 2)
                } else if self.c3_lit(0, "repair") && self.c3_lit(1, "cost") {
                    (1, 2)
                } else if self.c3_lit(0, "item") && self.c3_lit(1, "repair") && self.c3_lit(2, "cost") {
                    (1, 3)
                } else {
                    return Ok(None);
                };
                if !in_event(self, "anvil prepare") {
                    return Err("anvil values only exist in 'on anvil prepare'".into());
                }
                self.pos += used;
                Expr::ExtVal(slot)
            }
            "barter" | "bartering" if self.c3_lit(0, "drops") || self.c3_lit(0, "input") => {
                if !in_event(self, "piglin barter") {
                    return Err("barter values only exist in 'on piglin barter'".into());
                }
                let slot = u8::from(self.c3_lit(0, "drops"));
                self.pos += 1;
                Expr::ExtVal(slot)
            }
            _ => return Ok(None),
        };
        Ok(Some(e))
    }
}

// ---------------------------------------------------------------- effects

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EOp {
    SetDrops,
    AddDrops,
    RemoveDrops,
    ClearDrops,
}

pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    (
        "set [the] (barter|bartering) drops to %object%",
        EffKind::W5c3(EOp::SetDrops),
    ),
    (
        "add %object% to [the] (barter|bartering) drops",
        EffKind::W5c3(EOp::AddDrops),
    ),
    (
        "remove %object% from [the] (barter|bartering) drops",
        EffKind::W5c3(EOp::RemoveDrops),
    ),
    (
        "(clear|delete|reset) [the] (barter|bartering) drops",
        EffKind::W5c3(EOp::ClearDrops),
    ),
];

#[derive(Debug, Clone, PartialEq)]
pub struct W5Effect {
    pub op: EOp,
    pub value: Option<Expr>,
}

pub fn build_effect(op: EOp, c: &mut [Option<Capture>]) -> Option<Effect> {
    let value = match c.get_mut(0).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    };
    if op != EOp::ClearDrops && value.is_none() {
        return None;
    }
    Some(Effect::W5c3(Box::new(W5Effect { op, value })))
}

/// Slot of the drops in `ev` in `on piglin barter` (0 is the input item).
pub const DROPS_SLOT: u8 = 1;

fn items_of(v: Value) -> Vec<Value> {
    match v {
        Value::List(l) => l
            .into_iter()
            .filter(|x| matches!(x, Value::Item { .. }))
            .collect(),
        v @ Value::Item { .. } => vec![v],
        _ => Vec::new(),
    }
}

impl Engine {
    #[cold]
    pub(crate) fn run_w5c3(
        &mut self,
        w: &W5Effect,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        if ex.ctx.ev.len() <= usize::from(DROPS_SLOT) {
            return Err("barter drops only exist in 'on piglin barter'".into());
        }
        let v = match &w.value {
            Some(e) => self.c3_eval(e, ex, host)?,
            None => Value::None,
        };
        let slot = usize::from(DROPS_SLOT);
        let mut drops = items_of(std::mem::replace(&mut ex.ctx.ev[slot], Value::None));
        match w.op {
            EOp::SetDrops => drops = items_of(v),
            EOp::AddDrops => drops.extend(items_of(v)),
            EOp::RemoveDrops => {
                let gone = items_of(v);
                drops.retain(|d| !gone.iter().any(|g| g.loose_eq(d)));
            }
            EOp::ClearDrops => drops.clear(),
        }
        let list = Value::List(drops);
        ex.ctx.ev[slot] = list.clone();
        self.ext_out.push((DROPS_SLOT, list));
        Ok(())
    }
}
