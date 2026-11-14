//! SkBee-style ray tracing, worlds, chunks and Minecraft tags.
//!
//! - `ray trace from player with max distance 50`, then `hit block of`,
//!   `hit position of`, `hit entity of`, `hit face of`, `hit distance of`.
//! - `create world "x" with environment nether`, `load world "x"`,
//!   `unload world "x"`, `world "x" is loaded`.
//! - `chunk at <location>`, `chunk at 3, 4 in world "w"`, `chunk x of <chunk>`,
//!   `chunks in radius 2 around player`, `<chunk> is loaded`.
//! - `block is tagged with minecraft tag "logs"`, `minecraft tag "logs"`,
//!   `minecraft tags of <block>`.
//!
//! Expressions are one boxed [`Node`] (`Expr::Rtw`), effects and conditions
//! are boxed too, so the hot enums stay small. The engine evaluates the
//! arguments and calls [`apply`] / [`check`].

use crate::block_ops::BlockPos;
use crate::compile::{CondKind, EffKind, Effect};
use crate::expr::{Expr, P, Ty};
use crate::host::Host;
use crate::pattern::Capture;
use crate::value::{EntityRef, PlayerRef, Value};

/// How far a ray goes when the script gives no `max distance`.
pub const DEFAULT_RAY_DISTANCE: f64 = 20.0;
/// Longest ray, in blocks.
pub const MAX_RAY_DISTANCE: f64 = 512.0;
/// Largest `chunks in radius` radius.
pub const MAX_CHUNK_RADIUS: f64 = 16.0;

// ---- ray tracing ------------------------------------------------------------

/// A ray to cast through one world.
#[derive(Debug, Clone, PartialEq)]
pub struct RayReq {
    /// `None`: the default world.
    pub world: Option<String>,
    pub start: (f64, f64, f64),
    pub end: (f64, f64, f64),
    pub fluids: bool,
    pub blocks: bool,
    pub entities: bool,
    /// Entity uuid the ray must not hit (its own origin).
    pub exclude: Option<String>,
}

/// A block the host's ray ended on.
#[derive(Debug, Clone, PartialEq)]
pub struct RawBlock {
    pub pos: (i32, i32, i32),
    /// `up`, `down`, `north`, `south`, `east`, `west`.
    pub face: String,
    pub hit: (f64, f64, f64),
}

/// An entity the host's ray ended on.
#[derive(Debug, Clone, PartialEq)]
pub struct RawEntity {
    pub uuid: String,
    /// Entity type id without `minecraft:`; `player` for players.
    pub kind: String,
    pub hit: (f64, f64, f64),
}

/// What the host found along a ray: the first block and the first entity.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RayRaw {
    pub block: Option<RawBlock>,
    pub entity: Option<RawEntity>,
}

/// The result of `ray trace ...`, kept in a variable.
#[derive(Debug, Clone, PartialEq)]
pub struct RayHit {
    pub world: Option<String>,
    /// The block hit; `None` when the ray ended on an entity.
    pub block: Option<(i32, i32, i32)>,
    pub position: (f64, f64, f64),
    pub entity: Option<Value>,
    pub face: Option<String>,
    pub distance: f64,
}

fn dist(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2) + (a.2 - b.2).powi(2)).sqrt()
}

/// Pick the nearer of the block and entity hits.
pub fn pick_hit(req: &RayReq, raw: RayRaw, host: &mut dyn Host) -> Option<RayHit> {
    let bd = raw.block.as_ref().map(|b| dist(req.start, b.hit));
    let ed = raw.entity.as_ref().map(|e| dist(req.start, e.hit));
    let entity_wins = match (bd, ed) {
        (Some(b), Some(e)) => e < b,
        (None, Some(_)) => true,
        _ => false,
    };
    if entity_wins {
        let e = raw.entity?;
        let value = if e.kind == "player" {
            let name = host.resolve_player(&e.uuid).unwrap_or_default();
            Value::Player(PlayerRef { uuid: e.uuid.clone(), name })
        } else {
            Value::Entity(EntityRef { uuid: e.uuid.clone(), kind: e.kind.clone() })
        };
        return Some(RayHit {
            world: req.world.clone(),
            block: None,
            position: e.hit,
            entity: Some(value),
            face: None,
            distance: dist(req.start, e.hit),
        });
    }
    let b = raw.block?;
    Some(RayHit {
        world: req.world.clone(),
        block: Some(b.pos),
        position: b.hit,
        entity: None,
        face: Some(b.face),
        distance: dist(req.start, b.hit),
    })
}

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Num(n) if n.is_finite() => Some(*n),
        _ => None,
    }
}

fn loc_of(v: &Value, host: &mut dyn Host) -> Option<(f64, f64, f64, Option<String>)> {
    match v {
        Value::Location { x, y, z, world } => Some((*x, *y, *z, world.clone())),
        Value::Player(p) => {
            let p = live_player(p, host)?;
            host.player_location(&p).map(|(x, y, z, w)| (x, y, z, Some(w)))
        }
        Value::Entity(e) => host.entity_location(e).map(|(x, y, z, w)| (x, y, z, Some(w))),
        _ => None,
    }
}

fn live_player(p: &PlayerRef, host: &mut dyn Host) -> Option<PlayerRef> {
    host.resolve_player(&p.uuid).map(|name| PlayerRef { uuid: p.uuid.clone(), name })
}

fn vec_of(v: &Value) -> Option<(f64, f64, f64)> {
    match v {
        Value::Vector { x, y, z } if x.is_finite() && y.is_finite() && z.is_finite() => Some((*x, *y, *z)),
        _ => None,
    }
}

/// Unit look direction from Minecraft yaw and pitch (degrees).
pub fn look_dir(yaw: f64, pitch: f64) -> (f64, f64, f64) {
    let (y, p) = (yaw.to_radians(), pitch.to_radians());
    (-y.sin() * p.cos(), -p.sin(), y.cos() * p.cos())
}

/// Build the ray for `ray trace from <origin> ...`. `None` when the origin has
/// no position, or has no direction (locations and entities need `in direction`).
fn build_ray(
    from: &Value,
    dir: Option<(f64, f64, f64)>,
    max: f64,
    flags: RayFlags,
    host: &mut dyn Host,
) -> Option<RayReq> {
    use crate::w2_expressions_7::Query;
    let (start, world, exclude, dir) = match from {
        Value::Player(p) => {
            let p = live_player(p, host)?;
            let (x, y, z, w) = host.eye_location(&p)?;
            let dir = match dir {
                Some(d) => d,
                None => {
                    let yaw = num(&host.w2e7_query(Query::Yaw(&p))?)?;
                    let pitch = num(&host.w2e7_query(Query::Pitch(&p))?)?;
                    look_dir(yaw, pitch)
                }
            };
            ((x, y, z), Some(w), Some(p.uuid), dir)
        }
        Value::Entity(e) => {
            let (x, y, z, w) = host.entity_location(e)?;
            ((x, y, z), Some(w), Some(e.uuid.clone()), dir?)
        }
        Value::Location { x, y, z, world } => ((*x, *y, *z), world.clone(), None, dir?),
        _ => return None,
    };
    let len = (dir.0 * dir.0 + dir.1 * dir.1 + dir.2 * dir.2).sqrt();
    if !(len > 1e-9) || !max.is_finite() || max <= 0.0 {
        return None;
    }
    let max = max.min(MAX_RAY_DISTANCE);
    let k = max / len;
    Some(RayReq {
        world,
        start,
        end: (start.0 + dir.0 * k, start.1 + dir.1 * k, start.2 + dir.2 * k),
        fluids: flags.fluids,
        blocks: flags.blocks,
        entities: flags.entities,
        exclude,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RayFlags {
    pub dir: bool,
    pub dist: bool,
    pub fluids: bool,
    pub blocks: bool,
    pub entities: bool,
}

/// What `hit ... of <ray>` reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitField {
    Block,
    Position,
    Entity,
    Face,
    Distance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagKind {
    Any,
    Block,
    Item,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Ray(RayFlags),
    Hit(HitField),
    /// `chunk at <location>`, `chunk of <player>`
    ChunkAt,
    /// `chunk at x, z [in world]`
    ChunkXz(bool),
    /// `chunks in radius n around <location|chunk>`
    ChunksAround,
    /// `minecraft tag "logs"`
    TagMembers(TagKind),
    /// `minecraft tags of <block|item>`
    TagsOf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub op: Op,
    pub args: Vec<Expr>,
    pub ty: Ty,
}

fn node(op: Op, args: Vec<Expr>, ty: Ty) -> Expr {
    Expr::Rtw(Box::new(Node { op, args, ty }))
}

// ---- evaluation -------------------------------------------------------------

fn loc_value(p: (i32, i32, i32), world: Option<String>) -> Value {
    Value::Location { x: f64::from(p.0), y: f64::from(p.1), z: f64::from(p.2), world }
}

fn chunk_floor(c: f64) -> i32 {
    (c / 16.0).floor().clamp(-2_000_000.0, 2_000_000.0) as i32
}

pub fn chunk_value(world: Option<String>, x: i32, z: i32) -> Value {
    Value::Chunk { world, x, z }
}

fn world_name(v: &Value) -> Option<String> {
    match v {
        Value::World(w) => Some(w.clone()),
        Value::Text(t) if !t.trim().is_empty() => Some(t.trim().to_string()),
        _ => None,
    }
}

/// Run one expression of this module on its evaluated arguments.
pub fn apply(op: Op, vals: &[Value], host: &mut dyn Host) -> Result<Value, String> {
    let first = vals.first().unwrap_or(&Value::None);
    Ok(match op {
        Op::Ray(f) => {
            let mut i = 1;
            let dir = if f.dir {
                i += 1;
                vals.get(i - 1).and_then(vec_of)
            } else {
                None
            };
            if f.dir && dir.is_none() {
                return Ok(Value::None);
            }
            let max = if f.dist { vals.get(i).and_then(num) } else { Some(DEFAULT_RAY_DISTANCE) };
            let Some(max) = max else { return Ok(Value::None) };
            let Some(req) = build_ray(first, dir, max, f, host) else {
                return Ok(Value::None);
            };
            if !req.blocks && !req.entities {
                return Ok(Value::None);
            }
            let raw = host.ray_trace(&req);
            match pick_hit(&req, raw, host) {
                Some(h) => Value::Ray(Box::new(h)),
                None => Value::None,
            }
        }
        Op::Hit(field) => {
            let Value::Ray(h) = first else { return Ok(Value::None) };
            match field {
                HitField::Block => h.block.map_or(Value::None, |b| loc_value(b, h.world.clone())),
                HitField::Position => Value::Location {
                    x: h.position.0,
                    y: h.position.1,
                    z: h.position.2,
                    world: h.world.clone(),
                },
                HitField::Entity => h.entity.clone().unwrap_or(Value::None),
                HitField::Face => h.face.clone().map_or(Value::None, Value::Text),
                HitField::Distance => Value::Num(h.distance),
            }
        }
        Op::ChunkAt => match first {
            Value::Chunk { .. } => first.clone(),
            v => match loc_of(v, host) {
                Some((x, _, z, w)) => chunk_value(w, chunk_floor(x), chunk_floor(z)),
                None => Value::None,
            },
        },
        Op::ChunkXz(has_world) => {
            let (Some(x), Some(z)) = (num(first), vals.get(1).and_then(num)) else {
                return Ok(Value::None);
            };
            let world = if has_world { vals.get(2).and_then(world_name) } else { None };
            chunk_value(world, x.floor() as i32, z.floor() as i32)
        }
        Op::ChunksAround => {
            let Some(r) = num(first) else { return Ok(Value::None) };
            let r = r.clamp(0.0, MAX_CHUNK_RADIUS) as i32;
            let centre = match vals.get(1) {
                Some(Value::Chunk { world, x, z }) => Some((world.clone(), *x, *z)),
                Some(v) => loc_of(v, host).map(|(x, _, z, w)| (w, chunk_floor(x), chunk_floor(z))),
                None => None,
            };
            let Some((world, cx, cz)) = centre else { return Ok(Value::None) };
            let mut out = Vec::with_capacity(((2 * r + 1) * (2 * r + 1)) as usize);
            for dz in -r..=r {
                for dx in -r..=r {
                    out.push(chunk_value(world.clone(), cx + dx, cz + dz));
                }
            }
            Value::List(out)
        }
        Op::TagMembers(kind) => {
            let Some(name) = text_of(first) else { return Ok(Value::None) };
            let Some(names) = tag_members(kind, &name) else {
                host.log(
                    crate::host::LogLevel::Warn,
                    &format!("unknown minecraft tag '{}'", name.trim()),
                );
                return Ok(Value::List(Vec::new()));
            };
            Value::List(
                names
                    .split(' ')
                    .filter(|n| !n.is_empty())
                    .map(|n| Value::Item { id: n.to_string(), count: 1, meta: None })
                    .collect(),
            )
        }
        Op::TagsOf => {
            let Some((id, block)) = subject_id(first, host) else { return Ok(Value::None) };
            let mut out: Vec<Value> = Vec::new();
            let tables: &[&[(&str, &str)]] = match block {
                Some(true) => &[crate::w4_tags_data::BLOCK_TAGS],
                Some(false) => &[crate::w4_tags_data::ITEM_TAGS],
                None => &[crate::w4_tags_data::ITEM_TAGS, crate::w4_tags_data::BLOCK_TAGS],
            };
            for t in tables {
                for (k, members) in *t {
                    if members.split(' ').any(|m| m == id) {
                        let k = Value::Text((*k).to_string());
                        if !out.contains(&k) {
                            out.push(k);
                        }
                    }
                }
            }
            Value::List(out)
        }
    })
}

fn text_of(v: &Value) -> Option<String> {
    match v {
        Value::Text(t) => Some(t.clone()),
        _ => None,
    }
}

// ---- tags -------------------------------------------------------------------

/// `Logs`, `#logs`, `minecraft:logs` all name `minecraft:logs`.
pub fn normalize_tag(raw: &str) -> String {
    let t = raw.trim().trim_start_matches('#').to_ascii_lowercase().replace(' ', "_");
    if t.contains(':') { t } else { format!("minecraft:{t}") }
}

fn lookup(table: &'static [(&'static str, &'static str)], tag: &str) -> Option<&'static str> {
    table.binary_search_by(|(k, _)| (*k).cmp(tag)).ok().map(|i| table[i].1)
}

/// Space separated member names of a tag. `Any` tries items first, then blocks.
pub fn tag_members(kind: TagKind, raw: &str) -> Option<&'static str> {
    let tag = normalize_tag(raw);
    let items = || lookup(crate::w4_tags_data::ITEM_TAGS, &tag);
    let blocks = || lookup(crate::w4_tags_data::BLOCK_TAGS, &tag);
    match kind {
        TagKind::Item => items(),
        TagKind::Block => blocks(),
        TagKind::Any => items().or_else(blocks),
    }
}

/// The id of a thing that can be tagged, and whether it is a block
/// (`Some(true)`), an item (`Some(false)`) or unknown (`None`).
fn subject_id(v: &Value, host: &mut dyn Host) -> Option<(String, Option<bool>)> {
    match v {
        Value::Location { x, y, z, world } => {
            let at = BlockPos { x: x.floor() as i32, y: y.floor() as i32, z: z.floor() as i32, world: world.clone() };
            host.block_type(&at).map(|id| (id.trim_start_matches("minecraft:").to_string(), Some(true)))
        }
        Value::Item { id, .. } => Some((id.clone(), None)),
        Value::Text(t) => {
            let id = t.trim().trim_start_matches("minecraft:").to_ascii_lowercase().replace(' ', "_");
            Some((id, None))
        }
        _ => None,
    }
}

/// Is the block or item tagged with this tag? `None`: the tag does not exist
/// or the subject is not a block or item.
pub fn is_tagged(subject: &Value, tag: &str, host: &mut dyn Host) -> Option<bool> {
    let (id, kind) = subject_id(subject, host)?;
    let members = match kind {
        Some(true) => tag_members(TagKind::Block, tag).or_else(|| tag_members(TagKind::Item, tag)),
        _ => tag_members(TagKind::Any, tag),
    }?;
    Some(members.split(' ').any(|m| m == id))
}

// ---- conditions -------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CKind {
    Tagged,
    Loaded,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RtwCond {
    pub kind: CKind,
    pub negate: bool,
    pub a: Expr,
    pub b: Option<Expr>,
}

const fn ck(kind: CKind, negate: bool) -> CondKind {
    CondKind::Rtw(kind, negate)
}

/// Tried before the plain `is` comparison.
pub const CONDITION_PATTERNS: &[(&str, CondKind)] = &[
    (
        "%object% is tagged (as|with) [a] [the] [minecraft] [block|item] tag[s] %text%",
        ck(CKind::Tagged, false),
    ),
    (
        "%object% (is not|isn't) tagged (as|with) [a] [the] [minecraft] [block|item] tag[s] %text%",
        ck(CKind::Tagged, true),
    ),
    ("%object% is tagged (as|with) %text%", ck(CKind::Tagged, false)),
    ("%object% (is not|isn't) tagged (as|with) %text%", ck(CKind::Tagged, true)),
    ("%object% is loaded", ck(CKind::Loaded, false)),
    ("%object% (is not|isn't) loaded", ck(CKind::Loaded, true)),
    ("%object% is unloaded", ck(CKind::Loaded, true)),
];

pub fn kind_has_b(k: CKind) -> bool {
    k == CKind::Tagged
}

/// Run a condition on its evaluated subject and tag or world.
pub fn check(c: &RtwCond, a: &Value, b: Option<&Value>, host: &mut dyn Host) -> bool {
    let hit = match c.kind {
        CKind::Tagged => {
            let Some(tag) = b.and_then(text_of) else { return false };
            match is_tagged(a, &tag, host) {
                Some(h) => h,
                None => {
                    if matches!(a, Value::Location { .. } | Value::Item { .. } | Value::Text(_)) {
                        host.log(
                            crate::host::LogLevel::Warn,
                            &format!("unknown minecraft tag '{}'", tag.trim()),
                        );
                    }
                    return false;
                }
            }
        }
        CKind::Loaded => is_loaded(a, host),
    };
    hit != c.negate
}

fn is_loaded(v: &Value, host: &mut dyn Host) -> bool {
    match v {
        Value::World(w) => host.world_loaded(w),
        Value::Chunk { world, x, z } => {
            let w = world.clone().or_else(|| host.default_world());
            w.is_some_and(|w| host.chunk_loaded(&w, *x, *z) == Some(true))
        }
        Value::Location { x, z, world, .. } => {
            let w = world.clone().or_else(|| host.default_world());
            w.is_some_and(|w| host.chunk_loaded(&w, chunk_floor(*x), chunk_floor(*z)) == Some(true))
        }
        Value::Text(t) => host.world_loaded(t.trim()),
        _ => false,
    }
}

pub fn build_cond(kind: CKind, negate: bool, a: Expr, b: Option<Expr>) -> RtwCond {
    RtwCond { kind, negate, a, b: if kind_has_b(kind) { b } else { None } }
}

// ---- effects ----------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Env {
    Normal,
    Nether,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EOp {
    /// `create world "x" [with environment nether]` and `load world "x"`.
    Create(Env),
    Unload,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RtwEffect {
    pub op: EOp,
    pub name: Expr,
}

use EffKind::Rtw as R;

pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    (
        "create [a] [new] world [named|called] %text% with [the] (environment|dimension) (normal|overworld)",
        R(EOp::Create(Env::Normal)),
    ),
    (
        "create [a] [new] world [named|called] %text% with [the] (environment|dimension) (nether|the nether)",
        R(EOp::Create(Env::Nether)),
    ),
    (
        "create [a] [new] world [named|called] %text% with [the] (environment|dimension) (end|the end)",
        R(EOp::Create(Env::End)),
    ),
    ("create [a] [new] world [named|called] %text%", R(EOp::Create(Env::Normal))),
    ("load [the] world [named|called] %text%", R(EOp::Create(Env::Normal))),
    ("unload [the] world [named|called] %text%", R(EOp::Unload)),
];

pub fn build(op: EOp, caps: &mut [Option<Capture>]) -> Option<Effect> {
    let name = match caps.get_mut(0).and_then(Option::take) {
        Some(Capture::Expr(e)) => e,
        _ => return None,
    };
    Some(Effect::Rtw(Box::new(RtwEffect { op, name })))
}

/// A world name that is safe to create: letters, digits, `_ - .`, no folders.
pub fn clean_world_name(raw: &str) -> Result<String, String> {
    let n = raw.trim();
    let ok = !n.is_empty()
        && n.len() <= 64
        && !n.starts_with('.')
        && n.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    if ok {
        Ok(n.to_string())
    } else {
        Err(format!("'{raw}' is not a valid world name (letters, digits, - _ . only, no folders)"))
    }
}

pub fn run_effect(e: &RtwEffect, name: &Value, host: &mut dyn Host) -> Result<(), String> {
    let Some(raw) = world_name(name) else {
        return Err(format!("expected a world name, got {} ({})", name.type_name(), name));
    };
    match e.op {
        EOp::Create(env) => {
            let n = clean_world_name(&raw)?;
            host.create_world(&n, env)
        }
        EOp::Unload => host.unload_world(raw.trim()),
    }
}

// ---- parsing ----------------------------------------------------------------

impl P<'_> {
    fn r_lit(&self, off: usize, w: &str) -> bool {
        if self.pos + off >= self.toks.len() {
            self.hit_end.set(true);
        }
        matches!(self.toks.get(self.pos + off), Some(t) if t.is_lit(w))
    }

    fn r_num(&mut self, what: &str) -> Result<Expr, String> {
        let e = self.sum()?;
        if !Ty::Number.accepts(e.ty(self.env)) {
            return Err(format!("{what} needs a number"));
        }
        Ok(e)
    }

    fn r_ray(&mut self) -> Result<Expr, String> {
        let usage = "'ray trace from <player|location> [in direction <vector>] [with max distance <n>]'";
        if self.r_lit(0, "from") {
            self.pos += 1;
        }
        let from = self.primary().map_err(|e| format!("{usage}: {e}"))?;
        if !matches!(from.ty(self.env), Ty::Player | Ty::Location | Ty::Object) {
            return Err(format!("ray trace needs a player, entity or location, got {}", from.ty(self.env).name()));
        }
        let mut f = RayFlags { dir: false, dist: false, fluids: false, blocks: true, entities: true };
        let mut dir = None;
        let mut dist = None;
        loop {
            if self.r_lit(0, "and") && (self.r_lit(1, "with") || self.r_lit(1, "ignoring") || self.r_lit(1, "including")) {
                self.pos += 1;
            }
            if self.r_lit(0, "in") && (self.r_lit(1, "direction") || (self.r_lit(1, "the") && self.r_lit(2, "direction"))) {
                self.pos += if self.r_lit(1, "the") { 3 } else { 2 };
                if self.r_lit(0, "of") {
                    self.pos += 1;
                }
                let v = self.primary()?;
                if !matches!(v.ty(self.env), Ty::Vector | Ty::Object) {
                    return Err(format!("'in direction' needs a vector, got {}", v.ty(self.env).name()));
                }
                f.dir = true;
                dir = Some(v);
            } else if self.r_lit(0, "with")
                && (self.r_lit(1, "max") || self.r_lit(1, "maximum"))
                && self.r_lit(2, "distance")
            {
                self.pos += 3;
                dist = Some(self.r_num("the max distance")?);
                f.dist = true;
            } else if self.r_lit(0, "with") && self.r_lit(1, "ray") && self.r_lit(2, "size") {
                return Err("'with ray size' is not supported: Pumpkin only traces thin rays".into());
            } else if self.r_lit(0, "ignoring") || self.r_lit(0, "including") {
                let ignore = self.r_lit(0, "ignoring");
                let target = match self.toks.get(self.pos + 1) {
                    Some(t) if t.is_lit("fluids") || t.is_lit("fluid") => 0,
                    Some(t) if t.is_lit("blocks") || t.is_lit("block") => 1,
                    Some(t) if t.is_lit("entities") || t.is_lit("entity") => 2,
                    _ => return Err("expected 'ignoring fluids|blocks|entities' or 'including fluids'".into()),
                };
                self.pos += 2;
                match (target, ignore) {
                    (0, ig) => f.fluids = !ig,
                    (1, ig) => f.blocks = !ig,
                    (_, ig) => f.entities = !ig,
                }
            } else {
                break;
            }
        }
        let mut args = vec![from];
        args.extend(dir);
        args.extend(dist);
        Ok(node(Op::Ray(f), args, Ty::Object))
    }

    fn r_hit(&mut self) -> Result<Option<Expr>, String> {
        let (field, n) = match (self.toks.get(self.pos), self.toks.get(self.pos + 1)) {
            (Some(a), Some(b)) if a.is_lit("block") && b.is_lit("face") => (HitField::Face, 2),
            (Some(a), _) if a.is_lit("block") => (HitField::Block, 1),
            (Some(a), _) if a.is_lit("position") || a.is_lit("location") || a.is_lit("point") => (HitField::Position, 1),
            (Some(a), _) if a.is_lit("entity") => (HitField::Entity, 1),
            (Some(a), _) if a.is_lit("face") => (HitField::Face, 1),
            (Some(a), _) if a.is_lit("distance") => (HitField::Distance, 1),
            _ => return Ok(None),
        };
        if !self.r_lit(n, "of") {
            return Ok(None);
        }
        self.pos += n + 1;
        let ray = self.primary()?;
        let ty = match field {
            HitField::Block | HitField::Position => Ty::Location,
            HitField::Entity => Ty::Object,
            HitField::Face => Ty::Text,
            HitField::Distance => Ty::Number,
        };
        Ok(Some(node(Op::Hit(field), vec![ray], ty)))
    }

    fn r_chunk(&mut self) -> Result<Option<Expr>, String> {
        if !(self.r_lit(0, "at") || self.r_lit(0, "of")) {
            return Ok(None);
        }
        let at = self.r_lit(0, "at");
        self.pos += 1;
        let first = self.sum()?;
        let t = first.ty(self.env);
        if at && self.r_lit(0, ",") {
            if !Ty::Number.accepts(t) {
                return Err("'chunk at x, z' needs numbers".into());
            }
            self.pos += 1;
            let z = self.r_num("the chunk z")?;
            let mut args = vec![first, z];
            let mut world = false;
            if self.r_lit(0, "in") {
                self.pos += 1;
                let w = self.primary()?;
                args.push(w);
                world = true;
            }
            return Ok(Some(node(Op::ChunkXz(world), args, Ty::Object)));
        }
        if !matches!(t, Ty::Location | Ty::Player | Ty::Object) {
            return Err(format!("'chunk at' needs a location, got {}", t.name()));
        }
        Ok(Some(node(Op::ChunkAt, vec![first], Ty::Object)))
    }

    fn r_chunks(&mut self) -> Result<Option<Expr>, String> {
        if !(self.r_lit(0, "in") && self.r_lit(1, "radius") || self.r_lit(0, "around")) {
            return Ok(None);
        }
        let usage = "'chunks in radius <n> around <location|chunk>'";
        let radius;
        if self.r_lit(0, "around") {
            self.pos += 1;
            let centre = self.primary()?;
            if !self.r_lit(0, "in") || !self.r_lit(1, "radius") {
                return Err(format!("expected {usage}"));
            }
            self.pos += 2;
            radius = self.r_num("the radius")?;
            return Ok(Some(node(Op::ChunksAround, vec![radius, centre], Ty::List)));
        }
        self.pos += 2;
        radius = self.r_num("the radius")?;
        if !(self.r_lit(0, "around") || self.r_lit(0, "of")) {
            return Err(format!("expected {usage}"));
        }
        self.pos += 1;
        let centre = self.primary()?;
        Ok(Some(node(Op::ChunksAround, vec![radius, centre], Ty::List)))
    }

    fn r_tag(&mut self, kind: TagKind) -> Result<Expr, String> {
        // positioned on `tag`
        self.pos += 1;
        let name = self.primary()?;
        if !Ty::Text.accepts(name.ty(self.env)) {
            return Err("a tag name must be text, like \"logs\"".into());
        }
        Ok(node(Op::TagMembers(kind), vec![name], Ty::List))
    }

    /// Called with the head word read. `Ok(None)`: not one of ours.
    pub(crate) fn rtw_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let e = match lw {
            "ray" if self.r_lit(0, "trace") => {
                self.pos += 1;
                self.r_ray()?
            }
            "raytrace" => self.r_ray()?,
            "hit" => match self.r_hit()? {
                Some(e) => e,
                None => return Ok(None),
            },
            "chunk" => match self.r_chunk()? {
                Some(e) => e,
                None => return Ok(None),
            },
            "chunks" => match self.r_chunks()? {
                Some(e) => e,
                None => return Ok(None),
            },
            "minecraft" | "vanilla" => {
                let kind = if self.r_lit(0, "block") {
                    Some(TagKind::Block)
                } else if self.r_lit(0, "item") {
                    Some(TagKind::Item)
                } else {
                    None
                };
                let skip = usize::from(kind.is_some());
                if self.r_lit(skip, "tag") {
                    self.pos += skip;
                    self.r_tag(kind.unwrap_or(TagKind::Any))?
                } else if skip == 0 && self.r_lit(0, "tags") && self.r_lit(1, "of") {
                    self.pos += 2;
                    let of = self.primary()?;
                    node(Op::TagsOf, vec![of], Ty::List)
                } else {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(e))
    }
}
