//! Wave 2, expressions-6: special numbers, random characters and UUIDs,
//! `raw`, `repeated`, ternary, `reduce`, `recursive`, and a few host reads
//! (`remaining air`, `respawn location`, `sea level`, `simulation distance`,
//! `tablisted players`, `quit message`).
//!
//! The rare variants all live behind one boxed [`W6`] value in `Expr`, so the
//! hot expression enum does not grow. Running them is in `engine/w6_eval.rs`.

use crate::compile::{EffKind, Effect};
use crate::expr::{Expr, P, Ty, VarRef, parse_varref};
use crate::lexer::Tok;
use crate::list_ops::{ListSrc, parse_src_at};
use crate::pattern::Capture;

/// Local variable that holds the running total while `reduce` works.
/// The control character keeps scripts from clashing with it.
pub const REDUCED_KEY: &str = "_\u{1}reduced";

/// Longest text `repeated` may build, in characters.
pub const MAX_REPEAT_CHARS: usize = 1_000_000;

/// Values read from the server or a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Read {
    /// Timespan of air left.
    RemainingAir,
    RespawnLocation,
    /// Number, of a world.
    SeaLevel,
    SimulationDistance,
    /// Online players, as a list.
    Tablist,
}

#[derive(Debug, Clone, PartialEq)]
pub enum W6 {
    /// `a if <condition> else b`
    Ternary(Box<crate::compile::Cond>, Expr, Expr),
    /// `random alphanumeric character`, `random character between "a" and "z"`
    RandomChar {
        alnum: bool,
        from: Option<Expr>,
        to: Option<Expr>,
    },
    RandomUuid,
    /// `raw "&aHi"`
    Raw(Expr),
    /// `"ab" repeated 3 times`
    Repeat(Expr, Expr),
    /// `reduce {x::*} with [reduced value + input]`
    Reduce(ListSrc, Expr),
    /// `recursive size of {x::*}`
    RecursiveSize(VarRef),
    /// A host read; the expression is the player or world it is read from.
    Read(Read, Option<Expr>),
}

impl W6 {
    pub fn ty(&self, env: &crate::expr::Env) -> Ty {
        match self {
            W6::Ternary(_, a, b) => {
                let (x, y) = (a.ty(env), b.ty(env));
                if x == y { x } else { Ty::Object }
            }
            W6::RandomChar { .. } | W6::RandomUuid | W6::Raw(_) | W6::Repeat(..) => Ty::Text,
            W6::Reduce(..) => Ty::Object,
            W6::RecursiveSize(_) => Ty::Number,
            W6::Read(r, _) => match r {
                Read::RemainingAir => Ty::Timespan,
                Read::RespawnLocation => Ty::Location,
                Read::SeaLevel | Read::SimulationDistance => Ty::Number,
                Read::Tablist => Ty::List,
            },
        }
    }
}

/// Characters `raw` puts after `&`, `§` and `<`, so colour codes and tags in
/// the text print as written instead of being turned into colours.
const ZWSP: char = '\u{200B}';

/// `raw`: break every colour code and tag with an invisible character.
pub fn raw(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for c in s.chars() {
        out.push(c);
        if matches!(c, '&' | '§' | '<') {
            out.push(ZWSP);
        }
    }
    out
}

/// Format 16 random bytes as a version 4 UUID.
pub fn uuid_from(bytes: [u8; 16]) -> String {
    let mut b = bytes;
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
    format!("{}-{}-{}-{}-{}", &h[0..8], &h[8..12], &h[12..16], &h[16..20], &h[20..32])
}

/// Pick a character. `r` is a random number in [0, 1).
pub fn pick_char(alnum: bool, range: Option<(char, char)>, r: f64) -> char {
    const ALNUM: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    if let Some((a, b)) = range {
        let (lo, hi) = if a <= b { (a as u32, b as u32) } else { (b as u32, a as u32) };
        let n = hi - lo + 1;
        let mut code = lo + ((r * f64::from(n)) as u32).min(n - 1);
        // Skip the surrogate gap so any range gives a real character.
        if (0xD800..0xE000).contains(&code) {
            code = lo;
        }
        return char::from_u32(code).unwrap_or(a);
    }
    if alnum {
        let i = ((r * ALNUM.len() as f64) as usize).min(ALNUM.len() - 1);
        return char::from(ALNUM.get(i).copied().unwrap_or(b'a'));
    }
    // Printable ASCII without the space.
    char::from(33 + ((r * 94.0) as u8).min(93))
}

/// Repeat `s` `n` times. `None` when the result would be too long.
pub fn repeat(s: &str, n: f64) -> Option<String> {
    if !n.is_finite() || n < 1.0 {
        return Some(String::new());
    }
    let n = n as usize;
    if s.is_empty() {
        return Some(String::new());
    }
    if s.chars().count().checked_mul(n)? > MAX_REPEAT_CHARS {
        return None;
    }
    Some(s.repeat(n))
}

// ---------- parsing ----------

fn lw(t: Option<&Tok>) -> Option<String> {
    match t {
        Some(Tok::Word(w)) => Some(w.to_ascii_lowercase()),
        _ => None,
    }
}

/// Property words after `'s`: the read and how many tokens it used.
pub fn possessive(toks: &[Tok], i: usize) -> Option<(Read, usize)> {
    match (lw(toks.get(i)).as_deref(), lw(toks.get(i + 1)).as_deref()) {
        (Some("remaining"), Some("air")) => Some((Read::RemainingAir, 2)),
        (Some("respawn"), Some("location")) => Some((Read::RespawnLocation, 2)),
        _ => None,
    }
}

impl P<'_> {
    fn w6_next(&self, k: usize) -> Option<String> {
        if self.pos + k >= self.toks.len() {
            self.hit_end.set(true);
        }
        lw(self.toks.get(self.pos + k))
    }

    fn w6_expr(&self, w: W6) -> Expr {
        Expr::W6(Box::new(w))
    }

    /// `of <value>` after the words already read.
    fn w6_of(&mut self) -> Result<Expr, String> {
        if self.w6_next(0).as_deref() != Some("of") {
            return Err("expected 'of'".into());
        }
        self.pos += 1;
        self.primary()
    }

    /// Word-led expressions. `lw` is the word just read.
    pub(crate) fn w6_word(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let n0 = self.w6_next(0);
        let n1 = self.w6_next(1);
        let (n0, n1) = (n0.as_deref(), n1.as_deref());
        Ok(Some(match lw {
            "pi" => Expr::Num(std::f64::consts::PI),
            "infinity" => Expr::Num(f64::INFINITY),
            "nan" => Expr::Num(f64::NAN),
            "negative" | "minus" if n0 == Some("infinity") => {
                self.pos += 1;
                Expr::Num(f64::NEG_INFINITY)
            }
            "a" | "an" if n0 == Some("random") => {
                self.pos += 1;
                return self.w6_random();
            }
            "random" => return self.w6_random(),
            // `raw iron`, `raw gold block` are items, not `raw <text>`.
            "raw" if self.pos < self.toks.len() && !n0.is_some_and(|w| crate::items::is_known(&format!("raw_{w}"))) => {
                if matches!(n0, Some("string" | "text")) {
                    self.pos += 1;
                }
                if self.w6_next(0).as_deref() == Some("of") {
                    self.pos += 1;
                }
                let inner = self.primary()?;
                self.w6_expr(W6::Raw(inner))
            }
            "recursive" if n0 == Some("size") && n1 == Some("of") => {
                self.pos += 2;
                let var = self.w6_list_var("recursive size of")?;
                self.w6_expr(W6::RecursiveSize(var))
            }
            "recursive" if matches!(self.toks.get(self.pos), Some(Tok::Var(_))) => {
                let var = self.w6_list_var("recursive")?;
                Expr::Items(Box::new(ListSrc::Recursive(var)))
            }
            "reduce" if self.pos < self.toks.len() => {
                let (src, end) = parse_src_at(self.toks, self.pos, self.env, false).inspect_err(|_| self.hit_end.set(true))?;
                self.pos = end;
                self.w6_reduce_tail(src)?
            }
            "input" => Expr::Var(parse_varref(crate::list_ops::INPUT_KEY, self.env)?),
            "reduced" if n0 == Some("value") => {
                self.pos += 1;
                Expr::Var(parse_varref(REDUCED_KEY, self.env)?)
            }
            "quit" | "leave" if n0 == Some("message") => {
                if self.env.trigger_name != "'on quit'" || self.env.in_function {
                    return Err("'quit message' only exists in 'on quit'".into());
                }
                self.pos += 1;
                Expr::Message
            }
            "remaining" if n0 == Some("air") && n1 == Some("of") => {
                self.pos += 2;
                let p = self.primary()?;
                self.w6_expr(W6::Read(Read::RemainingAir, Some(p)))
            }
            "respawn" if n0 == Some("location") && n1 == Some("of") => {
                self.pos += 2;
                let p = self.primary()?;
                self.w6_expr(W6::Read(Read::RespawnLocation, Some(p)))
            }
            "sea" if n0 == Some("level") && n1 == Some("of") => {
                self.pos += 2;
                let w = self.primary()?;
                self.w6_expr(W6::Read(Read::SeaLevel, Some(w)))
            }
            "simulation" if n0 == Some("distance") => {
                self.pos += 1;
                self.w6_expr(W6::Read(Read::SimulationDistance, None))
            }
            "tablisted" if n0 == Some("players") => {
                self.pos += 1;
                let of = if self.w6_next(0).as_deref() == Some("of") {
                    Some(self.w6_of()?)
                } else {
                    None
                };
                self.w6_expr(W6::Read(Read::Tablist, of))
            }
            _ => return Ok(None),
        }))
    }

    fn w6_list_var(&mut self, what: &str) -> Result<VarRef, String> {
        match self.peek().cloned() {
            Some(Tok::Var(v)) => {
                self.pos += 1;
                let var = parse_varref(&v, self.env)?;
                if var.is_list() {
                    Ok(var)
                } else {
                    Err(format!("'{what}' needs a list variable like {{x::*}}"))
                }
            }
            _ => Err(format!("'{what}' needs a list variable like {{x::*}}")),
        }
    }

    /// After `random`: `uuid`, `[alphanumeric] character [between a and b]`.
    fn w6_random(&mut self) -> Result<Option<Expr>, String> {
        let n0 = self.w6_next(0);
        match n0.as_deref() {
            Some("uuid") => {
                self.pos += 1;
                Ok(Some(self.w6_expr(W6::RandomUuid)))
            }
            Some("alphanumeric" | "character" | "char") => {
                let alnum = n0.as_deref() == Some("alphanumeric");
                self.pos += 1;
                if alnum {
                    if !matches!(self.w6_next(0).as_deref(), Some("character" | "char")) {
                        return Err("expected 'random alphanumeric character'".into());
                    }
                    self.pos += 1;
                }
                let (mut from, mut to) = (None, None);
                if matches!(self.w6_next(0).as_deref(), Some("between" | "from")) {
                    self.pos += 1;
                    from = Some(self.primary_base()?);
                    if !matches!(self.w6_next(0).as_deref(), Some("and" | "to")) {
                        return Err("expected 'random character between \"a\" and \"z\"'".into());
                    }
                    self.pos += 1;
                    to = Some(self.primary_base()?);
                }
                Ok(Some(self.w6_expr(W6::RandomChar { alnum, from, to })))
            }
            _ => Ok(None),
        }
    }

    /// After the list in `reduce <list>`: `with [<expression>]`.
    fn w6_reduce_tail(&mut self, src: ListSrc) -> Result<Expr, String> {
        if self.w6_next(0).as_deref() != Some("with") {
            return Err("expected 'reduce {list::*} with [reduced value + input]'".into());
        }
        self.pos += 1;
        let start = self.pos;
        let (body, end) = if self.toks.get(start).is_some_and(|t| t.is_lit("[")) {
            let mut depth = 0i32;
            let mut close = None;
            for (i, t) in self.toks.iter().enumerate().skip(start) {
                if t.is_lit("[") {
                    depth += 1;
                } else if t.is_lit("]") {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(i);
                        break;
                    }
                }
            }
            let Some(close) = close else {
                self.hit_end.set(true);
                return Err("missing ']' after 'reduce ... with ['".into());
            };
            (self.toks.get(start + 1..close).unwrap_or(&[]), close + 1)
        } else {
            let mut depth = 0i32;
            let mut end = self.toks.len();
            for (i, t) in self.toks.iter().enumerate().skip(start) {
                if t.is_lit("(") {
                    depth += 1;
                } else if t.is_lit(")") {
                    depth -= 1;
                    if depth < 0 {
                        end = i;
                        break;
                    }
                }
            }
            (self.toks.get(start..end).unwrap_or(&[]), end)
        };
        if body.is_empty() {
            self.hit_end.set(true);
            return Err("empty 'reduce ... with' expression".into());
        }
        let body: Vec<Tok> = body
            .iter()
            .map(|t| match t {
                Tok::Word(w) if w.eq_ignore_ascii_case("input") => Tok::Var(crate::list_ops::INPUT_KEY.into()),
                t => t.clone(),
            })
            .collect();
        let body = crate::expr::parse_expr(&body, self.env).inspect_err(|_| self.hit_end.set(true))?;
        self.pos = end;
        Ok(self.w6_expr(W6::Reduce(src, body)))
    }

    /// Postfix forms: `"ab" repeated 3 times`, `{x::*} reduced with [...]`.
    pub(crate) fn w6_postfix(&mut self, mut e: Expr) -> Result<Expr, String> {
        loop {
            let n0 = lw(self.toks.get(self.pos));
            match n0.as_deref() {
                Some("repeated") if matches!(e.ty(self.env), Ty::Text | Ty::Object) => {
                    self.pos += 1;
                    let n = self.sum()?;
                    if !Ty::Number.accepts(n.ty(self.env)) {
                        return Err("'repeated' needs a number, like '\"ab\" repeated 3 times'".into());
                    }
                    if self.w6_next(0).as_deref().is_some_and(|w| w == "times" || w == "time") {
                        self.pos += 1;
                    } else {
                        return Err("expected 'times' after 'repeated N'".into());
                    }
                    e = self.w6_expr(W6::Repeat(e, n));
                }
                Some("reduced") if self.w6_next(1).as_deref() == Some("with") => {
                    let src = match e {
                        Expr::Var(v) if v.is_list() => ListSrc::Var(v),
                        Expr::Items(s) => *s,
                        _ => return Err("'reduced with' needs a list like {x::*}".into()),
                    };
                    self.pos += 1;
                    e = self.w6_reduce_tail(src)?;
                }
                _ => return Ok(e),
            }
        }
    }

    /// `a if <condition> else b`, below `?`. Leaves `e` alone when there is no
    /// `else` after the `if`.
    pub(crate) fn w6_ternary(&mut self, e: Expr) -> Result<Expr, String> {
        if !self.toks.get(self.pos).is_some_and(|t| t.is_lit("if")) {
            return Ok(e);
        }
        let start = self.pos + 1;
        let mut depth = 0i32;
        let mut at = None;
        for (i, t) in self.toks.iter().enumerate().skip(start) {
            if t.is_lit("(") || t.is_lit("[") {
                depth += 1;
            } else if t.is_lit(")") || t.is_lit("]") {
                depth -= 1;
            } else if depth == 0 && (t.is_lit("else") || t.is_lit("otherwise")) {
                at = Some(i);
                break;
            }
        }
        let Some(at) = at else {
            self.hit_end.set(true);
            return Ok(e);
        };
        let cond = crate::list_ops::parse_filter(self.toks.get(start..at).unwrap_or(&[]), self.env)
            .map_err(|m| format!("bad condition in 'a if <condition> else b': {m}"))?;
        self.pos = at + 1;
        let other = self.top()?;
        Ok(self.w6_expr(W6::Ternary(Box::new(cond), e, other)))
    }
}

// ---------- effects ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum W6Op {
    SetAir,
    AddAir,
    RemoveAir,
    SetRespawn,
}

#[derive(Debug, Clone, PartialEq)]
pub struct W6Effect {
    pub op: W6Op,
    pub player: Expr,
    pub value: Expr,
}

use EffKind::W6 as K;
use W6Op::{AddAir, RemoveAir, SetAir, SetRespawn};

/// Extra effect patterns, tried after the main list.
pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    ("set remaining air of %player% to %timespan%", K(SetAir)),
    ("set %player%'s remaining air to %timespan%", K(SetAir)),
    ("add %timespan% to remaining air of %player%", K(AddAir)),
    ("add %timespan% to %player%'s remaining air", K(AddAir)),
    ("remove %timespan% from remaining air of %player%", K(RemoveAir)),
    ("remove %timespan% from %player%'s remaining air", K(RemoveAir)),
    ("set respawn location of %player% to %location%", K(SetRespawn)),
    ("set %player%'s respawn location to %location%", K(SetRespawn)),
];

fn cap(caps: &mut [Option<Capture>], i: usize) -> Option<Expr> {
    match caps.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    }
}

/// Build the effect. `add` and `remove` capture the value first.
pub fn build(op: W6Op, caps: &mut [Option<Capture>]) -> Option<Effect> {
    let (p, v) = match op {
        AddAir | RemoveAir => (1, 0),
        _ => (0, 1),
    };
    Some(Effect::W6(Box::new(W6Effect {
        op,
        player: cap(caps, p)?,
        value: cap(caps, v)?,
    })))
}
