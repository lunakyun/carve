//! Vectors, server facts (`tps`, versions, view distance), player play time,
//! yaw and pitch, `environment of`, `N of item`
//! and `whether a is b`.
//!
//! Every expression here is one boxed [`Node`] (`Expr::VectorsServer`), so the hot
//! `Expr` and `eval` paths stay small. The engine evaluates the node's
//! arguments and calls [`apply`].

use std::sync::atomic::{AtomicU64, Ordering};

use crate::expr::{BinOp, Expr, P, Prop, Ty};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::{PlayerRef, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmp {
    Is,
    IsNot,
    Gt,
    Lt,
    Ge,
    Le,
    IsSet,
    NotSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// `vector(x, y, z)`
    New,
    Neg,
    Arith(BinOp),
    Len,
    LenSq,
    Norm,
    Dot,
    Cross,
    Angle,
    Project,
    Yaw,
    Pitch,
    FromYawPitch,
    /// Vector of a location or player position.
    FromLoc,
    /// Vector to a location, with an optional world.
    ToLoc,
    /// Location moved by a vector.
    Offset,
    Between,
    Random,
    Spherical,
    Cylindrical,
    /// Direction a player looks in.
    LookDir,
    /// Unit vector of a compass word: 0 north, 1 south, 2 east, 3 west, 4 up, 5 down.
    Compass(u8),
    Tps,
    Mspt,
    McVersion,
    ServerViewDistance,
    ClientViewDistance,
    TimePlayed,
    TimeLived,
    Environment,
    ItemCount,
    Whether(Cmp),
}

/// One expression of this module: what to do, the argument expressions, and
/// the type of the result.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub op: Op,
    pub args: Vec<Expr>,
    pub ty: Ty,
}

/// A question for the server that the core can't answer itself.
#[derive(Debug, Clone, Copy)]
pub enum Query<'a> {
    Tps,
    Mspt,
    MinecraftVersion,
    ServerViewDistance,
    /// Play time statistic, in ticks.
    PlayTicks(&'a PlayerRef),
    /// Ticks the player entity has existed.
    LivedTicks(&'a PlayerRef),
    Yaw(&'a PlayerRef),
    Pitch(&'a PlayerRef),
    /// Dimension id of a world, like `minecraft:the_nether`.
    Environment(&'a str),
}

fn node(op: Op, args: Vec<Expr>, ty: Ty) -> Expr {
    Expr::VectorsServer(Box::new(Node { op, args, ty }))
}

// ---- parsing ---------------------------------------------------------------

/// `-v` for a vector.
pub fn neg(inner: Expr) -> Expr {
    node(Op::Neg, vec![inner], Ty::Vector)
}

/// `a + b`, `a * 2`, ... where one side is a vector. Checks the types.
pub fn vec_bin(l: Expr, op: BinOp, r: Expr, lt: Ty, rt: Ty) -> Result<Expr, String> {
    let obj = |t: Ty| t == Ty::Object;
    let ty = match (lt, op, rt) {
        (Ty::Vector, _, Ty::Vector) => Ty::Vector,
        (Ty::Vector, BinOp::Mul | BinOp::Div, t) if Ty::Number.accepts(t) => Ty::Vector,
        (t, BinOp::Mul, Ty::Vector) if Ty::Number.accepts(t) => Ty::Vector,
        (Ty::Location, BinOp::Add | BinOp::Sub, Ty::Vector) => Ty::Location,
        (Ty::Vector, BinOp::Add, Ty::Location) => Ty::Location,
        (Ty::Vector, _, t) | (t, _, Ty::Vector) if obj(t) => Ty::Object,
        _ => {
            return Err(format!(
                "can't do {} and {} with that operator (vectors take + - between vectors, and * / with a number)",
                lt.name(),
                rt.name()
            ));
        }
    };
    Ok(node(Op::Arith(op), vec![l, r], ty))
}

impl P<'_> {
    fn w_lit(&self, off: usize, w: &str) -> bool {
        if self.pos + off >= self.toks.len() {
            self.hit_end.set(true);
        }
        matches!(self.toks.get(self.pos + off), Some(t) if t.is_lit(w))
    }

    fn w_expect(&mut self, w: &str, usage: &str) -> Result<(), String> {
        if self.w_lit(0, w) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("expected '{w}' in {usage}"))
        }
    }

    fn w_num(&mut self, what: &str) -> Result<Expr, String> {
        let e = self.sum()?;
        if !Ty::Number.accepts(e.ty(self.env)) {
            return Err(format!("{what} needs a number"));
        }
        Ok(e)
    }

    fn w_sep(&mut self) {
        if self.w_lit(0, ",") || self.w_lit(0, "and") {
            self.pos += 1;
        }
    }

    fn w_vec(&mut self, what: &str) -> Result<Expr, String> {
        let e = self.primary()?;
        match e.ty(self.env) {
            Ty::Vector | Ty::Object => Ok(e),
            t => Err(format!("{what} needs a vector, got {}", t.name())),
        }
    }

    /// `version [string] of [the] minecraft` from here.
    fn w_mc_version_ahead(&self) -> bool {
        let mut i = 0;
        if self.w_lit(i, "string") {
            i += 1;
        }
        if !self.w_lit(i, "of") {
            return false;
        }
        i += 1;
        if self.w_lit(i, "the") {
            i += 1;
        }
        self.w_lit(i, "minecraft")
    }

    fn w_of(&mut self, usage: &str) -> Result<(), String> {
        self.w_expect("of", usage)
    }

    /// Three numbers `x, y, z` after the first one is already parsed.
    fn w_xyz(&mut self, x: Expr) -> Result<Expr, String> {
        self.w_sep();
        let y = self.w_num("a vector y")?;
        self.w_sep();
        let z = self.w_num("a vector z")?;
        Ok(node(Op::New, vec![x, y, z], Ty::Vector))
    }

    /// Called with the head word already read. `Ok(None)`: not one of ours.
    pub(crate) fn vector_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let e = match lw {
            "a" | "an" if self.w_lit(0, "vector") || (self.w_lit(0, "random") && self.w_lit(1, "vector")) => {
                self.primary_base()?
            }
            "vector" => self.w_vector()?,
            "random" if self.w_lit(0, "vector") => {
                self.pos += 1;
                node(Op::Random, vec![], Ty::Vector)
            }
            "spherical" | "cylindrical" if self.w_lit(0, "vector") => {
                self.pos += 1;
                if self.w_lit(0, "with") {
                    self.pos += 1;
                }
                let usage = "'spherical vector radius R, yaw Y, pitch P'";
                self.w_expect("radius", usage)?;
                let r = self.w_num("the radius")?;
                self.w_sep();
                self.w_expect("yaw", usage)?;
                let yaw = self.w_num("the yaw")?;
                self.w_sep();
                let sph = lw == "spherical";
                self.w_expect(if sph { "pitch" } else { "height" }, usage)?;
                let last = self.w_num(if sph { "the pitch" } else { "the height" })?;
                let op = if sph { Op::Spherical } else { Op::Cylindrical };
                node(op, vec![r, yaw, last], Ty::Vector)
            }
            "normalized" | "normalised" => {
                if self.w_lit(0, "vector") && self.w_lit(1, "of") {
                    self.pos += 2;
                }
                let v = self.w_vec("'normalized'")?;
                node(Op::Norm, vec![v], Ty::Vector)
            }
            "squared" if self.w_lit(0, "length") || self.w_lit(0, "vector") => {
                if self.w_lit(0, "vector") {
                    self.pos += 1;
                }
                self.w_expect("length", "'squared length of <vector>'")?;
                self.w_of("'squared length of <vector>'")?;
                let v = self.w_vec("'squared length of'")?;
                node(Op::LenSq, vec![v], Ty::Number)
            }
            "angle" if self.w_lit(0, "between") => {
                self.pos += 1;
                let a = self.w_vec("'angle between'")?;
                self.w_expect("and", "'angle between <vector> and <vector>'")?;
                let b = self.w_vec("'angle between'")?;
                node(Op::Angle, vec![a, b], Ty::Number)
            }
            "projection" if self.w_lit(0, "of") => {
                self.pos += 1;
                let a = self.w_vec("'projection of'")?;
                self.w_expect("onto", "'projection of <vector> onto <vector>'")?;
                let b = self.w_vec("'projection of'")?;
                node(Op::Project, vec![a, b], Ty::Vector)
            }
            "yaw" | "pitch" if self.w_lit(0, "of") => {
                self.pos += 1;
                let inner = self.primary()?;
                if !matches!(inner.ty(self.env), Ty::Vector | Ty::Player | Ty::Object) {
                    return Err(format!("'{lw} of' needs a player or vector"));
                }
                node(
                    if lw == "yaw" { Op::Yaw } else { Op::Pitch },
                    vec![inner],
                    Ty::Number,
                )
            }
            "x" | "y" | "z" if self.w_lit(0, "component") && self.w_lit(1, "of") => {
                self.pos += 2;
                let inner = self.primary()?;
                let prop = match lw {
                    "x" => Prop::X,
                    "y" => Prop::Y,
                    _ => Prop::Z,
                };
                self.prop(prop, inner)?
            }
            "location" if self.w_lit(0, "from") && self.w_lit(1, "vector") => {
                self.pos += 2;
                let v = self.w_vec("'location from vector'")?;
                self.w_in_world(v)?
            }
            "tps" => node(Op::Tps, vec![], Ty::Number),
            "mspt" => node(Op::Mspt, vec![], Ty::Number),
            "ticks" if self.w_lit(0, "per") && self.w_lit(1, "second") => {
                self.pos += 2;
                node(Op::Tps, vec![], Ty::Number)
            }
            "server" if self.w_lit(0, "tps") => {
                self.pos += 1;
                node(Op::Tps, vec![], Ty::Number)
            }
            "server" if self.w_lit(0, "mspt") => {
                self.pos += 1;
                node(Op::Mspt, vec![], Ty::Number)
            }
            "minecraft" if self.w_lit(0, "version") => {
                self.pos += 1;
                self.w_skip_string();
                node(Op::McVersion, vec![], Ty::Text)
            }
            // `version of [the] server` is `server_misc`'s.
            "version" if self.w_mc_version_ahead() => {
                self.w_skip_string();
                self.pos += 1;
                if self.w_lit(0, "the") {
                    self.pos += 1;
                }
                self.pos += 1;
                node(Op::McVersion, vec![], Ty::Text)
            }
            "server" if self.w_lit(0, "view") && self.w_lit(1, "distance") => {
                self.pos += 2;
                node(Op::ServerViewDistance, vec![], Ty::Number)
            }
            "view" | "client" if self.w_view_ahead(lw) => {
                self.pos += if lw == "client" { 3 } else { 2 };
                if self.w_lit(0, "server") {
                    self.pos += 1;
                    node(Op::ServerViewDistance, vec![], Ty::Number)
                } else {
                    let p = self.primary()?;
                    if !matches!(p.ty(self.env), Ty::Player | Ty::Object) {
                        return Err("'view distance of' needs a player or 'server'".into());
                    }
                    node(Op::ClientViewDistance, vec![p], Ty::Number)
                }
            }
            "time" if (self.w_lit(0, "played") || self.w_lit(0, "lived")) && self.w_lit(1, "of") => {
                let played = self.w_lit(0, "played");
                self.pos += 2;
                let p = self.primary()?;
                if !matches!(p.ty(self.env), Ty::Player | Ty::Object) {
                    return Err("'time played of' needs a player".into());
                }
                node(
                    if played { Op::TimePlayed } else { Op::TimeLived },
                    vec![p],
                    Ty::Timespan,
                )
            }
            "environment" | "world" if self.w_env_ahead(lw) => {
                self.pos += if lw == "world" { 2 } else { 1 };
                let w = self.primary()?;
                if !matches!(w.ty(self.env), Ty::World | Ty::Text | Ty::Object) {
                    return Err("'environment of' needs a world".into());
                }
                node(Op::Environment, vec![w], Ty::Text)
            }
            "whether" => self.w_whether()?,
            _ => return Ok(None),
        };
        Ok(Some(e))
    }

    /// `3 of diamond`: an amount of an item. The number is already read.
    pub(crate) fn vector_num_of(&mut self, n: f64) -> Option<Expr> {
        if !matches!(self.toks.get(self.pos), Some(t) if t.is_lit("of")) {
            return None;
        }
        let save = self.pos;
        self.pos += 1;
        match self.primary() {
            Ok(item) if item.ty(self.env) == Ty::Item => {
                Some(node(Op::ItemCount, vec![Expr::Num(n), item], Ty::Item))
            }
            _ => {
                self.pos = save;
                None
            }
        }
    }

    fn w_skip_string(&mut self) {
        if self.w_lit(0, "string") {
            self.pos += 1;
        }
    }

    fn w_view_ahead(&self, lw: &str) -> bool {
        if lw == "client" {
            self.w_lit(0, "view") && self.w_lit(1, "distance") && self.w_lit(2, "of")
        } else {
            self.w_lit(0, "distance") && self.w_lit(1, "of")
        }
    }

    fn w_env_ahead(&self, lw: &str) -> bool {
        if lw == "world" {
            self.w_lit(0, "environment") && self.w_lit(1, "of")
        } else {
            self.w_lit(0, "of")
        }
    }

    /// `<vector> [in [world] W]` after `location from vector`, or `to location in W`.
    fn w_in_world(&mut self, v: Expr) -> Result<Expr, String> {
        let mut args = vec![v];
        if self.w_lit(0, "in") {
            self.pos += 1;
            if self.w_lit(0, "world") && !matches!(self.toks.get(self.pos + 1), Some(Tok::Str(_))) {
                self.pos += 1;
            }
            let w = self.primary()?;
            if !matches!(w.ty(self.env), Ty::World | Ty::Text | Ty::Object) {
                return Err("expected a world after 'in'".into());
            }
            args.push(w);
        }
        Ok(node(Op::ToLoc, args, Ty::Location))
    }

    /// After the word `vector`.
    fn w_vector(&mut self) -> Result<Expr, String> {
        if self.w_lit(0, "(") && !self.has_function("vector") {
            self.pos += 1;
            let x = self.w_num("a vector x")?;
            if !self.eat_sym(",") {
                return Err("vector(x, y, z) needs three numbers".into());
            }
            let e = self.w_xyz(x)?;
            if !self.eat_sym(")") {
                return Err("missing ')' after vector(x, y, z)".into());
            }
            return Ok(e);
        }
        if self.w_lit(0, "length") && self.w_lit(1, "of") {
            self.pos += 2;
            let v = self.w_vec("'vector length of'")?;
            return Ok(node(Op::Len, vec![v], Ty::Number));
        }
        for w in ["from", "at", "with", "of", "to", "between"] {
            if self.w_lit(0, w) {
                self.pos += 1;
                break;
            }
        }
        if self.w_lit(0, "yaw") {
            self.pos += 1;
            let usage = "'vector from yaw Y and pitch P'";
            let yaw = self.w_num("the yaw")?;
            self.w_sep();
            self.w_expect("pitch", usage)?;
            let pitch = self.w_num("the pitch")?;
            return Ok(node(Op::FromYawPitch, vec![yaw, pitch], Ty::Vector));
        }
        if self.w_lit(0, "direction") {
            self.pos += 1;
            if self.w_lit(0, "of") {
                self.pos += 1;
                let p = self.primary()?;
                if !matches!(p.ty(self.env), Ty::Player | Ty::Object) {
                    return Err("'vector from direction of' needs a player".into());
                }
                return Ok(node(Op::LookDir, vec![p], Ty::Vector));
            }
            let word = match self.toks.get(self.pos) {
                Some(Tok::Word(w)) => w.to_ascii_lowercase(),
                _ => {
                    return Err(
                        "expected 'vector from direction of <player>' or a direction like north".into(),
                    );
                }
            };
            let n = match word.as_str() {
                "north" => 0,
                "south" => 1,
                "east" => 2,
                "west" => 3,
                "up" | "upwards" | "above" => 4,
                "down" | "downwards" | "below" => 5,
                _ => {
                    return Err(format!(
                        "unknown direction '{word}' (north, south, east, west, up, down)"
                    ));
                }
            };
            self.pos += 1;
            return Ok(node(Op::Compass(n), vec![], Ty::Vector));
        }
        let first = self.sum()?;
        let t = first.ty(self.env);
        let comma = self.w_lit(0, ",");
        if t == Ty::Number || (t == Ty::Object && comma) {
            return self.w_xyz(first);
        }
        if !matches!(t, Ty::Location | Ty::Player | Ty::Object) {
            return Err(format!(
                "'vector from' needs numbers or a location, got {}",
                t.name()
            ));
        }
        if self.w_lit(0, "to") || self.w_lit(0, "and") {
            self.pos += 1;
            let b = self.primary()?;
            return Ok(node(Op::Between, vec![first, b], Ty::Vector));
        }
        Ok(node(Op::FromLoc, vec![first], Ty::Vector))
    }

    /// `v dot w`, `v cross w`, `v projected onto w`, `v normalized`,
    /// `loc offset by v`, `v to location in w`.
    pub(crate) fn vector_post(&mut self, mut e: Expr) -> Result<Expr, String> {
        loop {
            let Some(Tok::Word(w)) = self.toks.get(self.pos) else {
                return Ok(e);
            };
            let lw = w.to_ascii_lowercase();
            let t = e.ty(self.env);
            let vecish = matches!(t, Ty::Vector | Ty::Object);
            match lw.as_str() {
                "dot" | "cross" if vecish => {
                    self.pos += 1;
                    let r = self.w_vec(&format!("'{lw}'"))?;
                    let (op, ty) = if lw == "dot" {
                        (Op::Dot, Ty::Number)
                    } else {
                        (Op::Cross, Ty::Vector)
                    };
                    e = node(op, vec![e, r], ty);
                }
                "projected" if vecish && self.w_lit(1, "onto") => {
                    self.pos += 2;
                    let r = self.w_vec("'projected onto'")?;
                    e = node(Op::Project, vec![e, r], Ty::Vector);
                }
                "normalized" | "normalised" if vecish => {
                    self.pos += 1;
                    e = node(Op::Norm, vec![e], Ty::Vector);
                }
                "offset" if self.w_lit(1, "by") && matches!(t, Ty::Location | Ty::Player | Ty::Object) => {
                    self.pos += 2;
                    let r = self.w_vec("'offset by'")?;
                    e = node(Op::Offset, vec![e, r], Ty::Location);
                }
                "to" if t == Ty::Vector && self.w_lit(1, "location") => {
                    self.pos += 2;
                    e = self.w_in_world(e)?;
                }
                _ => return Ok(e),
            }
        }
    }

    fn w_whether(&mut self) -> Result<Expr, String> {
        let usage = "'whether <a> is <b>' (also is not, is set, is not set, is greater than, is less than, is at least, is at most)";
        let a = self.sum()?;
        let mut cmp = None;
        if self.w_lit(0, "is") || self.w_lit(0, "are") {
            self.pos += 1;
            let not = if self.w_lit(0, "not") {
                self.pos += 1;
                true
            } else {
                false
            };
            if self.w_lit(0, "set") {
                self.pos += 1;
                cmp = Some(if not { Cmp::NotSet } else { Cmp::IsSet });
            } else if self.w_lit(0, "greater") || self.w_lit(0, "less") {
                let gt = self.w_lit(0, "greater");
                self.pos += 1;
                self.w_expect("than", usage)?;
                cmp = Some(if gt { Cmp::Gt } else { Cmp::Lt });
            } else if self.w_lit(0, "at") && (self.w_lit(1, "least") || self.w_lit(1, "most")) {
                let ge = self.w_lit(1, "least");
                self.pos += 2;
                cmp = Some(if ge { Cmp::Ge } else { Cmp::Le });
            } else {
                cmp = Some(if not { Cmp::IsNot } else { Cmp::Is });
            }
        } else if self.w_lit(0, "isn't") {
            self.pos += 1;
            cmp = Some(Cmp::IsNot);
        } else {
            for (sym, c) in [
                ("=", Cmp::Is),
                ("!=", Cmp::IsNot),
                (">=", Cmp::Ge),
                ("<=", Cmp::Le),
                (">", Cmp::Gt),
                ("<", Cmp::Lt),
            ] {
                if self.eat_sym(sym) {
                    cmp = Some(c);
                    break;
                }
            }
        }
        let Some(cmp) = cmp else {
            return Err(format!("expected {usage}"));
        };
        let mut args = vec![a];
        if !matches!(cmp, Cmp::IsSet | Cmp::NotSet) {
            args.push(self.sum()?);
        }
        Ok(node(Op::Whether(cmp), args, Ty::Boolean))
    }
}

// ---- running ---------------------------------------------------------------

type V3 = (f64, f64, f64);

fn vec3(v: &Value) -> Option<V3> {
    match v {
        Value::Vector { x, y, z } => Some((*x, *y, *z)),
        _ => None,
    }
}

fn mk(v: V3) -> Value {
    Value::Vector {
        x: v.0,
        y: v.1,
        z: v.2,
    }
}

fn dot(a: V3, b: V3) -> f64 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

/// Length of a vector.
pub fn length(v: V3) -> f64 {
    dot(v, v).sqrt()
}

/// Unit direction for a Minecraft yaw and pitch in degrees (yaw 0 faces +z).
pub fn direction(yaw: f64, pitch: f64) -> V3 {
    let (y, p) = (yaw.to_radians(), pitch.to_radians());
    (-y.sin() * p.cos(), -p.sin(), y.cos() * p.cos())
}

fn yaw_of(v: V3) -> f64 {
    if v.0 == 0.0 && v.2 == 0.0 {
        0.0
    } else {
        (-v.0).atan2(v.2).to_degrees()
    }
}

fn pitch_of(v: V3) -> f64 {
    let l = length(v);
    if l == 0.0 {
        0.0
    } else {
        (-v.1 / l).clamp(-1.0, 1.0).asin().to_degrees()
    }
}

fn scale(v: V3, k: f64) -> V3 {
    (v.0 * k, v.1 * k, v.2 * k)
}

fn norm(v: V3) -> V3 {
    let l = length(v);
    if l == 0.0 {
        (0.0, 0.0, 0.0)
    } else {
        scale(v, 1.0 / l)
    }
}

fn num_of(v: &Value) -> Option<f64> {
    match v {
        Value::Num(n) => Some(*n),
        Value::Text(t) => t.trim().parse().ok(),
        _ => None,
    }
}

fn player_of(v: &Value, host: &mut dyn Host) -> Option<PlayerRef> {
    match v {
        Value::Player(p) => host.resolve_player(&p.uuid).map(|name| PlayerRef {
            uuid: p.uuid.clone(),
            name,
        }),
        Value::Text(n) => host.find_player(n),
        _ => None,
    }
}

type Loc = (f64, f64, f64, Option<String>);

fn loc_of(v: &Value, host: &mut dyn Host) -> Option<Loc> {
    match v {
        Value::Location { x, y, z, world } => Some((*x, *y, *z, world.clone())),
        Value::Player(_) | Value::Text(_) => {
            let p = player_of(v, host)?;
            host.player_location(&p).map(|(x, y, z, w)| (x, y, z, Some(w)))
        }
        _ => None,
    }
}

fn rnd(host: &mut dyn Host) -> f64 {
    static STATE: AtomicU64 = AtomicU64::new(0);
    let mut s = STATE.load(Ordering::Relaxed);
    if s == 0 {
        s = (host.wall_clock_millis() as u64) ^ 0x9E37_79B9_7F4A_7C15;
    }
    s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
    STATE.store(s, Ordering::Relaxed);
    let mut z = s;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64
}

/// Math with a vector on a side (`v + w`, `v * 2`, `loc + v`). Used by the
/// engine for variables that turn out to hold vectors.
pub fn arith(x: &Value, op: BinOp, y: &Value) -> Result<Value, String> {
    let bad = || {
        Err(format!(
            "can't do {} and {} with that operator (vectors take + - between vectors, and * / with a number)",
            x.type_name(),
            y.type_name()
        ))
    };
    match (x, y) {
        (Value::Vector { .. }, Value::Vector { .. }) => {
            let (a, b) = (vec3(x).unwrap_or_default(), vec3(y).unwrap_or_default());
            Ok(mk(match op {
                BinOp::Add => (a.0 + b.0, a.1 + b.1, a.2 + b.2),
                BinOp::Sub => (a.0 - b.0, a.1 - b.1, a.2 - b.2),
                BinOp::Mul => (a.0 * b.0, a.1 * b.1, a.2 * b.2),
                BinOp::Div => {
                    if b.0 == 0.0 || b.1 == 0.0 || b.2 == 0.0 {
                        return Err("division by zero".into());
                    }
                    (a.0 / b.0, a.1 / b.1, a.2 / b.2)
                }
            }))
        }
        (Value::Vector { .. }, n) | (n, Value::Vector { .. }) => {
            let vec_left = matches!(x, Value::Vector { .. });
            let (v, n) = (vec3(if vec_left { x } else { y }).unwrap_or_default(), num_of(n));
            match (n, op, vec_left) {
                (Some(n), BinOp::Mul, _) => Ok(mk(scale(v, n))),
                (Some(n), BinOp::Div, true) => {
                    if n == 0.0 {
                        return Err("division by zero".into());
                    }
                    Ok(mk(scale(v, 1.0 / n)))
                }
                _ => match (x, y, op) {
                    (
                        Value::Location {
                            x: lx,
                            y: ly,
                            z: lz,
                            world,
                        },
                        Value::Vector { x: a, y: b, z: c },
                        BinOp::Add,
                    ) => Ok(Value::Location {
                        x: lx + a,
                        y: ly + b,
                        z: lz + c,
                        world: world.clone(),
                    }),
                    (
                        Value::Location {
                            x: lx,
                            y: ly,
                            z: lz,
                            world,
                        },
                        Value::Vector { x: a, y: b, z: c },
                        BinOp::Sub,
                    ) => Ok(Value::Location {
                        x: lx - a,
                        y: ly - b,
                        z: lz - c,
                        world: world.clone(),
                    }),
                    (
                        Value::Vector { x: a, y: b, z: c },
                        Value::Location {
                            x: lx,
                            y: ly,
                            z: lz,
                            world,
                        },
                        BinOp::Add,
                    ) => Ok(Value::Location {
                        x: lx + a,
                        y: ly + b,
                        z: lz + c,
                        world: world.clone(),
                    }),
                    _ => bad(),
                },
            }
        }
        _ => bad(),
    }
}

fn truthy_cmp(cmp: Cmp, a: &Value, b: Option<&Value>) -> bool {
    let ord = |a: &Value, b: &Value| -> Option<std::cmp::Ordering> {
        let key = |v: &Value| match v {
            Value::Num(n) => Some(*n),
            Value::Date(d) => Some(*d as f64),
            Value::Timespan { millis } => Some(*millis as f64),
            Value::Text(t) => t.trim().parse().ok(),
            _ => None,
        };
        key(a)?.partial_cmp(&key(b)?)
    };
    match cmp {
        Cmp::IsSet => !matches!(a, Value::None),
        Cmp::NotSet => matches!(a, Value::None),
        Cmp::Is => b.is_some_and(|b| a.loose_eq(b)),
        Cmp::IsNot => !b.is_some_and(|b| a.loose_eq(b)),
        Cmp::Gt => b.and_then(|b| ord(a, b)).is_some_and(|o| o.is_gt()),
        Cmp::Lt => b.and_then(|b| ord(a, b)).is_some_and(|o| o.is_lt()),
        Cmp::Ge => b.and_then(|b| ord(a, b)).is_some_and(|o| o.is_ge()),
        Cmp::Le => b.and_then(|b| ord(a, b)).is_some_and(|o| o.is_le()),
    }
}

fn environment_name(id: &str) -> &'static str {
    let id = id.strip_prefix("minecraft:").unwrap_or(id);
    match id {
        "the_nether" | "nether" => "nether",
        "the_end" | "end" => "the end",
        _ => "normal",
    }
}

/// Run one node on its evaluated arguments.
pub fn apply(op: Op, a: &[Value], host: &mut dyn Host) -> Result<Value, String> {
    let first = a.first().unwrap_or(&Value::None);
    let second = a.get(1).unwrap_or(&Value::None);
    let v1 = vec3(first);
    let v2 = vec3(second);
    Ok(match op {
        Op::New => match (num_of(first), num_of(second), a.get(2).and_then(num_of)) {
            (Some(x), Some(y), Some(z)) => mk((x, y, z)),
            _ => Value::None,
        },
        Op::Neg => v1.map_or(Value::None, |v| mk(scale(v, -1.0))),
        Op::Arith(op) => arith(first, op, second)?,
        Op::Len => v1.map_or(Value::None, |v| Value::Num(length(v))),
        Op::LenSq => v1.map_or(Value::None, |v| Value::Num(dot(v, v))),
        Op::Norm => v1.map_or(Value::None, |v| mk(norm(v))),
        Op::Dot => match (v1, v2) {
            (Some(x), Some(y)) => Value::Num(dot(x, y)),
            _ => Value::None,
        },
        Op::Cross => match (v1, v2) {
            (Some(x), Some(y)) => mk((
                x.1 * y.2 - x.2 * y.1,
                x.2 * y.0 - x.0 * y.2,
                x.0 * y.1 - x.1 * y.0,
            )),
            _ => Value::None,
        },
        Op::Angle => match (v1, v2) {
            (Some(x), Some(y)) => {
                let d = length(x) * length(y);
                Value::Num(if d == 0.0 {
                    0.0
                } else {
                    (dot(x, y) / d).clamp(-1.0, 1.0).acos().to_degrees()
                })
            }
            _ => Value::None,
        },
        Op::Project => match (v1, v2) {
            (Some(x), Some(y)) => {
                let d = dot(y, y);
                mk(if d == 0.0 {
                    (0.0, 0.0, 0.0)
                } else {
                    scale(y, dot(x, y) / d)
                })
            }
            _ => Value::None,
        },
        Op::Yaw | Op::Pitch => {
            let yaw = op == Op::Yaw;
            if let Some(v) = v1 {
                Value::Num(if yaw { yaw_of(v) } else { pitch_of(v) })
            } else if let Some(p) = player_of(first, host) {
                let q = if yaw { Query::Yaw(&p) } else { Query::Pitch(&p) };
                host.server_query(q).unwrap_or(Value::None)
            } else {
                Value::None
            }
        }
        Op::FromYawPitch => match (num_of(first), num_of(second)) {
            (Some(y), Some(p)) => mk(direction(y, p)),
            _ => Value::None,
        },
        Op::LookDir => match player_of(first, host) {
            Some(p) => {
                let yaw = host.server_query(Query::Yaw(&p)).as_ref().and_then(num_of);
                let pitch = host.server_query(Query::Pitch(&p)).as_ref().and_then(num_of);
                match (yaw, pitch) {
                    (Some(y), Some(p)) => mk(direction(y, p)),
                    _ => Value::None,
                }
            }
            None => Value::None,
        },
        Op::Compass(n) => mk(match n {
            0 => (0.0, 0.0, -1.0),
            1 => (0.0, 0.0, 1.0),
            2 => (1.0, 0.0, 0.0),
            3 => (-1.0, 0.0, 0.0),
            4 => (0.0, 1.0, 0.0),
            _ => (0.0, -1.0, 0.0),
        }),
        Op::FromLoc => loc_of(first, host).map_or(Value::None, |l| mk((l.0, l.1, l.2))),
        Op::ToLoc => {
            let world = match second {
                Value::World(w) | Value::Text(w) => Some(w.clone()),
                _ => None,
            };
            v1.map_or(Value::None, |v| Value::Location {
                x: v.0,
                y: v.1,
                z: v.2,
                world,
            })
        }
        Op::Offset => match (loc_of(first, host), v2) {
            (Some(l), Some(v)) => Value::Location {
                x: l.0 + v.0,
                y: l.1 + v.1,
                z: l.2 + v.2,
                world: l.3,
            },
            _ => Value::None,
        },
        Op::Between => match (loc_of(first, host), loc_of(second, host)) {
            (Some(p), Some(q)) => mk((q.0 - p.0, q.1 - p.1, q.2 - p.2)),
            _ => Value::None,
        },
        Op::Random => {
            let z = rnd(host) * 2.0 - 1.0;
            let t = rnd(host) * std::f64::consts::TAU;
            let r = (1.0 - z * z).max(0.0).sqrt();
            mk((r * t.cos(), z, r * t.sin()))
        }
        Op::Spherical => match (num_of(first), num_of(second), a.get(2).and_then(num_of)) {
            (Some(r), Some(y), Some(p)) => mk(scale(direction(y, p), r)),
            _ => Value::None,
        },
        Op::Cylindrical => match (num_of(first), num_of(second), a.get(2).and_then(num_of)) {
            (Some(r), Some(y), Some(h)) => {
                let d = direction(y, 0.0);
                mk((d.0 * r, h, d.2 * r))
            }
            _ => Value::None,
        },
        Op::Tps => host.server_query(Query::Tps).unwrap_or(Value::None),
        Op::Mspt => host.server_query(Query::Mspt).unwrap_or(Value::None),
        Op::McVersion => host.server_query(Query::MinecraftVersion).unwrap_or(Value::None),
        Op::ServerViewDistance => host
            .server_query(Query::ServerViewDistance)
            .unwrap_or(Value::None),
        // The same host read as SkBee's `view distance of player`.
        Op::ClientViewDistance => player_of(first, host)
            .and_then(|p| host.player_info(&p, crate::skbee_utilities::Info::ViewDistance))
            .and_then(|s| s.trim().parse::<f64>().ok())
            .map_or(Value::None, Value::Num),
        Op::TimePlayed | Op::TimeLived => match player_of(first, host) {
            Some(p) => {
                let q = match op {
                    Op::TimePlayed => Query::PlayTicks(&p),
                    _ => Query::LivedTicks(&p),
                };
                match host.server_query(q) {
                    Some(Value::Num(n)) => Value::Timespan {
                        millis: (n * 50.0).round() as i64,
                    },
                    Some(v) => v,
                    None => Value::None,
                }
            }
            None => Value::None,
        },
        Op::Environment => {
            let name = match first {
                Value::World(w) | Value::Text(w) => w.as_str(),
                _ => return Ok(Value::None),
            };
            match host.server_query(Query::Environment(name)) {
                Some(Value::Text(id)) => Value::Text(environment_name(&id).into()),
                _ => Value::None,
            }
        }
        Op::ItemCount => match (num_of(first), second) {
            (Some(n), Value::Item { id, meta, .. }) if n >= 0.0 => Value::Item {
                id: id.clone(),
                count: n.min(f64::from(u32::MAX)) as u32,
                meta: meta.clone(),
            },
            _ => Value::None,
        },
        Op::Whether(c) => Value::Bool(truthy_cmp(c, first, a.get(1))),
    })
}

/// `x of {vec}` and friends for the engine's property code.
pub fn component(prop: Prop, v: &Value) -> Option<Value> {
    let (x, y, z) = vec3(v)?;
    Some(match prop {
        Prop::X => Value::Num(x),
        Prop::Y => Value::Num(y),
        Prop::Z => Value::Num(z),
        _ => Value::Location { x, y, z, world: None },
    })
}
