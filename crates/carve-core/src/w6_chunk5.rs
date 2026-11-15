//! Wave 6, chunk 5: `parsed as integer|boolean|player`, `<timespan> formatted
//! human-readable`, loot tables and loot contexts (plain records), item values
//! with a component (`with custom model data`, `with enchantment glint`,
//! `with item model`) and a few game facts through one host
//! method, [`Host::w6c5`].
//!
//! Every expression is one boxed [`Node`] (`Expr::W6c5`), so the hot `Expr`
//! and `eval` paths stay small.

use crate::expr::{Expr, P, Ty};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::{Value, push_num};

/// A loot table key or a loot context. Stored in `Value::W5` (see
/// `w5_chunk5::Obj::Loot`). Not saved in global variables.
#[derive(Debug, Clone, PartialEq)]
pub enum Loot {
    /// A namespaced key like `minecraft:chests/simple_dungeon`.
    Table(String),
    Context(Box<Ctx>),
}

/// `loot context at <location> with luck 2 with looter <player>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Ctx {
    pub loc: Value,
    pub luck: f64,
    pub looter: Value,
    pub looted: Value,
}

impl Loot {
    pub fn type_name(&self) -> &'static str {
        match self {
            Loot::Table(_) => "loot table",
            Loot::Context(_) => "loot context",
        }
    }

    pub fn push_to(&self, out: &mut String) {
        match self {
            Loot::Table(k) => out.push_str(k),
            Loot::Context(c) => {
                out.push_str("loot context at ");
                out.push_str(&c.loc.to_string());
                out.push_str(" with luck ");
                push_num(out, c.luck);
            }
        }
    }
}

/// What [`Host::w6c5`] is asked. `None` means unknown.
#[derive(Debug)]
pub enum Q<'a> {
    /// Largest stack of an item id, read from the server.
    MaxStack(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Integer,
    Boolean,
    Player,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comp {
    CustomModelData,
    Glint,
    Model,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    ParseAs(Kind),
    /// timespan.
    HumanSpan,
    /// text key.
    TableFromKey,
    /// location, luck, looter, looted entity (empty text: none).
    CtxNew,
    CtxLocation,
    CtxLooter,
    CtxLooted,
    CtxLuck,
    /// item, value.
    ItemWith(Comp),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub op: Op,
    pub args: Vec<Expr>,
    pub ty: Ty,
}

fn node(op: Op, args: Vec<Expr>, ty: Ty) -> Expr {
    Expr::W6c5(Box::new(Node { op, args, ty }))
}

fn loot_value(l: Loot) -> Value {
    Value::W5(Box::new(crate::w5_chunk5::Obj::Loot(l)))
}

/// `key of <value>`: the namespaced key of an item, a named value or a loot table.
pub fn key_of(v: &Value) -> Value {
    match v {
        Value::Item { id, .. } => Value::Text(format!("minecraft:{id}")),
        Value::Enum(e) => Value::Text(format!("minecraft:{}", e.id())),
        Value::W5(o) => match &**o {
            crate::w5_chunk5::Obj::Loot(Loot::Table(k)) => Value::Text(k.clone()),
            _ => Value::None,
        },
        _ => Value::None,
    }
}

/// `true`, `yes`, `on`, `false`, `no`, `off`.
pub fn parse_bool(t: &str) -> Option<bool> {
    match t.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" => Some(true),
        "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// An integer written as text: optional sign and digits only.
pub fn parse_integer(t: &str) -> Option<i64> {
    let t = t.trim();
    let d = t.strip_prefix(['-', '+']).unwrap_or(t);
    if d.is_empty() || !d.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    t.parse().ok()
}

/// `1 hour, 5 minutes and 3 seconds`.
pub fn human_span(ms: i64) -> String {
    let mut s = String::new();
    crate::timespans::push_span(&mut s, ms);
    s
}

fn namespaced(k: &str) -> String {
    let k = k.trim().to_ascii_lowercase();
    if k.contains(':') {
        k
    } else {
        format!("minecraft:{k}")
    }
}

/// Component name and SNBT for `with <comp>`.
fn comp_snbt(c: Comp, v: &Value) -> Result<(&'static str, String), String> {
    Ok(match c {
        Comp::CustomModelData => {
            let n = v
                .as_num()
                .filter(|n| n.is_finite())
                .ok_or("custom model data must be a number")?;
            let mut f = String::new();
            push_num(&mut f, n);
            if !f.contains('.') {
                f.push_str(".0");
            }
            ("custom_model_data", format!("{{floats:[{f}f]}}"))
        }
        Comp::Glint => match v {
            Value::Bool(b) => ("enchantment_glint_override", b.to_string()),
            _ => return Err("enchantment glint must be true or false".into()),
        },
        Comp::Model => ("item_model", format!("\"{}\"", namespaced(&v.to_string()))),
    })
}

fn is_unset(v: &Value) -> bool {
    matches!(v, Value::None) || matches!(v, Value::Text(t) if t.is_empty())
}

pub fn apply(op: Op, vals: &[Value], host: &mut dyn Host) -> Result<Value, String> {
    let a = vals.first().unwrap_or(&Value::None);
    let b = vals.get(1).unwrap_or(&Value::None);
    Ok(match op {
        Op::ParseAs(k) => {
            let t = a.to_string();
            match k {
                Kind::Integer => parse_integer(&t).map_or(Value::None, |n| Value::Num(n as f64)),
                Kind::Boolean => parse_bool(&t).map_or(Value::None, Value::Bool),
                Kind::Text => Value::Text(t),
                Kind::Player => host.find_player(t.trim()).map_or(Value::None, Value::Player),
            }
        }
        Op::HumanSpan => match a {
            Value::Timespan { millis } => Value::Text(human_span(*millis)),
            _ => Value::None,
        },
        Op::TableFromKey => match a {
            Value::None => Value::None,
            v => loot_value(Loot::Table(namespaced(&v.to_string()))),
        },
        Op::CtxNew => {
            let Value::Location { .. } = a else {
                return Err("a loot context needs a location".into());
            };
            let pick = |i: usize| {
                vals.get(i)
                    .filter(|v| !is_unset(v))
                    .cloned()
                    .unwrap_or(Value::None)
            };
            loot_value(Loot::Context(Box::new(Ctx {
                loc: a.clone(),
                luck: b.as_num().unwrap_or(0.0),
                looter: pick(2),
                looted: pick(3),
            })))
        }
        Op::CtxLocation | Op::CtxLooter | Op::CtxLooted | Op::CtxLuck => match a {
            Value::W5(o) => match &**o {
                crate::w5_chunk5::Obj::Loot(Loot::Context(c)) => match op {
                    Op::CtxLocation => c.loc.clone(),
                    Op::CtxLooter => c.looter.clone(),
                    Op::CtxLooted => c.looted.clone(),
                    _ => Value::Num(c.luck),
                },
                _ => Value::None,
            },
            _ => Value::None,
        },
        Op::ItemWith(c) => {
            let Value::Item { id, count, meta } = a else {
                return Ok(Value::None);
            };
            let (name, snbt) = comp_snbt(c, b)?;
            let mut m = meta.as_deref().cloned().unwrap_or_default();
            m.comps.retain(|(k, _)| k != name);
            m.comps.push((name.to_string(), snbt));
            Value::Item {
                id: id.clone(),
                count: *count,
                meta: Some(Box::new(m)),
            }
        }
    })
}

/// Largest stack of an item: the server's value, else the built-in table.
pub fn max_stack(id: &str, host: &mut dyn Host) -> u32 {
    host.w6c5(Q::MaxStack(id))
        .and_then(|v| v.as_num())
        .filter(|n| *n >= 1.0)
        .map_or_else(|| crate::w2_expressions_5::default_stack_size(id), |n| n as u32)
}

// ---- parsing ---------------------------------------------------------------------------

fn word(t: Option<&Tok>) -> Option<String> {
    match t {
        Some(Tok::Word(w)) => Some(w.to_ascii_lowercase()),
        _ => None,
    }
}

/// `integer`, `boolean`, ... after `parsed as`: the kind and tokens used.
pub fn parse_kind(toks: &[Tok], at: usize) -> Option<(Kind, usize)> {
    let k = match word(toks.get(at))?.as_str() {
        "integer" | "int" => Kind::Integer,
        "boolean" | "bool" => Kind::Boolean,
        "player" => Kind::Player,
        "text" | "string" => Kind::Text,
        _ => return None,
    };
    Some((k, 1))
}

pub fn parsed(e: Expr, k: Kind) -> Expr {
    let ty = match k {
        Kind::Integer => Ty::Number,
        Kind::Boolean => Ty::Boolean,
        Kind::Player => Ty::Player,
        Kind::Text => Ty::Text,
    };
    node(Op::ParseAs(k), vec![e], ty)
}

/// `human-readable` after `formatted`: the number of tokens it uses.
pub fn human_readable(toks: &[Tok], pos: usize) -> Option<usize> {
    if word(toks.get(pos)).as_deref() == Some("human-readable") {
        return Some(1);
    }
    if word(toks.get(pos)).as_deref() == Some("human")
        && matches!(toks.get(pos + 1), Some(t) if t.is_lit("-"))
        && word(toks.get(pos + 2)).as_deref() == Some("readable")
    {
        return Some(3);
    }
    None
}

pub fn human_span_expr(e: Expr) -> Expr {
    node(Op::HumanSpan, vec![e], Ty::Text)
}

impl P<'_> {
    fn x6_lit(&self, off: usize, w: &str) -> bool {
        if self.pos + off >= self.toks.len() {
            self.hit_end.set(true);
        }
        matches!(self.toks.get(self.pos + off), Some(t) if t.is_lit(w))
    }

    /// `of <operand>` with the `of` at `self.pos`.
    fn x6_of(&mut self) -> Result<Expr, String> {
        self.pos += 1;
        if self.pos >= self.toks.len() {
            self.hit_end.set(true);
        }
        self.primary()
    }

    /// Heads: `loot table from key "x"`, `loot context at <loc> ...`,
    /// `loot location|luck of <ctx>`, `looter of <ctx>`, `looted entity of <ctx>`.
    pub(crate) fn w6c5_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let e = match lw {
            "a" | "an" if self.x6_lit(0, "loot") && self.x6_lit(1, "context") => {
                self.pos += 1;
                return self.w6c5_head("loot");
            }
            "loot" if self.x6_lit(0, "table") && (self.x6_lit(1, "from") || self.x6_lit(1, "of")) => {
                self.pos += 2;
                if self.x6_lit(0, "key") {
                    self.pos += 1;
                }
                if self.pos >= self.toks.len() {
                    self.hit_end.set(true);
                }
                let k = self.primary()?;
                node(Op::TableFromKey, vec![k], Ty::Object)
            }
            "loot" if self.x6_lit(0, "context") && self.x6_lit(1, "at") => {
                self.pos += 2;
                let loc = self.primary()?;
                let mut luck = Expr::Num(0.0);
                let mut looter = Expr::Text(vec![]);
                let mut looted = Expr::Text(vec![]);
                while self.x6_lit(0, "with") {
                    let save = self.pos;
                    self.pos += 1;
                    if self.x6_lit(0, "luck") {
                        self.pos += 1;
                        luck = self.sum()?;
                    } else if self.x6_lit(0, "looter") {
                        self.pos += 1;
                        looter = self.primary()?;
                    } else if self.x6_lit(0, "looted") && self.x6_lit(1, "entity") {
                        self.pos += 2;
                        looted = self.primary()?;
                    } else {
                        self.pos = save;
                        break;
                    }
                }
                node(Op::CtxNew, vec![loc, luck, looter, looted], Ty::Object)
            }
            "loot" if self.x6_lit(0, "location") && self.x6_lit(1, "of") => {
                self.pos += 1;
                let c = self.x6_of()?;
                node(Op::CtxLocation, vec![c], Ty::Location)
            }
            "loot" if self.x6_lit(0, "luck") && self.x6_lit(1, "of") => {
                self.pos += 1;
                let c = self.x6_of()?;
                node(Op::CtxLuck, vec![c], Ty::Number)
            }
            "looter" if self.x6_lit(0, "of") => {
                let c = self.x6_of()?;
                node(Op::CtxLooter, vec![c], Ty::Object)
            }
            "looted" if self.x6_lit(0, "entity") && self.x6_lit(1, "of") => {
                self.pos += 1;
                let c = self.x6_of()?;
                node(Op::CtxLooted, vec![c], Ty::Object)
            }
            _ => return Ok(None),
        };
        Ok(Some(e))
    }

    /// `<item> with custom model data 5`, `with enchantment glint true`,
    /// `with item model "ns:key"`.
    pub(crate) fn w6c5_post(&mut self, mut e: Expr) -> Result<Expr, String> {
        while self.x6_lit(0, "with") && matches!(e.ty(self.env), Ty::Item) {
            let save = self.pos;
            self.pos += 1;
            let comp = match word(self.toks.get(self.pos)).as_deref() {
                Some("custom") if self.x6_lit(1, "model") && self.x6_lit(2, "data") => {
                    self.pos += 3;
                    Some(Comp::CustomModelData)
                }
                Some("enchantment" | "enchant") if self.x6_lit(1, "glint") => {
                    self.pos += 2;
                    Some(Comp::Glint)
                }
                Some("glint") => {
                    self.pos += 1;
                    Some(Comp::Glint)
                }
                Some("item") if self.x6_lit(1, "model") => {
                    self.pos += 2;
                    Some(Comp::Model)
                }
                Some("model") => {
                    self.pos += 1;
                    Some(Comp::Model)
                }
                _ => None,
            };
            let Some(c) = comp else {
                self.pos = save;
                break;
            };
            if c == Comp::Glint && self.x6_lit(0, "override") {
                self.pos += 1;
            }
            let arg = match c {
                Comp::Glint => {
                    let on = match word(self.toks.get(self.pos)).as_deref() {
                        Some("true" | "enabled" | "on" | "shown" | "visible") => true,
                        Some("false" | "disabled" | "off" | "hidden") => false,
                        _ => {
                            if self.pos >= self.toks.len() {
                                self.hit_end.set(true);
                            }
                            return Err("expected true/false, enabled/disabled or shown/hidden".into());
                        }
                    };
                    self.pos += 1;
                    Expr::Bool(on)
                }
                Comp::CustomModelData => {
                    if self.pos >= self.toks.len() {
                        self.hit_end.set(true);
                    }
                    self.sum()?
                }
                Comp::Model => {
                    if self.pos >= self.toks.len() {
                        self.hit_end.set(true);
                    }
                    self.primary()?
                }
            };
            e = node(Op::ItemWith(c), vec![e, arg], Ty::Item);
        }
        Ok(e)
    }
}
