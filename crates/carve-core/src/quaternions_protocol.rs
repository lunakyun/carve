//! Wave 6, chunk 6: rows that were marked missing but the API (or plain
//! logic) can do.
//!
//! - `plain <item>`, `<n> times`, `w of <quaternion>`, `rotation angle|axis of
//!   <quaternion>`, `<vector|quaternion> rotated around <axis> by <degrees>`.
//! - `portal cooldown of <entity>`, `protocol version [of <player>]`,
//!   `vehicle of`, `passengers of`, `villager level|profession|experience of`,
//!   `ringing time of <bell>`, `whitelist enabled`, `whitelisted players`.
//! - `sign text of`, `skull owner|texture of`, `spawner type of` and
//!   `loot table seed of` read the block entity's NBT.
//! - `set portal cooldown ...`, `set whitelist to on|off`, `set yaw|pitch of ...`,
//!   `set x|y|z|w of {var} to ...`, `add|remove ... world border size`,
//!   `set spawn of <world> to <location>`.
//!
//! Every expression is one boxed [`Node`] (`Expr::RandomTernary`), every change one boxed
//! [`QuatEffect`] (`Effect::RandomTernary`). Both run out of line (`#[cold]`).

use crate::compile::Effect;
use crate::engine::{Engine, Exec, online};
use crate::expr::{Env, Expr, P, Ty, parse_expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::skbee_nbt::{Compound, Tag};
use crate::value::{EntityRef, PlayerRef, Value};

/// The protocol number of the Minecraft version this server speaks.
pub const SERVER_PROTOCOL: f64 = 777.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vf {
    Level,
    Profession,
    Experience,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Plain,
    Times,
    QuatW,
    RotAngle,
    RotAxis,
    Portal,
    /// With an argument: that player's client; without: the server's.
    Protocol,
    Vehicle,
    Passengers,
    Villager(Vf),
    Ringing,
    WhitelistOn,
    WhitelistPlayers,
    SignText,
    SkullOwner,
    SkullTexture,
    SpawnerType,
    LootSeed,
    /// `<vector|quaternion> rotated around <axis> by <degrees>`.
    Rotated,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub op: Op,
    pub args: Vec<Expr>,
    pub ty: Ty,
}

fn node(op: Op, args: Vec<Expr>, ty: Ty) -> Expr {
    Expr::Quaternions(Box::new(Node { op, args, ty }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comp {
    X,
    Y,
    Z,
    W,
    Yaw,
    Pitch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Set,
    Add,
    Remove,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuatEffect {
    /// `set x of {v} to 3`, `set yaw of player to 10`.
    Comp {
        comp: Comp,
        target: Expr,
        change: Change,
        value: Expr,
    },
    Portal {
        target: Expr,
        value: Expr,
    },
    Whitelist {
        value: Expr,
    },
    Border {
        world: Option<Expr>,
        change: Change,
        value: Expr,
    },
    Spawn {
        world: Expr,
        value: Expr,
    },
}

// ---- host interface ---------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub enum Get<'a> {
    /// Ticks until the entity can use a portal again.
    Portal(&'a EntityRef),
    /// Protocol number of a Java player's client.
    Protocol(&'a PlayerRef),
    Vehicle(&'a EntityRef),
    Passengers(&'a EntityRef),
    /// Villager level, experience (numbers) or profession (text).
    Villager(&'a EntityRef, Vf),
    /// Ticks a bell has been ringing.
    Ringing(&'a str, i32, i32, i32),
    Whitelist,
    WhitelistPlayers,
}

#[derive(Debug, Clone, Copy)]
pub enum Set<'a> {
    Portal(&'a EntityRef, u32),
    Whitelist(bool),
    /// Yaw and pitch; `None` keeps the current one.
    Rotation(&'a EntityRef, Option<f32>, Option<f32>),
}

// ---- pure logic --------------------------------------------------------------

/// Quaternion rotation angle in degrees.
pub fn quat_angle(w: f64) -> f64 {
    2.0 * w.clamp(-1.0, 1.0).acos().to_degrees()
}

/// Quaternion rotation axis (unit length; `(0, 0, 1)` for no rotation).
pub fn quat_axis(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let n = (x * x + y * y + z * z).sqrt();
    if n < 1e-12 {
        (0.0, 0.0, 1.0)
    } else {
        (x / n, y / n, z / n)
    }
}

/// Rodrigues rotation of a vector around an axis, by `deg` degrees.
pub fn rotate_vec(v: (f64, f64, f64), axis: (f64, f64, f64), deg: f64) -> (f64, f64, f64) {
    let (ax, ay, az) = quat_axis(axis.0, axis.1, axis.2);
    let (s, c) = deg.to_radians().sin_cos();
    let dot = ax * v.0 + ay * v.1 + az * v.2;
    let cross = (ay * v.2 - az * v.1, az * v.0 - ax * v.2, ax * v.1 - ay * v.0);
    (
        v.0 * c + cross.0 * s + ax * dot * (1.0 - c),
        v.1 * c + cross.1 * s + ay * dot * (1.0 - c),
        v.2 * c + cross.2 * s + az * dot * (1.0 - c),
    )
}

/// `q * r` where `r` is a rotation around `axis` by `deg` degrees.
pub fn rotate_quat(q: (f64, f64, f64, f64), axis: (f64, f64, f64), deg: f64) -> (f64, f64, f64, f64) {
    let (ax, ay, az) = quat_axis(axis.0, axis.1, axis.2);
    let (s, c) = (deg.to_radians() / 2.0).sin_cos();
    let (rw, rx, ry, rz) = (c, ax * s, ay * s, az * s);
    let (w, x, y, z) = q;
    (
        w * rw - x * rx - y * ry - z * rz,
        w * rx + x * rw + y * rz - z * ry,
        w * ry - x * rz + y * rw + z * rx,
        w * rz + x * ry - y * rx + z * rw,
    )
}

/// `<n> times` as the list 1..n.
pub fn times_list(n: f64) -> Value {
    let n = n.floor().clamp(0.0, 1_000_000.0) as i64;
    Value::List((1..=n).map(|i| Value::Num(i as f64)).collect())
}

const PROTOCOLS: &[(&str, u32)] = &[
    ("1.7.2", 4),
    ("1.7.6", 5),
    ("1.8", 47),
    ("1.9", 107),
    ("1.9.1", 108),
    ("1.9.2", 109),
    ("1.9.3", 110),
    ("1.10", 210),
    ("1.11", 315),
    ("1.11.1", 316),
    ("1.12", 335),
    ("1.12.1", 338),
    ("1.12.2", 340),
    ("1.13", 393),
    ("1.13.1", 401),
    ("1.13.2", 404),
    ("1.14", 477),
    ("1.14.1", 480),
    ("1.14.2", 485),
    ("1.14.3", 490),
    ("1.14.4", 498),
    ("1.15", 573),
    ("1.15.1", 575),
    ("1.15.2", 578),
    ("1.16", 735),
    ("1.16.1", 736),
    ("1.16.2", 751),
    ("1.16.3", 753),
    ("1.16.4", 754),
    ("1.17", 755),
    ("1.17.1", 756),
    ("1.18", 757),
    ("1.18.2", 758),
    ("1.19", 759),
    ("1.19.1", 760),
    ("1.19.3", 761),
    ("1.19.4", 762),
    ("1.20", 763),
    ("1.20.2", 764),
    ("1.20.3", 765),
    ("1.20.5", 766),
    ("1.21", 767),
    ("1.21.2", 768),
    ("1.21.4", 769),
    ("1.21.5", 770),
    ("1.21.6", 771),
    ("1.21.7", 772),
    ("1.21.9", 773),
    ("1.21.11", 774),
    ("26.1", 775),
    ("26.2", 776),
    ("26.3", 777),
];

/// Protocol number for a Pumpkin `java-minecraft-version` variant. Only the
/// digits of its Debug name count (`V1_21_11` and `V12111` are both 1.21.11).
/// `None` for `unknown`.
pub fn protocol_of(name: &str) -> Option<u32> {
    let key: String = name.chars().filter(char::is_ascii_digit).collect();
    PROTOCOLS
        .iter()
        .find(|(v, _)| v.chars().filter(char::is_ascii_digit).eq(key.chars()))
        .map(|(_, n)| *n)
}

fn nbt_of(v: &Value, host: &mut dyn Host) -> Option<Compound> {
    let Value::Location { x, y, z, world } = v else {
        return None;
    };
    let w = match world {
        Some(w) => w.clone(),
        None => host.default_world()?,
    };
    let bytes = host.block_entity_nbt(&w, x.floor() as i32, y.floor() as i32, z.floor() as i32)?;
    Compound::from_bytes(&bytes).ok()
}

/// Plain text of one sign line: a JSON string, or a JSON object with `text`.
fn line_text(t: &Tag) -> String {
    match t {
        Tag::Str(s) => {
            let s = s.trim();
            if let Some(inner) = s.strip_prefix('"').and_then(|x| x.strip_suffix('"')) {
                return inner.replace("\\\"", "\"").replace("\\\\", "\\");
            }
            if s.starts_with('{')
                && let Some(i) = s.find("\"text\":\"")
            {
                let rest = &s[i + 8..];
                return rest.split('"').next().unwrap_or("").to_string();
            }
            s.to_string()
        }
        Tag::Compound(c) => match c.get("text") {
            Some(Tag::Str(s)) => s.clone(),
            _ => String::new(),
        },
        _ => String::new(),
    }
}

/// The four front lines of a sign from its block-entity NBT.
pub fn sign_lines(nbt: &Compound) -> Option<Vec<String>> {
    let Some(Tag::Compound(front)) = nbt.get("front_text") else {
        return None;
    };
    let Some(Tag::List(msgs)) = front.get("messages") else {
        return None;
    };
    let mut lines: Vec<String> = msgs.iter().map(line_text).collect();
    lines.resize(4, String::new());
    Some(lines)
}

/// Owner name and texture value of a skull's `profile` compound.
pub fn skull_profile(nbt: &Compound) -> (Option<String>, Option<String>) {
    let Some(Tag::Compound(p)) = nbt.get("profile") else {
        return (None, None);
    };
    let name = match p.get("name") {
        Some(Tag::Str(s)) if !s.is_empty() => Some(s.clone()),
        _ => None,
    };
    let tex = match p.get("properties") {
        Some(Tag::List(props)) => props.iter().find_map(|t| match t {
            Tag::Compound(c) if matches!(c.get("name"), Some(Tag::Str(n)) if n == "textures") => {
                match c.get("value") {
                    Some(Tag::Str(v)) => Some(v.clone()),
                    _ => None,
                }
            }
            _ => None,
        }),
        _ => None,
    };
    (name, tex)
}

/// The entity id a spawner spawns, without `minecraft:`.
pub fn spawner_type(nbt: &Compound) -> Option<String> {
    match nbt.get_path("SpawnData;entity;id") {
        Some(Tag::Str(s)) => Some(s.strip_prefix("minecraft:").unwrap_or(s).to_string()),
        _ => None,
    }
}

// ---- parsing -----------------------------------------------------------------

fn lit(toks: &[Tok], i: usize, w: &str) -> bool {
    toks.get(i).is_some_and(|t| t.is_lit(w))
}

const HEADS: &[&str] = &[
    "plain",
    "portal",
    "protocol",
    "vehicle",
    "passengers",
    "passenger",
    "villager",
    "ringing",
    "whitelist",
    "whitelisted",
    "sign",
    "lines",
    "skull",
    "spawner",
    "loot",
    "rotation",
    "w",
];

impl P<'_> {
    fn quat_lit(&self, off: usize, w: &str) -> bool {
        if self.pos + off >= self.toks.len() {
            self.hit_end.set(true);
        }
        lit(self.toks, self.pos + off, w)
    }

    /// `of <x>` operand, or `None` when no `of` follows.
    fn quat_of(&mut self) -> Result<Option<Expr>, String> {
        if !self.quat_lit(0, "of") {
            return Ok(None);
        }
        self.pos += 1;
        if self.pos >= self.toks.len() {
            self.hit_end.set(true);
            return Err("missing what to read after 'of'".into());
        }
        self.primary().map(Some)
    }

    fn quat_need(&mut self, op: Op, ty: Ty) -> Result<Option<Expr>, String> {
        match self.quat_of()? {
            Some(x) => Ok(Some(node(op, vec![x], ty))),
            None => Ok(None),
        }
    }

    /// Called with the head word read. `Ok(None)`: not ours, position kept.
    pub(crate) fn quat_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        if !HEADS.contains(&lw) {
            return Ok(None);
        }
        let start = self.pos;
        let r = self.quat_inner(lw);
        if matches!(r, Ok(None)) {
            self.pos = start;
        }
        r
    }

    fn quat_inner(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        match lw {
            "plain" => {
                if self.pos >= self.toks.len() || self.quat_lit(0, "text") {
                    return Ok(None);
                }
                let Ok(x) = self.primary() else {
                    return Ok(None);
                };
                if !matches!(x.ty(self.env), Ty::Item | Ty::Object) {
                    return Ok(None);
                }
                Ok(Some(node(Op::Plain, vec![x], Ty::Item)))
            }
            "portal" if self.quat_lit(0, "cooldown") => {
                self.pos += 1;
                self.quat_need(Op::Portal, Ty::Timespan)
            }
            "protocol" if self.quat_lit(0, "version") => {
                self.pos += 1;
                match self.quat_of()? {
                    Some(x) => Ok(Some(node(Op::Protocol, vec![x], Ty::Number))),
                    None => Ok(Some(node(Op::Protocol, vec![], Ty::Number))),
                }
            }
            "vehicle" => self.quat_need(Op::Vehicle, Ty::Object),
            "passengers" | "passenger" => self.quat_need(Op::Passengers, Ty::List),
            "villager" => {
                let (vf, ty) = if self.quat_lit(0, "level") {
                    (Vf::Level, Ty::Number)
                } else if self.quat_lit(0, "profession") {
                    (Vf::Profession, Ty::Text)
                } else if self.quat_lit(0, "experience") || self.quat_lit(0, "xp") {
                    (Vf::Experience, Ty::Number)
                } else {
                    return Ok(None);
                };
                self.pos += 1;
                self.quat_need(Op::Villager(vf), ty)
            }
            "ringing" if self.quat_lit(0, "time") => {
                self.pos += 1;
                self.quat_need(Op::Ringing, Ty::Timespan)
            }
            "whitelist"
                if self.quat_lit(0, "enabled") || self.quat_lit(0, "status") || self.quat_lit(0, "state") =>
            {
                self.pos += 1;
                Ok(Some(node(Op::WhitelistOn, vec![], Ty::Boolean)))
            }
            "whitelisted" if self.quat_lit(0, "players") => {
                self.pos += 1;
                Ok(Some(node(Op::WhitelistPlayers, vec![], Ty::List)))
            }
            "sign" if self.quat_lit(0, "text") || self.quat_lit(0, "lines") => {
                self.pos += 1;
                self.quat_need(Op::SignText, Ty::List)
            }
            "lines" => self.quat_need(Op::SignText, Ty::List),
            "skull" if self.quat_lit(0, "owner") => {
                self.pos += 1;
                self.quat_need(Op::SkullOwner, Ty::Text)
            }
            "skull" if self.quat_lit(0, "texture") => {
                self.pos += 1;
                self.quat_need(Op::SkullTexture, Ty::Text)
            }
            "spawner" if self.quat_lit(0, "type") => {
                self.pos += 1;
                self.quat_need(Op::SpawnerType, Ty::Text)
            }
            "loot" if self.quat_lit(0, "table") && self.quat_lit(1, "seed") => {
                self.pos += 2;
                self.quat_need(Op::LootSeed, Ty::Number)
            }
            "rotation" if self.quat_lit(0, "angle") => {
                self.pos += 1;
                self.quat_need(Op::RotAngle, Ty::Number)
            }
            "rotation" if self.quat_lit(0, "axis") => {
                self.pos += 1;
                self.quat_need(Op::RotAxis, Ty::Vector)
            }
            "w" if self.quat_lit(0, "of") || self.quat_lit(0, "component") => {
                if self.quat_lit(0, "component") {
                    self.pos += 1;
                }
                self.quat_need(Op::QuatW, Ty::Number)
            }
            _ => Ok(None),
        }
    }

    /// `<n> times` right after a number (end of the expression, not `3 times 4`).
    pub(crate) fn quat_times(&mut self, n: f64) -> Option<Expr> {
        if !self.quat_lit(0, "times") {
            return None;
        }
        // Only where a list can start: `to 3 times`, `(3 times)`, `, 3 times`, a line start.
        // Not `"ab" repeated 3 times` or `x + 1 times`.
        let before = self.pos.checked_sub(2).and_then(|i| self.toks.get(i));
        if !before.is_none_or(|t| t.is_lit("to") || t.is_lit("in") || t.is_lit("(") || t.is_lit(",")) {
            return None;
        }
        // Only a bare `N times`: anything that continues like an operand is left alone.
        if let Some(Tok::Num(_) | Tok::Var(_) | Tok::Str(_)) = self.toks.get(self.pos + 1) {
            return None;
        }
        self.pos += 1;
        Some(node(Op::Times, vec![Expr::Num(n)], Ty::List))
    }

    /// After a vector or quaternion: `rotated around <axis> by <degrees>`.
    pub(crate) fn quat_post(&mut self, e: Expr) -> Result<Expr, String> {
        if !(self.quat_lit(0, "rotated") && self.quat_lit(1, "around")) {
            return Ok(e);
        }
        self.pos += 2;
        let axis = match self.toks.get(self.pos) {
            Some(Tok::Word(w)) if matches!(w.to_ascii_lowercase().as_str(), "x" | "y" | "z") => {
                let name = w.to_ascii_lowercase();
                self.pos += 1;
                if self.quat_lit(0, "axis") {
                    self.pos += 1;
                }
                Expr::Text(vec![crate::expr::Part::Lit(name)])
            }
            _ => self.primary()?,
        };
        if !self.quat_lit(0, "by") {
            return Err("expected 'rotated around <axis> by <degrees>'".into());
        }
        self.pos += 1;
        let by = self.primary()?;
        let ty = e.ty(self.env);
        Ok(node(Op::Rotated, vec![e, axis, by], ty))
    }
}

pub fn is_list_expr(toks: &[Tok]) -> bool {
    let i = usize::from(lit(toks, 0, "the"));
    (lit(toks, i, "passengers") || lit(toks, i, "passenger") || lit(toks, i, "lines"))
        && lit(toks, i + 1, "of")
        || (lit(toks, i, "whitelisted") && lit(toks, i + 1, "players"))
        || (lit(toks, i, "sign")
            && (lit(toks, i + 1, "text") || lit(toks, i + 1, "lines"))
            && lit(toks, i + 2, "of"))
        || (toks.len() == i + 2 && matches!(toks.get(i), Some(Tok::Num(_))) && lit(toks, i + 1, "times"))
}

// ---- effects ---------------------------------------------------------------------

fn comp_of(w: &str) -> Option<Comp> {
    Some(match w.to_ascii_lowercase().as_str() {
        "x" => Comp::X,
        "y" => Comp::Y,
        "z" => Comp::Z,
        "w" => Comp::W,
        "yaw" => Comp::Yaw,
        "pitch" => Comp::Pitch,
        _ => return None,
    })
}

fn expr_or_word(v: &[Tok], env: &Env) -> Result<Expr, String> {
    match (parse_expr(v, env), v) {
        (Ok(e), _) => Ok(e),
        (Err(_), [Tok::Word(w)]) => Ok(Expr::Text(vec![crate::expr::Part::Lit(w.to_ascii_lowercase())])),
        (Err(m), _) => Err(m),
    }
}

/// `Some`: this line is one of ours (maybe wrong). `None`: not ours.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Option<Result<Effect, String>> {
    let verb = match toks.first() {
        Some(Tok::Word(w)) => w.to_ascii_lowercase(),
        _ => return None,
    };
    let (change, target, value) = match verb.as_str() {
        "set" => {
            let to = toks.iter().position(|t| t.is_lit("to"))?;
            (Change::Set, toks.get(1..to)?, toks.get(to + 1..)?)
        }
        "add" | "remove" => {
            let kw = if verb == "add" { "to" } else { "from" };
            let at = toks.iter().rposition(|t| t.is_lit(kw))?;
            let ch = if verb == "add" {
                Change::Add
            } else {
                Change::Remove
            };
            (ch, toks.get(at + 1..)?, toks.get(1..at)?)
        }
        _ => return None,
    };
    if target.is_empty() || value.is_empty() {
        return None;
    }
    let ti = usize::from(lit(target, 0, "the"));
    let fx = fx_of(change, target, ti, value, env);
    fx.map(|r| r.map(|f| Effect::Quaternions(Box::new(f))))
}

fn fx_of(
    change: Change,
    t: &[Tok],
    i: usize,
    value: &[Tok],
    env: &Env,
) -> Option<Result<QuatEffect, String>> {
    let first = match t.get(i) {
        Some(Tok::Word(w)) => w.to_ascii_lowercase(),
        _ => return None,
    };
    let val = |e: &Env| expr_or_word(value, e);
    // `set portal cooldown of X to 5 seconds`
    if first == "portal" && lit(t, i + 1, "cooldown") && lit(t, i + 2, "of") {
        let tg = parse_expr(t.get(i + 3..)?, env).ok()?;
        return Some(val(env).map(|value| QuatEffect::Portal { target: tg, value }));
    }
    if first == "whitelist" && change == Change::Set {
        let rest = t.get(i + 1..)?;
        if rest.is_empty()
            || (rest.len() == 1 && ["enabled", "status", "state"].iter().any(|w| rest[0].is_lit(w)))
        {
            return Some(val(env).map(|value| QuatEffect::Whitelist { value }));
        }
        return None;
    }
    // `add 5 to world border size [of W]`, also `set` (left to `server_misc`).
    if first == "world" && lit(t, i + 1, "border") && (lit(t, i + 2, "size") || lit(t, i + 2, "diameter")) {
        if change == Change::Set {
            return None;
        }
        let world = match t.get(i + 3..) {
            Some([]) | None => None,
            Some(r) if lit(r, 0, "of") => parse_expr(r.get(1..)?, env).ok(),
            _ => return None,
        };
        return Some(val(env).map(|value| QuatEffect::Border { world, change, value }));
    }
    if first == "spawn" && change == Change::Set && lit(t, i + 1, "of") {
        let w = parse_expr(t.get(i + 2..)?, env).ok()?;
        return Some(val(env).map(|value| QuatEffect::Spawn { world: w, value }));
    }
    if let Some(comp) = comp_of(&first) {
        let at = i + 1;
        let mut j = at;
        if lit(t, j, "component") || lit(t, j, "coordinate") {
            j += 1;
        }
        if !lit(t, j, "of") {
            return None;
        }
        let rest = t.get(j + 1..)?;
        let target = parse_expr(rest, env).ok()?;
        // Only variables (vectors, quaternions) and the rotation of entities.
        let ok = match comp {
            Comp::Yaw | Comp::Pitch => true,
            _ => matches!(target, Expr::Var(_)),
        };
        if !ok {
            return None;
        }
        // Yaw and pitch of a variable that holds a vector are changed in place too.
        return Some(val(env).map(|value| QuatEffect::Comp {
            comp,
            target,
            change,
            value,
        }));
    }
    None
}

// ---- running -----------------------------------------------------------------------

fn handle(v: &Value, ex: &mut Exec, host: &mut dyn Host) -> Option<EntityRef> {
    match v {
        Value::Entity(r) => Some(r.clone()),
        Value::Player(_) | Value::Text(_) => online(v, &mut ex.seen, host).map(|p| EntityRef {
            uuid: p.uuid,
            kind: "player".into(),
        }),
        _ => None,
    }
}

fn vec3(v: &Value) -> Option<(f64, f64, f64)> {
    match v {
        Value::Text(t) => match t.as_str() {
            "x" => Some((1.0, 0.0, 0.0)),
            "y" => Some((0.0, 1.0, 0.0)),
            "z" => Some((0.0, 0.0, 1.0)),
            _ => None,
        },
        Value::Vector { x, y, z } => Some((*x, *y, *z)),
        Value::Quaternion { x, y, z, .. } => Some((*x, *y, *z)),
        _ => None,
    }
}

pub fn quat_comp(prop: crate::expr::Prop, v: &Value) -> Option<Value> {
    use crate::expr::Prop;
    let Value::Quaternion { w: _, x, y, z } = v else {
        return None;
    };
    Some(Value::Num(match prop {
        Prop::X => *x,
        Prop::Y => *y,
        Prop::Z => *z,
        _ => return None,
    }))
}

impl Engine {
    #[cold]
    #[inline(never)]
    pub(crate) fn eval_quaternions(
        &mut self,
        n: &Node,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        let mut a = Vec::with_capacity(n.args.len());
        for e in &n.args {
            a.push(self.eval(e, ex, host)?);
        }
        let a0 = a.first().cloned().unwrap_or(Value::None);
        Ok(read(n.op, &a0, &a, ex, host).unwrap_or(Value::None))
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn exec_quaternions(
        &mut self,
        f: &QuatEffect,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        match f {
            QuatEffect::Portal { target, value } => {
                let t = self.eval(target, ex, host)?;
                let v = self.eval(value, ex, host)?;
                let ticks = match v {
                    Value::Timespan { millis } => millis as f64 / 50.0,
                    Value::Num(n) => n,
                    other => {
                        return Err(format!(
                            "expected a timespan for the portal cooldown, got {}",
                            other.type_name()
                        ));
                    }
                };
                if let Some(e) = handle(&t, ex, host) {
                    host.protocol_set(Set::Portal(&e, ticks.max(0.0).round() as u32));
                }
            }
            QuatEffect::Whitelist { value } => {
                let v = self.eval(value, ex, host)?;
                let on = match &v {
                    Value::Bool(b) => *b,
                    Value::Text(t) => match t.to_ascii_lowercase().as_str() {
                        "on" | "true" | "yes" | "enabled" => true,
                        "off" | "false" | "no" | "disabled" => false,
                        _ => return Err(format!("expected on or off for the whitelist, got '{t}'")),
                    },
                    other => {
                        return Err(format!(
                            "expected on or off for the whitelist, got {}",
                            other.type_name()
                        ));
                    }
                };
                host.protocol_set(Set::Whitelist(on));
            }
            QuatEffect::Border { world, change, value } => {
                let w = match world {
                    Some(e) => match self.eval(e, ex, host)? {
                        Value::World(w) | Value::Text(w) => Some(w),
                        _ => None,
                    },
                    None => host.default_world(),
                };
                let Some(w) = w else {
                    return Ok(());
                };
                let d = match self.eval(value, ex, host)? {
                    Value::Num(n) => n,
                    other => {
                        return Err(format!(
                            "expected a number to change the border by, got {}",
                            other.type_name()
                        ));
                    }
                };
                let cur = host.border_size(&w).unwrap_or(0.0);
                let new = match change {
                    Change::Remove => cur - d,
                    _ => cur + d,
                };
                host.set_border_size(&w, new.clamp(1.0, 59_999_968.0), 0.0);
            }
            QuatEffect::Spawn { world, value } => {
                let w = match self.eval(world, ex, host)? {
                    Value::World(w) | Value::Text(w) => w,
                    other => return Err(format!("expected a world, got {}", other.type_name())),
                };
                let Value::Location { x, y, z, .. } = self.eval(value, ex, host)? else {
                    return Err("expected a location to set the spawn to".into());
                };
                if host.default_world().is_some_and(|d| d != w) {
                    return Err("only the spawn of the main world can be set".into());
                }
                // The plugin API v0.1 can read the world spawn but not set it.
                host.execute_command(&format!(
                    "setworldspawn {} {} {}",
                    x.floor() as i64,
                    y.floor() as i64,
                    z.floor() as i64
                ));
            }
            QuatEffect::Comp {
                comp,
                target,
                change,
                value,
            } => {
                let t = self.eval(target, ex, host)?;
                let v = match self.eval(value, ex, host)? {
                    Value::Num(n) => n,
                    other => return Err(format!("expected a number, got {}", other.type_name())),
                };
                let apply = |cur: f64| match change {
                    Change::Set => v,
                    Change::Add => cur + v,
                    Change::Remove => cur - v,
                };
                match (&t, comp) {
                    (Value::Vector { x, y, z }, Comp::X | Comp::Y | Comp::Z) => {
                        let (mut nx, mut ny, mut nz) = (*x, *y, *z);
                        match comp {
                            Comp::X => nx = apply(nx),
                            Comp::Y => ny = apply(ny),
                            _ => nz = apply(nz),
                        }
                        self.quat_store(target, Value::Vector { x: nx, y: ny, z: nz }, ex, host)?;
                    }
                    (Value::Quaternion { w, x, y, z }, Comp::X | Comp::Y | Comp::Z | Comp::W) => {
                        let (mut nw, mut nx, mut ny, mut nz) = (*w, *x, *y, *z);
                        match comp {
                            Comp::W => nw = apply(nw),
                            Comp::X => nx = apply(nx),
                            Comp::Y => ny = apply(ny),
                            _ => nz = apply(nz),
                        }
                        self.quat_store(
                            target,
                            Value::Quaternion {
                                w: nw,
                                x: nx,
                                y: ny,
                                z: nz,
                            },
                            ex,
                            host,
                        )?;
                    }
                    (Value::Vector { x, y, z }, Comp::Yaw | Comp::Pitch) => {
                        let len = (x * x + y * y + z * z).sqrt();
                        let yaw = (-x).atan2(*z).to_degrees();
                        let pitch = (-y).atan2((x * x + z * z).sqrt()).to_degrees();
                        let (yaw, pitch) = if *comp == Comp::Yaw {
                            (apply(yaw), pitch)
                        } else {
                            (yaw, apply(pitch))
                        };
                        let (yr, pr) = (yaw.to_radians(), pitch.to_radians());
                        let nv = Value::Vector {
                            x: -pr.cos() * yr.sin() * len,
                            y: -pr.sin() * len,
                            z: pr.cos() * yr.cos() * len,
                        };
                        self.quat_store(target, nv, ex, host)?;
                    }
                    (_, Comp::Yaw | Comp::Pitch) => {
                        let Some(e) = handle(&t, ex, host) else {
                            return Ok(());
                        };
                        let cur = match (&t, comp) {
                            (_, Comp::Yaw) => host.entity_rotation(&e).map(|r| r.0),
                            _ => host.entity_rotation(&e).map(|r| r.1),
                        };
                        let new = apply(cur.unwrap_or(0.0)) as f32;
                        if *comp == Comp::Yaw {
                            host.protocol_set(Set::Rotation(&e, Some(new), None));
                        } else {
                            host.protocol_set(Set::Rotation(&e, None, Some(new.clamp(-90.0, 90.0))));
                        }
                    }
                    _ => return Err(format!("can't change the {comp:?} of {}", t.type_name())),
                }
            }
        }
        Ok(())
    }

    fn quat_store(
        &mut self,
        target: &Expr,
        new: Value,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        let Expr::Var(var) = target else {
            return Err("a vector can only be changed in a variable, like {_v}".into());
        };
        let k = self.key(var, ex, host)?.into_owned();
        self.store(var.local, &k, new, ex, host);
        Ok(())
    }
}

fn read(op: Op, a: &Value, all: &[Value], ex: &mut Exec, host: &mut dyn Host) -> Option<Value> {
    match op {
        Op::Plain => match a {
            Value::Item { id, count, .. } => Some(Value::Item {
                id: id.clone(),
                count: *count,
                meta: None,
            }),
            _ => None,
        },
        Op::Times => a.as_num().map(times_list),
        Op::QuatW => match a {
            Value::Quaternion { w, .. } => Some(Value::Num(*w)),
            _ => None,
        },
        Op::RotAngle => match a {
            Value::Quaternion { w, .. } => Some(Value::Num(quat_angle(*w))),
            _ => None,
        },
        Op::RotAxis => match a {
            Value::Quaternion { x, y, z, .. } => {
                let (x, y, z) = quat_axis(*x, *y, *z);
                Some(Value::Vector { x, y, z })
            }
            _ => None,
        },
        Op::Rotated => {
            let axis = vec3(all.get(1)?)?;
            let deg = all.get(2)?.as_num()?;
            match a {
                Value::Vector { x, y, z } => {
                    let (x, y, z) = rotate_vec((*x, *y, *z), axis, deg);
                    Some(Value::Vector { x, y, z })
                }
                Value::Quaternion { w, x, y, z } => {
                    let (w, x, y, z) = rotate_quat((*w, *x, *y, *z), axis, deg);
                    Some(Value::Quaternion { w, x, y, z })
                }
                _ => None,
            }
        }
        Op::Portal => {
            let e = handle(a, ex, host)?;
            match host.protocol_get(Get::Portal(&e))? {
                Value::Num(t) => Some(Value::Timespan {
                    millis: (t * 50.0) as i64,
                }),
                v => Some(v),
            }
        }
        Op::Protocol => {
            if matches!(a, Value::None) {
                return Some(Value::Num(SERVER_PROTOCOL));
            }
            let p = online(a, &mut ex.seen, host)?;
            host.protocol_get(Get::Protocol(&p))
        }
        Op::Vehicle => {
            let e = handle(a, ex, host)?;
            host.protocol_get(Get::Vehicle(&e))
        }
        Op::Passengers => {
            let e = handle(a, ex, host)?;
            host.protocol_get(Get::Passengers(&e))
        }
        Op::Villager(f) => {
            let e = handle(a, ex, host)?;
            host.protocol_get(Get::Villager(&e, f))
        }
        Op::Ringing => {
            let Value::Location { x, y, z, world } = a else {
                return None;
            };
            let w = match world {
                Some(w) => w.clone(),
                None => host.default_world()?,
            };
            match host.protocol_get(Get::Ringing(
                &w,
                x.floor() as i32,
                y.floor() as i32,
                z.floor() as i32,
            ))? {
                Value::Num(t) => Some(Value::Timespan {
                    millis: (t * 50.0) as i64,
                }),
                v => Some(v),
            }
        }
        Op::WhitelistOn => host.protocol_get(Get::Whitelist),
        Op::WhitelistPlayers => host.protocol_get(Get::WhitelistPlayers),
        Op::SignText => {
            let nbt = nbt_of(a, host)?;
            Some(Value::List(
                sign_lines(&nbt)?.into_iter().map(Value::Text).collect(),
            ))
        }
        Op::SkullOwner => skull_profile(&nbt_of(a, host)?).0.map(Value::Text),
        Op::SkullTexture => skull_profile(&nbt_of(a, host)?).1.map(Value::Text),
        Op::SpawnerType => spawner_type(&nbt_of(a, host)?).map(Value::Text),
        Op::LootSeed => match nbt_of(a, host)?.get("LootTableSeed") {
            Some(Tag::Long(n)) => Some(Value::Num(*n as f64)),
            _ => None,
        },
    }
}
