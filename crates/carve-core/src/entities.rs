//! Entities: spawning, killing, pushing, finding and testing them.
//!
//! The lines here have free-form words (`all zombies in radius 10 of player`),
//! so they are parsed by hand. The compiler calls `parse_effect` and
//! `parse_cond`, the expression parser calls `parse_expr_at`, and the
//! `Engine` methods at the bottom run the results. All of it sits behind
//! boxed variants, so the hot enums in the engine do not grow.

use crate::compile::{Cond, Effect};
use crate::engine::{Engine, Exec, online};
use crate::entity_names::ENTITY_TYPES;
use crate::expr::{Env, Expr, Ty, parse_expr};
use crate::host::{Host, LogLevel};
use crate::lexer::Tok;
use crate::value::{EntityRef, PlayerRef, Value};
use crate::world_effects::FxPos;

const MAX_SPAWN: f64 = 100.0;
const MAX_SPEED: f64 = 20.0;
/// Search radius when a script gives none.
const UNLIMITED: f64 = 1.0e9;

#[derive(Debug, Clone, PartialEq)]
pub enum EntFx {
    Spawn {
        kind: String,
        count: Option<Expr>,
        at: Option<Expr>,
    },
    Kill(Expr),
    Remove(Expr),
    /// Add `dir * speed` to the velocity.
    Push {
        who: Expr,
        dir: (f64, f64, f64),
        speed: Option<Expr>,
    },
    /// `set y velocity of <e> to 1`: axis 0, 1 or 2.
    SetVel {
        who: Expr,
        axis: u8,
        value: Expr,
    },
    SetName {
        who: Expr,
        name: Expr,
    },
    Teleport {
        who: Expr,
        to: Expr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntCond {
    pub subject: Expr,
    pub kind: String,
    pub negate: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntExpr {
    LastSpawned,
    /// A list. `None` kind: every entity. `at: None`: everywhere.
    All {
        kind: Option<String>,
        radius: Option<Expr>,
        at: Option<Expr>,
    },
    Nearest {
        kind: Option<String>,
        to: Expr,
        radius: Option<Expr>,
    },
    TypeOf(Expr),
    /// How many items a list expression has.
    Count(Expr),
    /// Axis 0, 1 or 2 of the velocity.
    Vel(u8, Expr),
}

// ---------- names ----------

/// `zombie`, `zombies`, `iron golems`: the entity type id.
pub fn kind_of(name: &str) -> Option<&'static str> {
    let find = |n: &str| ENTITY_TYPES.iter().copied().find(|k| *k == n);
    find(name)
        .or_else(|| name.strip_suffix('s').and_then(find))
        .or_else(|| name.strip_suffix("es").and_then(find))
        .or_else(|| name.strip_suffix("ies").and_then(|s| find(&format!("{s}y"))))
        .or_else(|| name.strip_suffix("ves").and_then(|s| find(&format!("{s}f"))))
}

/// Plain words joined with `_`, lowercase.
fn joined(toks: &[Tok]) -> Option<String> {
    let mut out = String::new();
    for t in toks {
        let Tok::Word(w) = t else {
            return None;
        };
        if !out.is_empty() {
            out.push('_');
        }
        out.push_str(&w.to_ascii_lowercase());
    }
    (!out.is_empty()).then_some(out)
}

fn kind_words(toks: &[Tok]) -> Option<&'static str> {
    kind_of(&joined(toks)?)
}

/// Words that mean "any entity".
fn any_entity(toks: &[Tok]) -> bool {
    matches!(joined(toks).as_deref(), Some("entity" | "entities" | "mob" | "mobs" | "creature" | "creatures"))
}

/// `None` inside: every entity. `Err`: not a kind at all.
fn kind_filter(toks: &[Tok]) -> Result<Option<String>, ()> {
    if toks.is_empty() || any_entity(toks) {
        return Ok(None);
    }
    kind_words(toks).map(|k| Some(k.to_string())).ok_or(())
}

fn is(t: &Tok, w: &str) -> bool {
    t.is_lit(w)
}

fn pos_of(toks: &[Tok], from: usize, words: &[&str]) -> Option<usize> {
    toks.iter()
        .enumerate()
        .skip(from)
        .find(|(_, t)| words.iter().any(|w| is(t, w)))
        .map(|(i, _)| i)
}

const PREPS: &[&str] = &["of", "around", "from", "near", "at", "to"];

/// The end of a sub-expression: the next word that starts a condition or a radius.
fn expr_end(toks: &[Tok], from: usize) -> usize {
    pos_of(
        toks,
        from,
        &["is", "isn't", "are", "aren't", "has", "hasn't", "then", "in", "within"],
    )
    .unwrap_or(toks.len())
}

fn sub(toks: &[Tok], a: usize, b: usize, env: &Env) -> Result<Expr, String> {
    let s = toks.get(a..b).filter(|s| !s.is_empty()).ok_or("missing expression")?;
    parse_expr(s, env)
}

/// `[a] [radius [of]] N [blocks]` from `i`: the radius and the next position.
fn radius_at(toks: &[Tok], mut i: usize, env: &Env) -> Result<(Expr, usize), String> {
    while toks
        .get(i)
        .is_some_and(|t| is(t, "a") || is(t, "the") || is(t, "radius") || is(t, "of"))
    {
        i += 1;
    }
    let end = pos_of(toks, i, PREPS).unwrap_or(toks.len());
    let mut num_end = end;
    if end > i
        && toks
            .get(end - 1)
            .is_some_and(|u| ["block", "blocks", "meter", "meters", "metre", "metres"].iter().any(|w| is(u, w)))
    {
        num_end = end - 1;
    }
    if num_end <= i {
        return Err("expected a radius number".into());
    }
    Ok((sub(toks, i, num_end, env)?, end))
}

// ---------- expressions ----------

fn ent(e: EntExpr) -> Expr {
    Expr::Ent(Box::new(e))
}

/// An entity expression starting at `start` (the first word). `Ok(None)`:
/// not ours. On success: the expression and the position after it.
pub fn parse_expr_at(
    toks: &[Tok],
    start: usize,
    env: &Env,
    hit_end: &std::cell::Cell<bool>,
) -> Result<Option<(Expr, usize)>, String> {
    let Some(Tok::Word(w)) = toks.get(start) else {
        return Ok(None);
    };
    let r = match w.to_ascii_lowercase().as_str() {
        "last" => last_spawned(toks, start),
        "nearest" | "closest" => nearest(toks, start, env),
        "all" => all_of(toks, start, env),
        "entity" | "type" => type_of(toks, start, env),
        "x" | "y" | "z" => velocity(toks, start, env),
        "size" | "amount" | "number" => count(toks, start, env),
        _ => return Ok(None),
    };
    match r {
        Ok(v) => Ok(v),
        Err(e) => {
            // The words may continue past the cut, so longer spans can still fit.
            hit_end.set(true);
            Err(e)
        }
    }
}

type Parsed = Result<Option<(Expr, usize)>, String>;

fn last_spawned(toks: &[Tok], s: usize) -> Parsed {
    if !toks.get(s + 1).is_some_and(|t| is(t, "spawned")) {
        return Ok(None);
    }
    match toks.get(s + 2) {
        Some(Tok::Word(w)) if matches!(w.to_ascii_lowercase().as_str(), "entity" | "mob" | "creature") => {
            Ok(Some((ent(EntExpr::LastSpawned), s + 3)))
        }
        Some(Tok::Word(w)) if kind_of(&w.to_ascii_lowercase()).is_some() => {
            Ok(Some((ent(EntExpr::LastSpawned), s + 3)))
        }
        _ => Err("expected 'last spawned entity'".into()),
    }
}

fn nearest(toks: &[Tok], s: usize, env: &Env) -> Parsed {
    let Some(p) = pos_of(toks, s + 1, PREPS) else {
        return Err("expected 'nearest entity to <place>'".into());
    };
    let words = toks.get(s + 1..p).unwrap_or_default();
    let Ok(kind) = kind_filter(words) else {
        return Ok(None);
    };
    let stop = expr_end(toks, p + 1);
    let to = sub(toks, p + 1, stop, env)?;
    let (radius, end) = if toks.get(stop).is_some_and(|t| is(t, "in") || is(t, "within")) {
        let (r, j) = radius_at(toks, stop + 1, env)?;
        (Some(r), j)
    } else {
        (None, stop)
    };
    Ok(Some((ent(EntExpr::Nearest { kind, to, radius }), end)))
}

fn all_of(toks: &[Tok], s: usize, env: &Env) -> Parsed {
    let m = pos_of(toks, s + 1, &["in", "within"]);
    let words = toks.get(s + 1..m.unwrap_or(toks.len())).unwrap_or_default();
    if words.is_empty() {
        return Ok(None);
    }
    let kind = match joined(words).as_deref() {
        Some("player" | "players") => Some("player".to_string()),
        _ => match kind_filter(words) {
            Ok(k) => k,
            Err(()) => return Ok(None),
        },
    };
    let Some(m) = m else {
        return Ok(Some((
            ent(EntExpr::All {
                kind,
                radius: None,
                at: None,
            }),
            toks.len(),
        )));
    };
    let (radius, j) = radius_at(toks, m + 1, env)?;
    if !toks.get(j).is_some_and(|t| PREPS.iter().any(|w| is(t, w))) {
        return Err("expected 'of <place>' after the radius".into());
    }
    let stop = expr_end(toks, j + 1);
    let at = sub(toks, j + 1, stop, env)?;
    Ok(Some((
        ent(EntExpr::All {
            kind,
            radius: Some(radius),
            at: Some(at),
        }),
        stop,
    )))
}

fn type_of(toks: &[Tok], s: usize, env: &Env) -> Parsed {
    let mut i = s;
    if toks.get(i).is_some_and(|t| is(t, "entity")) {
        i += 1;
    }
    if !(toks.get(i).is_some_and(|t| is(t, "type")) && toks.get(i + 1).is_some_and(|t| is(t, "of"))) {
        return Ok(None);
    }
    let stop = expr_end(toks, i + 2);
    Ok(Some((ent(EntExpr::TypeOf(sub(toks, i + 2, stop, env)?)), stop)))
}

/// `size of all zombies in radius 10 of player`.
fn count(toks: &[Tok], s: usize, env: &Env) -> Parsed {
    if !(toks.get(s + 1).is_some_and(|t| is(t, "of")) && toks.get(s + 2).is_some_and(|t| is(t, "all"))) {
        return Ok(None);
    }
    match all_of(toks, s + 2, env)? {
        Some((list, end)) => Ok(Some((ent(EntExpr::Count(list)), end))),
        None => Ok(None),
    }
}

fn velocity(toks: &[Tok], s: usize, env: &Env) -> Parsed {
    if !(toks.get(s + 1).is_some_and(|t| is(t, "velocity")) && toks.get(s + 2).is_some_and(|t| is(t, "of"))) {
        return Ok(None);
    }
    let axis = match toks.get(s) {
        Some(t) if is(t, "x") => 0,
        Some(t) if is(t, "y") => 1,
        _ => 2,
    };
    let stop = expr_end(toks, s + 3);
    Ok(Some((ent(EntExpr::Vel(axis, sub(toks, s + 3, stop, env)?)), stop)))
}

pub(crate) fn ty(e: &EntExpr) -> Ty {
    match e {
        EntExpr::TypeOf(_) => Ty::Text,
        EntExpr::Vel(..) | EntExpr::Count(_) => Ty::Number,
        EntExpr::All { .. } => Ty::List,
        _ => Ty::Object,
    }
}

/// Entity list expressions for `loop`.
pub fn parse_list(toks: &[Tok], env: &Env) -> Option<Expr> {
    let hit = std::cell::Cell::new(false);
    match parse_expr_at(toks, 0, env, &hit) {
        Ok(Some((e @ Expr::Ent(_), end))) if end == toks.len() => Some(e),
        _ => None,
    }
}

// ---------- effects ----------

fn is_entity_expr(e: &Expr) -> bool {
    matches!(e, Expr::Ent(_))
}

/// Strip a leading `the` and `entity`/`entities`; true when `entity` was there.
fn strip_entity(toks: &[Tok]) -> (&[Tok], bool) {
    let mut t = toks;
    if t.first().is_some_and(|x| is(x, "the")) {
        t = t.get(1..).unwrap_or_default();
    }
    if t.first().is_some_and(|x| is(x, "entity") || is(x, "entities")) && t.len() > 1 {
        return (t.get(1..).unwrap_or_default(), true);
    }
    (t, false)
}

/// `Ok(None)`: not an entity line. `Err`: it is one, but wrong.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    let fx = match first.to_ascii_lowercase().as_str() {
        "spawn" | "summon" => spawn(toks, env)?,
        "kill" | "remove" | "delete" | "despawn" => kill(toks, env)?,
        "push" => push(toks, env)?,
        "set" => set(toks, env)?,
        "teleport" | "tp" => teleport(toks, env)?,
        _ => None,
    };
    Ok(fx.map(|f| Effect::Ent(Box::new(f))))
}

fn spawn(toks: &[Tok], env: &Env) -> Result<Option<EntFx>, String> {
    let at_i = pos_of(toks, 1, &["at", "on", "near"]);
    let mut head = toks.get(1..at_i.unwrap_or(toks.len())).unwrap_or_default();
    while head.first().is_some_and(|t| is(t, "a") || is(t, "an") || is(t, "the")) {
        head = head.get(1..).unwrap_or_default();
    }
    let mut count = None;
    if let Some((c @ (Tok::Num(_) | Tok::Var(_)), rest)) = head.split_first() {
        count = Some(parse_expr(std::slice::from_ref(c), env)?);
        head = rest;
    }
    let Some(name) = joined(head) else {
        return Ok(None);
    };
    if matches!(name.as_str(), "lightning" | "lightning_bolt" | "explosion") || name.ends_with("particles") {
        return Ok(None);
    }
    let Some(kind) = kind_of(&name) else {
        return Err(format!("unknown entity '{}'", name.replace('_', " ")));
    };
    if kind == "player" {
        return Err("a script can't spawn a player".into());
    }
    let at = match at_i {
        Some(i) => Some(sub(toks, i + 1, toks.len(), env)?),
        None if env.no_player || env.in_function => {
            return Err("spawn needs 'at <location>' here, there is no player".into());
        }
        None => None,
    };
    Ok(Some(EntFx::Spawn {
        kind: kind.to_string(),
        count,
        at,
    }))
}

fn kill(toks: &[Tok], env: &Env) -> Result<Option<EntFx>, String> {
    let verb = toks.first().map(|t| matches!(t, Tok::Word(w) if w.eq_ignore_ascii_case("kill")));
    let (rest, said) = strip_entity(toks.get(1..).unwrap_or_default());
    if rest.is_empty() {
        return Ok(None);
    }
    let Ok(e) = parse_expr(rest, env) else {
        return Ok(None);
    };
    if !(said || is_entity_expr(&e)) {
        return Ok(None);
    }
    Ok(Some(if verb == Some(true) {
        EntFx::Kill(e)
    } else {
        EntFx::Remove(e)
    }))
}

const DIRS: &[(&str, (f64, f64, f64))] = &[
    ("up", (0.0, 1.0, 0.0)),
    ("upwards", (0.0, 1.0, 0.0)),
    ("upward", (0.0, 1.0, 0.0)),
    ("down", (0.0, -1.0, 0.0)),
    ("downwards", (0.0, -1.0, 0.0)),
    ("downward", (0.0, -1.0, 0.0)),
    ("north", (0.0, 0.0, -1.0)),
    ("south", (0.0, 0.0, 1.0)),
    ("east", (1.0, 0.0, 0.0)),
    ("west", (-1.0, 0.0, 0.0)),
];

/// `push <who> upwards [with speed 2]`.
fn push(toks: &[Tok], env: &Env) -> Result<Option<EntFx>, String> {
    let Some((d, dir)) = toks.iter().enumerate().skip(2).find_map(|(i, t)| match t {
        Tok::Word(w) => DIRS
            .iter()
            .find(|(n, _)| w.eq_ignore_ascii_case(n))
            .map(|(_, v)| (i, *v)),
        _ => None,
    }) else {
        return Ok(None);
    };
    let who = sub(toks, 1, d, env)?;
    let mut after = toks.get(d + 1..).unwrap_or_default();
    let mut speed = None;
    if let Some((p, rest)) = after.split_first()
        && (is(p, "with") || is(p, "at") || is(p, "by") || is(p, "using"))
    {
        after = rest;
        while after.first().is_some_and(|t| is(t, "the") || is(t, "a")) {
            after = after.get(1..).unwrap_or_default();
        }
        let Some((w, n)) = after.split_first() else {
            return Err("expected 'speed <number>'".into());
        };
        if !(is(w, "speed") || is(w, "velocity") || is(w, "force")) {
            return Err("expected 'with speed <number>'".into());
        }
        speed = Some(parse_expr(n, env)?);
    } else if !after.is_empty() {
        return Err("expected 'with speed <number>' after the direction".into());
    }
    Ok(Some(EntFx::Push { who, dir, speed }))
}

/// `set [the] name of <entity> to "x"`, `set [the] y velocity of <e> to 1`.
fn set(toks: &[Tok], env: &Env) -> Result<Option<EntFx>, String> {
    let mut i = 1;
    if toks.get(i).is_some_and(|t| is(t, "the")) {
        i += 1;
    }
    let (Some(w), Some(n)) = (toks.get(i), toks.get(i + 1)) else {
        return Ok(None);
    };
    if (is(w, "x") || is(w, "y") || is(w, "z")) && is(n, "velocity") {
        let axis = if is(w, "x") { 0 } else if is(w, "y") { 1 } else { 2 };
        if !toks.get(i + 2).is_some_and(|t| is(t, "of")) {
            return Ok(None);
        }
        let Some(to) = pos_of(toks, i + 3, &["to"]) else {
            return Ok(None);
        };
        return Ok(Some(EntFx::SetVel {
            who: sub(toks, i + 3, to, env)?,
            axis,
            value: sub(toks, to + 1, toks.len(), env)?,
        }));
    }
    if is(w, "name") && is(n, "of") {
        let Some(to) = pos_of(toks, i + 2, &["to"]) else {
            return Ok(None);
        };
        let (who_toks, said) = strip_entity(toks.get(i + 2..to).unwrap_or_default());
        let Ok(who) = parse_expr(who_toks, env) else {
            return Ok(None);
        };
        if !(said || is_entity_expr(&who)) {
            return Ok(None);
        }
        return Ok(Some(EntFx::SetName {
            who,
            name: sub(toks, to + 1, toks.len(), env)?,
        }));
    }
    Ok(None)
}

fn teleport(toks: &[Tok], env: &Env) -> Result<Option<EntFx>, String> {
    let Some(to) = pos_of(toks, 2, &["to"]) else {
        return Ok(None);
    };
    let (who_toks, said) = strip_entity(toks.get(1..to).unwrap_or_default());
    let Ok(who) = parse_expr(who_toks, env) else {
        return Ok(None);
    };
    if !(said || is_entity_expr(&who)) {
        return Ok(None);
    }
    Ok(Some(EntFx::Teleport {
        who,
        to: sub(toks, to + 1, toks.len(), env)?,
    }))
}

// ---------- conditions ----------

/// `<x> is [not] a zombie`, `<x> isn't a zombie`.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    let i = pos_of(toks, 1, &["is", "isn't"])?;
    let mut negate = toks.get(i).is_some_and(|t| is(t, "isn't"));
    let mut j = i + 1;
    if toks.get(j).is_some_and(|t| is(t, "not")) {
        negate = !negate;
        j += 1;
    }
    let article = toks.get(j).is_some_and(|t| is(t, "a") || is(t, "an"));
    if article {
        j += 1;
    }
    let kind = kind_words(toks.get(j..)?)?;
    // `arg-1 is player` compares two players; only `is a player` is a type test.
    if kind == "player" && !article {
        return None;
    }
    let subject = parse_expr(toks.get(..i)?, env).ok()?;
    Some(Cond::Ent(Box::new(EntCond {
        subject,
        kind: kind.to_string(),
        negate,
    })))
}

// ---------- running ----------

fn flatten(v: Value, out: &mut Vec<Value>) {
    match v {
        Value::List(items) => items.into_iter().for_each(|x| flatten(x, out)),
        Value::None => {}
        other => out.push(other),
    }
}

fn handle(v: &Value) -> Option<EntityRef> {
    match v {
        Value::Entity(e) => Some(e.clone()),
        Value::Player(p) => Some(EntityRef {
            uuid: p.uuid.clone(),
            kind: "player".into(),
        }),
        _ => None,
    }
}

fn dist(a: (f64, f64, f64), b: &FxPos) -> f64 {
    let (dx, dy, dz) = (a.0 - b.x, a.1 - b.y, a.2 - b.z);
    (dx * dx + dy * dy + dz * dz).sqrt()
}

impl Engine {
    fn ent_targets(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<Vec<Value>, String> {
        let v = self.eval(e, ex, host)?;
        let mut out = Vec::new();
        flatten(v, &mut out);
        Ok(out)
    }

    /// A point from a location, player or entity value.
    fn ent_pos(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<Option<FxPos>, String> {
        let v = self.eval(e, ex, host)?;
        Ok(match &v {
            Value::Location { x, y, z, world } => Some(FxPos {
                x: *x,
                y: *y,
                z: *z,
                world: world.clone(),
            }),
            Value::Entity(r) => host.entity_location(r).map(|(x, y, z, w)| FxPos {
                x,
                y,
                z,
                world: Some(w),
            }),
            Value::Player(_) | Value::Text(_) => {
                let Some(p) = online(&v, &mut ex.seen, host) else {
                    return Ok(None);
                };
                host.player_location(&p).map(|(x, y, z, w)| FxPos {
                    x,
                    y,
                    z,
                    world: Some(w),
                })
            }
            Value::None => None,
            other => {
                return Err(format!(
                    "expected a location, player or entity, got {} ({other})",
                    other.type_name()
                ));
            }
        })
    }

    fn ent_num(&mut self, e: &Option<Expr>, default: f64, max: f64, ex: &mut Exec, host: &mut dyn Host) -> Result<f64, String> {
        match e {
            Some(e) => {
                let n = self.num(e, ex, host)?;
                Ok(if n.is_finite() { n.clamp(-max, max) } else { default })
            }
            None => Ok(default),
        }
    }

    #[inline(never)]
    pub(crate) fn ent_effect(&mut self, f: &EntFx, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        match f {
            EntFx::Spawn { kind, count, at } => {
                let n = self.ent_num(count, 1.0, MAX_SPAWN, ex, host)?.clamp(0.0, MAX_SPAWN) as u32;
                let at = match at {
                    Some(a) => a,
                    None => &Expr::Player,
                };
                let Some(pos) = self.ent_pos(at, ex, host)? else {
                    return Ok(());
                };
                for _ in 0..n {
                    if let Some(e) = host.spawn_entity(kind, &pos) {
                        ex.last_entity = Some(e);
                    }
                }
            }
            EntFx::Kill(e) => self.kill_any(e, ex, host)?,
            EntFx::Remove(e) => {
                for v in self.ent_targets(e, ex, host)? {
                    match &v {
                        Value::Entity(r) => host.entity_remove(r),
                        _ => host.log(LogLevel::Warn, "a player can't be removed, use kick or kill"),
                    }
                }
            }
            EntFx::Push { who, dir, speed } => {
                let s = self.ent_num(speed, 1.0, MAX_SPEED, ex, host)?;
                for v in self.ent_targets(who, ex, host)? {
                    if let Some(r) = self.live_handle(&v, ex, host) {
                        let cur = host.entity_velocity(&r).unwrap_or((0.0, 0.0, 0.0));
                        host.set_entity_velocity(&r, (cur.0 + dir.0 * s, cur.1 + dir.1 * s, cur.2 + dir.2 * s));
                    }
                }
            }
            EntFx::SetVel { who, axis, value } => {
                let n = self.num(value, ex, host)?;
                let n = if n.is_finite() { n.clamp(-MAX_SPEED, MAX_SPEED) } else { 0.0 };
                for v in self.ent_targets(who, ex, host)? {
                    if let Some(r) = self.live_handle(&v, ex, host) {
                        let mut cur = host.entity_velocity(&r).unwrap_or((0.0, 0.0, 0.0));
                        match axis {
                            0 => cur.0 = n,
                            1 => cur.1 = n,
                            _ => cur.2 = n,
                        }
                        host.set_entity_velocity(&r, cur);
                    }
                }
            }
            EntFx::SetName { who, name } => {
                let name = self.text(name, ex, host)?;
                for v in self.ent_targets(who, ex, host)? {
                    match &v {
                        Value::Entity(r) => host.set_entity_name(r, Some(&name)),
                        Value::Player(_) => {
                            if let Some(p) = online(&v, &mut ex.seen, host) {
                                host.set_display_name(&p, Some(&name));
                            }
                        }
                        _ => {}
                    }
                }
            }
            EntFx::Teleport { who, to } => {
                let Some(pos) = self.ent_pos(to, ex, host)? else {
                    return Ok(());
                };
                for v in self.ent_targets(who, ex, host)? {
                    match &v {
                        Value::Entity(r) => host.entity_teleport(r, pos.x, pos.y, pos.z, pos.world.as_deref()),
                        Value::Player(_) => {
                            if let Some(p) = online(&v, &mut ex.seen, host) {
                                host.teleport_to(&p, pos.x, pos.y, pos.z, pos.world.as_deref());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    /// A handle for an entity, or an online player.
    fn live_handle(&mut self, v: &Value, ex: &mut Exec, host: &mut dyn Host) -> Option<EntityRef> {
        match v {
            Value::Entity(r) => Some(r.clone()),
            Value::Player(_) | Value::Text(_) => online(v, &mut ex.seen, host).map(|p: PlayerRef| EntityRef {
                uuid: p.uuid,
                kind: "player".into(),
            }),
            _ => None,
        }
    }

    /// `kill <player, entity or list of them>`.
    #[inline(never)]
    pub(crate) fn kill_any(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        for v in self.ent_targets(e, ex, host)? {
            match &v {
                Value::Entity(r) => host.entity_kill(r),
                _ => {
                    if let Some(p) = online(&v, &mut ex.seen, host) {
                        host.kill(&p);
                    }
                }
            }
        }
        Ok(())
    }

    #[inline(never)]
    pub(crate) fn ent_cond(&mut self, c: &EntCond, ex: &mut Exec, host: &mut dyn Host) -> Result<bool, String> {
        let v = self.eval(&c.subject, ex, host)?;
        let is_kind = match &v {
            Value::Player(_) => c.kind == "player",
            Value::Entity(e) => e.kind == c.kind,
            _ => false,
        };
        Ok(is_kind != c.negate)
    }

    /// Entities (and players) of a kind near a point, with their distance.
    fn gather(
        &mut self,
        kind: &Option<String>,
        center: Option<&FxPos>,
        radius: f64,
        host: &mut dyn Host,
    ) -> Vec<(Value, f64)> {
        let mut out = Vec::new();
        if kind.as_deref().is_none_or(|k| k == "player") {
            for p in host.online_players() {
                let Some((x, y, z, w)) = host.player_location(&p) else {
                    continue;
                };
                let d = match center {
                    Some(c) => {
                        if c.world.as_deref().is_some_and(|cw| !cw.eq_ignore_ascii_case(&w)) {
                            continue;
                        }
                        dist((x, y, z), c)
                    }
                    None => 0.0,
                };
                if d <= radius {
                    out.push((Value::Player(p), d));
                }
            }
        }
        if kind.as_deref() != Some("player") {
            let here = center.cloned().unwrap_or(FxPos {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                world: None,
            });
            for e in host.entities_near(&here, radius, kind.as_deref()) {
                let d = match (center, host.entity_location(&e)) {
                    (Some(c), Some((x, y, z, _))) => dist((x, y, z), c),
                    _ => 0.0,
                };
                out.push((Value::Entity(e), d));
            }
        }
        out
    }

    #[inline(never)]
    pub(crate) fn ent_expr(&mut self, e: &EntExpr, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        Ok(match e {
            EntExpr::LastSpawned => ex.last_entity.clone().map_or(Value::None, Value::Entity),
            EntExpr::All { kind, radius, at } => {
                let center = match at {
                    Some(a) => match self.ent_pos(a, ex, host)? {
                        Some(p) => Some(p),
                        None => return Ok(Value::List(Vec::new())),
                    },
                    None => None,
                };
                let r = self.ent_num(radius, UNLIMITED, UNLIMITED, ex, host)?.max(0.0);
                Value::List(self.gather(kind, center.as_ref(), r, host).into_iter().map(|(v, _)| v).collect())
            }
            EntExpr::Nearest { kind, to, radius } => {
                let skip = self.eval(to, ex, host)?;
                let skip = handle(&skip).map(|h| h.uuid);
                let Some(center) = self.ent_pos(to, ex, host)? else {
                    return Ok(Value::None);
                };
                let r = self.ent_num(radius, UNLIMITED, UNLIMITED, ex, host)?.max(0.0);
                let mut best: Option<(Value, f64)> = None;
                for (v, d) in self.gather(kind, Some(&center), r, host) {
                    if handle(&v).map(|h| h.uuid) == skip && skip.is_some() {
                        continue;
                    }
                    if best.as_ref().is_none_or(|(_, bd)| d < *bd) {
                        best = Some((v, d));
                    }
                }
                best.map_or(Value::None, |(v, _)| v)
            }
            EntExpr::Count(list) => match self.eval(list, ex, host)? {
                Value::List(items) => Value::Num(items.len() as f64),
                _ => Value::Num(0.0),
            },
            EntExpr::TypeOf(x) => match self.eval(x, ex, host)? {
                Value::Entity(e) => Value::Text(e.kind),
                Value::Player(_) => Value::Text("player".into()),
                _ => Value::None,
            },
            EntExpr::Vel(axis, x) => {
                let v = self.eval(x, ex, host)?;
                let Some(r) = self.live_handle(&v, ex, host) else {
                    return Ok(Value::None);
                };
                match host.entity_velocity(&r) {
                    Some((a, b, c)) => Value::Num([a, b, c].get(usize::from(*axis)).copied().unwrap_or(0.0)),
                    None => Value::None,
                }
            }
        })
    }

    /// Properties of an entity that need the host. `None`: use the shared path.
    pub(crate) fn entity_prop(&mut self, prop: crate::expr::Prop, e: &EntityRef, host: &mut dyn Host) -> Option<Value> {
        use crate::expr::Prop;
        Some(match prop {
            Prop::Uuid => Value::Text(e.uuid.clone()),
            Prop::Name => host.entity_name(e).map_or(Value::None, Value::Text),
            Prop::Health => host.entity_health(e).map_or(Value::None, Value::Num),
            _ => return None,
        })
    }
}
