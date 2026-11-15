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

/// Walk the ray block by block (voxel traversal) and stop at the first block
/// that is not air. Fluids are skipped unless the request includes them.
pub fn walk_blocks<H: Host + ?Sized>(req: &RayReq, host: &mut H) -> RayRaw {
    use crate::block_ops::BlockPos;
    let mut raw = RayRaw::default();
    if !req.blocks {
        return raw;
    }
    let (sx, sy, sz) = req.start;
    let d = (req.end.0 - sx, req.end.1 - sy, req.end.2 - sz);
    let len = (d.0 * d.0 + d.1 * d.1 + d.2 * d.2).sqrt();
    if !len.is_finite() || len < 1e-9 {
        return raw;
    }
    let dir = [d.0 / len, d.1 / len, d.2 / len];
    let o = [sx, sy, sz];
    let mut cell = [sx.floor() as i32, sy.floor() as i32, sz.floor() as i32];
    let mut step = [0i32; 3];
    let mut t_max = [f64::INFINITY; 3];
    let mut t_delta = [f64::INFINITY; 3];
    for a in 0..3 {
        if dir[a] > 0.0 {
            step[a] = 1;
            t_delta[a] = 1.0 / dir[a];
            t_max[a] = (f64::from(cell[a]) + 1.0 - o[a]) / dir[a];
        } else if dir[a] < 0.0 {
            step[a] = -1;
            t_delta[a] = -1.0 / dir[a];
            t_max[a] = (o[a] - f64::from(cell[a])) / -dir[a];
        }
    }
    let mut t = 0.0;
    let mut face_axis = 1;
    for _ in 0..(MAX_RAY_DISTANCE as usize * 3 + 3) {
        if t > len {
            break;
        }
        let at = BlockPos {
            x: cell[0],
            y: cell[1],
            z: cell[2],
            world: req.world.clone(),
        };
        if let Some(id) = host.block_type(&at) {
            let solid = match id.as_str() {
                "air" | "cave_air" | "void_air" | "light" => false,
                "water" | "lava" | "bubble_column" => req.fluids,
                _ => true,
            };
            if solid {
                let face = match (face_axis, step[face_axis]) {
                    (0, s) if s > 0 => "west",
                    (0, _) => "east",
                    (1, s) if s > 0 => "down",
                    (1, _) => "up",
                    (_, s) if s > 0 => "north",
                    _ => "south",
                };
                let tt = t.min(len);
                raw.block = Some(RawBlock {
                    pos: (cell[0], cell[1], cell[2]),
                    face: face.to_string(),
                    hit: (o[0] + dir[0] * tt, o[1] + dir[1] * tt, o[2] + dir[2] * tt),
                });
                return raw;
            }
        }
        let a = if t_max[0] <= t_max[1] && t_max[0] <= t_max[2] {
            0
        } else if t_max[1] <= t_max[2] {
            1
        } else {
            2
        };
        if !t_max[a].is_finite() {
            break;
        }
        t = t_max[a];
        t_max[a] += t_delta[a];
        cell[a] += step[a];
        face_axis = a;
    }
    raw
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
            Value::Player(PlayerRef {
                uuid: e.uuid.clone(),
                name,
            })
        } else {
            Value::Entity(EntityRef {
                uuid: e.uuid.clone(),
                kind: e.kind.clone(),
            })
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
    host.resolve_player(&p.uuid).map(|name| PlayerRef {
        uuid: p.uuid.clone(),
        name,
    })
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
    use crate::expr_vectors_server::Query;
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
    if len.is_nan() || len <= 1e-9 || !max.is_finite() || max <= 0.0 {
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
    Value::Location {
        x: f64::from(p.0),
        y: f64::from(p.1),
        z: f64::from(p.2),
        world,
    }
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
            let max = if f.dist {
                vals.get(i).and_then(num)
            } else {
                Some(DEFAULT_RAY_DISTANCE)
            };
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
            let Value::Ray(h) = first else {
                return Ok(Value::None);
            };
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
            let world = if has_world {
                vals.get(2).and_then(world_name)
            } else {
                None
            };
            chunk_value(world, x.floor() as i32, z.floor() as i32)
        }
        Op::ChunksAround => {
            let Some(r) = num(first) else {
                return Ok(Value::None);
            };
            let r = r.clamp(0.0, MAX_CHUNK_RADIUS) as i32;
            let centre = match vals.get(1) {
                Some(Value::Chunk { world, x, z }) => Some((world.clone(), *x, *z)),
                Some(v) => loc_of(v, host).map(|(x, _, z, w)| (w, chunk_floor(x), chunk_floor(z))),
                None => None,
            };
            let Some((world, cx, cz)) = centre else {
                return Ok(Value::None);
            };
            let mut out = Vec::with_capacity(((2 * r + 1) * (2 * r + 1)) as usize);
            for dz in -r..=r {
                for dx in -r..=r {
                    out.push(chunk_value(world.clone(), cx + dx, cz + dz));
                }
            }
            Value::List(out)
        }
        Op::TagMembers(kind) => {
            let Some(name) = text_of(first) else {
                return Ok(Value::None);
            };
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
                    .map(|n| Value::Item {
                        id: n.to_string(),
                        count: 1,
                        meta: None,
                    })
                    .collect(),
            )
        }
        Op::TagsOf => {
            let Some((id, block)) = subject_id(first, host) else {
                return Ok(Value::None);
            };
            let mut out: Vec<Value> = Vec::new();
            let tables: &[&[(&str, &str)]] = match block {
                Some(true) => &[crate::data::tags::BLOCK_TAGS],
                Some(false) => &[crate::data::tags::ITEM_TAGS],
                None => &[crate::data::tags::ITEM_TAGS, crate::data::tags::BLOCK_TAGS],
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
    let t = raw
        .trim()
        .trim_start_matches('#')
        .to_ascii_lowercase()
        .replace(' ', "_");
    if t.contains(':') {
        t
    } else {
        format!("minecraft:{t}")
    }
}

fn lookup(table: &'static [(&'static str, &'static str)], tag: &str) -> Option<&'static str> {
    table
        .binary_search_by(|(k, _)| (*k).cmp(tag))
        .ok()
        .map(|i| table[i].1)
}

/// Space separated member names of a tag. `Any` tries items first, then blocks.
pub fn tag_members(kind: TagKind, raw: &str) -> Option<&'static str> {
    let tag = normalize_tag(raw);
    let items = || lookup(crate::data::tags::ITEM_TAGS, &tag);
    let blocks = || lookup(crate::data::tags::BLOCK_TAGS, &tag);
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
            let at = BlockPos {
                x: x.floor() as i32,
                y: y.floor() as i32,
                z: z.floor() as i32,
                world: world.clone(),
            };
            host.block_type(&at)
                .map(|id| (id.trim_start_matches("minecraft:").to_string(), Some(true)))
        }
        Value::Item { id, .. } => Some((id.clone(), None)),
        Value::Text(t) => {
            let id = t
                .trim()
                .trim_start_matches("minecraft:")
                .to_ascii_lowercase()
                .replace(' ', "_");
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
        "%object% is tagged (as|with) [a] [the] [minecraft] [(block|item)] tag[s] %text%",
        ck(CKind::Tagged, false),
    ),
    (
        "%object% (is not|isn't) tagged (as|with) [a] [the] [minecraft] [(block|item)] tag[s] %text%",
        ck(CKind::Tagged, true),
    ),
    ("%object% is tagged (as|with) %text%", ck(CKind::Tagged, false)),
    (
        "%object% (is not|isn't) tagged (as|with) %text%",
        ck(CKind::Tagged, true),
    ),
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
            let Some(tag) = b.and_then(text_of) else {
                return false;
            };
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
    RtwCond {
        kind,
        negate,
        a,
        b: if kind_has_b(kind) { b } else { None },
    }
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
        "create [a] [new] world [named] %text% with environment (normal|overworld)",
        R(EOp::Create(Env::Normal)),
    ),
    (
        "create [a] [new] world [named] %text% with environment nether",
        R(EOp::Create(Env::Nether)),
    ),
    (
        "create [a] [new] world [named] %text% with environment end",
        R(EOp::Create(Env::End)),
    ),
    (
        "create [a] [new] world [named] %text%",
        R(EOp::Create(Env::Normal)),
    ),
    ("load world [named] %text%", R(EOp::Create(Env::Normal))),
    ("unload world [named] %text%", R(EOp::Unload)),
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
        && n.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    if ok {
        Ok(n.to_string())
    } else {
        Err(format!(
            "'{raw}' is not a valid world name (letters, digits, - _ . only, no folders)"
        ))
    }
}

pub fn run_effect(e: &RtwEffect, name: &Value, host: &mut dyn Host) -> Result<(), String> {
    let Some(raw) = world_name(name) else {
        return Err(format!(
            "expected a world name, got {} ({})",
            name.type_name(),
            name
        ));
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
            return Err(format!(
                "ray trace needs a player, entity or location, got {}",
                from.ty(self.env).name()
            ));
        }
        let mut f = RayFlags {
            dir: false,
            dist: false,
            fluids: false,
            blocks: true,
            entities: true,
        };
        let mut dir = None;
        let mut dist = None;
        loop {
            if self.r_lit(0, "and")
                && (self.r_lit(1, "with") || self.r_lit(1, "ignoring") || self.r_lit(1, "including"))
            {
                self.pos += 1;
            }
            if self.r_lit(0, "in")
                && (self.r_lit(1, "direction") || (self.r_lit(1, "the") && self.r_lit(2, "direction")))
            {
                self.pos += if self.r_lit(1, "the") { 3 } else { 2 };
                if self.r_lit(0, "of") {
                    self.pos += 1;
                }
                let v = self.primary()?;
                if !matches!(v.ty(self.env), Ty::Vector | Ty::Object) {
                    return Err(format!(
                        "'in direction' needs a vector, got {}",
                        v.ty(self.env).name()
                    ));
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
                // A flag word cut off at the end of the span means a longer span may parse.
                if self.pos + 1 >= self.toks.len() {
                    self.hit_end.set(true);
                }
                let target = match self.toks.get(self.pos + 1) {
                    Some(t) if t.is_lit("fluids") || t.is_lit("fluid") => 0,
                    Some(t) if t.is_lit("blocks") || t.is_lit("block") => 1,
                    Some(t) if t.is_lit("entities") || t.is_lit("entity") => 2,
                    _ => {
                        return Err("expected 'ignoring fluids|blocks|entities' or 'including fluids'".into());
                    }
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
            (Some(a), _) if a.is_lit("position") || a.is_lit("location") || a.is_lit("point") => {
                (HitField::Position, 1)
            }
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

/// Cheap pre-check: can any of our condition patterns match this line? They
/// all begin with a slot, so trying them on every line is slow.
pub fn line_may_match(toks: &[crate::lexer::Tok]) -> bool {
    toks.iter()
        .any(|t| t.is_lit("tagged") || t.is_lit("loaded") || t.is_lit("unloaded"))
}

/// Does this token run start one of our list expressions (`chunks in radius`,
/// `minecraft tag "x"`, `minecraft tags of x`)?
pub fn is_list_expr(toks: &[crate::lexer::Tok]) -> bool {
    let lit = |i: usize, w: &str| toks.get(i).is_some_and(|t| t.is_lit(w));
    if lit(0, "chunks") {
        return lit(1, "in") || lit(1, "around");
    }
    if lit(0, "minecraft") || lit(0, "vanilla") {
        let skip = usize::from(lit(1, "block") || lit(1, "item"));
        return lit(1 + skip, "tags") || lit(1 + skip, "tag");
    }
    false
}
