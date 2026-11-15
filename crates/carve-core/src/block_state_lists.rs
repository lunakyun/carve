//! Wave 6, chunk 4: block-state properties (honey level, brushing stage, age
//! of a block), `first N elements of`, `last N elements of`,
//! `elements N to M of`, `dequeue`, `banner pattern item of` and
//! `persistent data keys of`.
//!
//! Every expression is one boxed [`Node`] (`Expr::W6C4`) and every change one
//! boxed [`W6Eff`] (`Effect::W6C4`), so the hot enums stay small. The engine
//! side is `#[cold]`. Block properties go through the existing
//! `Host::c6_block_state` / `Host::c6_set` calls, so there is no new host call.

use crate::block_ops::BlockPos;
use crate::compile::Effect;
use crate::data::block_states::{BLOCKS, PROPS};
use crate::engine::{Engine, Exec};
use crate::entity_block_properties::{Set, decode_state, encode_state};
use crate::expr::{Env, Expr, P, Ty, parse_expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::Value;

/// The block-state properties these expressions read and write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prop {
    Honey,
    Brushing,
    Age,
}

impl Prop {
    /// The block-state property name.
    pub fn key(self) -> &'static str {
        match self {
            Prop::Honey => "honey_level",
            Prop::Brushing => "dusted",
            Prop::Age => "age",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Prop::Honey => "honey level",
            Prop::Brushing => "brushing stage",
            Prop::Age => "age",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// A block property of a block; `true` is the maximum (`max age of`).
    Prop(Prop, bool),
    /// `first N elements of {x::*}`: args are the count and the list.
    First,
    /// `last N elements of {x::*}`.
    Last,
    /// `elements N to M of {x::*}`: args are N, M and the list.
    Range,
    /// `dequeue {q}`: args are the queue variable and an optional count.
    Dequeue,
    /// `banner pattern item of "creeper"`.
    BannerItem,
    /// `persistent data keys of <thing>`.
    DataKeys,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub op: Op,
    pub args: Vec<Expr>,
    pub ty: Ty,
}

impl Node {
    fn changeable(&self) -> bool {
        matches!(self.op, Op::Prop(_, false))
    }

    pub fn name(&self) -> &'static str {
        match self.op {
            Op::Prop(p, _) => p.name(),
            Op::First | Op::Last | Op::Range => "elements",
            Op::Dequeue => "dequeue",
            Op::BannerItem => "banner pattern item",
            Op::DataKeys => "persistent data keys",
        }
    }
}

fn node(op: Op, args: Vec<Expr>, ty: Ty) -> Expr {
    Expr::W6C4(Box::new(Node { op, args, ty }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Set,
    Add,
    Remove,
    Reset,
}

/// `set honey level of <block> to 3` and friends.
#[derive(Debug, Clone, PartialEq)]
pub struct W6Eff {
    pub change: Change,
    pub target: Node,
    pub value: Option<Expr>,
}

// ---------------------------------------------------------------- data

/// Banner pattern type and the item that unlocks it in a loom.
pub static BANNER_PATTERN_ITEMS: &[(&str, &str)] = &[
    ("bordure_indented", "bordure_indented_banner_pattern"),
    ("creeper", "creeper_banner_pattern"),
    ("field_masoned", "field_masoned_banner_pattern"),
    ("flow", "flow_banner_pattern"),
    ("flower", "flower_banner_pattern"),
    ("globe", "globe_banner_pattern"),
    ("guster", "guster_banner_pattern"),
    ("mojang", "mojang_banner_pattern"),
    ("piglin", "piglin_banner_pattern"),
    ("skull", "skull_banner_pattern"),
];

/// The pattern item of a banner pattern type (`creeper`, `minecraft:flow`,
/// `Field Masoned`). `None` for the patterns that have no item.
pub fn banner_pattern_item(name: &str) -> Option<&'static str> {
    let n = name.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    let n = n.strip_prefix("minecraft:").unwrap_or(&n);
    BANNER_PATTERN_ITEMS
        .iter()
        .find(|(k, _)| *k == n)
        .map(|(_, v)| *v)
}

/// The largest value of a block-state property of `id`, if the block has it.
pub fn prop_max(id: &str, prop: Prop) -> Option<i64> {
    let i = BLOCKS.binary_search_by(|(n, _)| (*n).cmp(id)).ok()?;
    BLOCKS[i]
        .1
        .iter()
        .map(|&p| &PROPS[usize::from(p)])
        .find(|(n, _)| *n == prop.key())
        .and_then(|(_, vals)| vals.iter().filter_map(|v| v.parse::<i64>().ok()).max())
}

/// The value of a block-state property.
pub fn prop_value(id: &str, state: u16, first: u16, prop: Prop) -> Option<i64> {
    decode_state(id, state, first)
        .into_iter()
        .find(|(n, _)| *n == prop.key())
        .and_then(|(_, v)| v.parse().ok())
}

/// `first N` / `last N` of a list.
pub fn take_ends(items: &[Value], n: usize, first: bool) -> Vec<Value> {
    let n = n.min(items.len());
    if first {
        items[..n].to_vec()
    } else {
        items[items.len() - n..].to_vec()
    }
}

/// `elements N to M` of a list (1-based, inclusive; reversed bounds swap).
pub fn range(items: &[Value], a: i64, b: i64) -> Vec<Value> {
    let (a, b) = if a <= b { (a, b) } else { (b, a) };
    let a = a.max(1) as usize;
    let b = (b.max(0) as usize).min(items.len());
    if a > b {
        return Vec::new();
    }
    items[a - 1..b].to_vec()
}

// ---------------------------------------------------------------- parsing

fn word(t: Option<&Tok>) -> Option<String> {
    match t {
        Some(Tok::Word(w)) => Some(w.to_ascii_lowercase()),
        _ => None,
    }
}

fn is(t: Option<&Tok>, w: &str) -> bool {
    t.is_some_and(|t| t.is_lit(w))
}

/// Words that start one of our heads (the cheap test of `may_target`).
const HEADS: [&str; 10] = [
    "honey", "beehive", "bee", "brushing", "age", "max", "maximum", "the", "brush", "hive",
];

impl P<'_> {
    fn x6_at(&self, n: usize) -> Option<&Tok> {
        if self.pos + n >= self.toks.len() {
            self.hit_end.set(true);
        }
        self.toks.get(self.pos + n)
    }

    pub(crate) fn x6_lit(&self, n: usize, w: &str) -> bool {
        is(self.x6_at(n), w)
    }

    fn x6_word(&self, n: usize) -> Option<String> {
        word(self.x6_at(n))
    }

    /// A number token or a variable: one token only, so `3 elements` stays apart.
    fn x6_count(&mut self, n: usize) -> Result<Option<Expr>, String> {
        if !matches!(self.x6_at(n), Some(Tok::Num(_) | Tok::Var(_))) {
            return Ok(None);
        }
        let at = self.pos + n;
        let e = parse_expr(&self.toks[at..=at], self.env)?;
        Ok(Some(e))
    }

    fn x6_list(&mut self, what: &str) -> Result<Expr, String> {
        if !self.x6_lit(0, "of") {
            return Err(format!("expected 'of {{list::*}}' after '{what}'"));
        }
        if let Some(Tok::Var(_)) = self.x6_at(1) {
            let at = self.pos + 1;
            let e = parse_expr(&self.toks[at..=at], self.env)?;
            if matches!(&e, Expr::Var(v) if v.list) {
                self.pos += 2;
                return Ok(e);
            }
        }
        Err(format!("'{what} of' needs a list variable like {{x::*}}"))
    }

    fn x6_of_primary(&mut self, what: &str) -> Result<Expr, String> {
        if !(self.x6_lit(0, "of") || self.x6_lit(0, "in")) {
            return Err(format!("expected 'of' after '{what}'"));
        }
        self.pos += 1;
        self.primary()
    }

    /// Expression heads of this module. `self.pos` is after the first word `lw`.
    pub(crate) fn w6c4_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let start = self.pos;
        let r = self.x6_head_inner(lw);
        if matches!(r, Ok(None)) {
            self.pos = start;
        }
        r
    }

    fn x6_head_inner(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        // `max age of`, `maximum honey level of`
        if matches!(lw, "max" | "maximum") {
            let Some(p) = self.x6_prop_words()? else {
                return Ok(None);
            };
            let b = self.x6_of_primary(p.name())?;
            return Ok(Some(node(Op::Prop(p, true), vec![b], Ty::Number)));
        }
        let mut lw = lw.to_string();
        // `beehive honey level`, `bee hive honey level`, `bee nest honey level`
        if matches!(lw.as_str(), "beehive" | "bee") {
            if lw == "bee" && matches!(self.x6_word(0).as_deref(), Some("hive" | "nest")) {
                self.pos += 1;
            }
            lw = match self.x6_word(0) {
                Some(w) => {
                    self.pos += 1;
                    w
                }
                None => return Ok(None),
            };
        }
        if matches!(lw.as_str(), "honey" | "brushing" | "age") {
            let Some(p) = self.x6_prop_from(&lw)? else {
                return Ok(None);
            };
            let b = self.x6_of_primary(p.name())?;
            return Ok(Some(node(Op::Prop(p, false), vec![b], Ty::Number)));
        }
        match lw.as_str() {
            "first" | "last" => {
                let Some(n) = self.x6_count(0)? else {
                    return Ok(None);
                };
                if !matches!(self.x6_word(1).as_deref(), Some("elements")) {
                    return Ok(None);
                }
                self.pos += 2;
                let list = self.x6_list("elements")?;
                let op = if lw == "first" { Op::First } else { Op::Last };
                Ok(Some(node(op, vec![n, list], Ty::List)))
            }
            "elements" => {
                let Some(a) = self.x6_count(0)? else {
                    return Ok(None);
                };
                if !self.x6_lit(1, "to") {
                    return Ok(None);
                }
                let Some(b) = self.x6_count(2)? else {
                    return Ok(None);
                };
                self.pos += 3;
                let list = self.x6_list("elements N to M")?;
                Ok(Some(node(Op::Range, vec![a, b, list], Ty::List)))
            }
            "dequeue" | "de" => {
                if lw == "de" {
                    if !(self.x6_lit(0, "-") && self.x6_lit(1, "queue")) {
                        return Ok(None);
                    }
                    self.pos += 2;
                }
                // `dequeue {q}`, `dequeue first 2 elements of {q}`, `dequeue 2 elements of {q}`
                let mut n = None;
                let mut i = 0;
                if self.x6_lit(i, "first") {
                    i += 1;
                }
                if matches!(self.x6_word(i + 1).as_deref(), Some("elements" | "element")) {
                    if let Some(c) = self.x6_count(i)? {
                        n = Some(c);
                        self.pos += i + 2;
                        if self.x6_lit(0, "of") || self.x6_lit(0, "from") {
                            self.pos += 1;
                        }
                    }
                } else if i > 0 && self.x6_word(i).as_deref() == Some("element") {
                    self.pos += i + 1;
                    if self.x6_lit(0, "of") || self.x6_lit(0, "from") {
                        self.pos += 1;
                    }
                }
                let Some(Tok::Var(_)) = self.x6_at(0) else {
                    return Ok(None);
                };
                let at = self.pos;
                let q = parse_expr(&self.toks[at..=at], self.env)?;
                if !matches!(&q, Expr::Var(v) if !v.list) {
                    return Err("'dequeue' needs a queue variable like {_queue}".into());
                }
                self.pos += 1;
                let mut args = vec![q];
                let ty = if n.is_some() { Ty::List } else { Ty::Object };
                args.extend(n);
                Ok(Some(node(Op::Dequeue, args, ty)))
            }
            "banner" if self.x6_lit(0, "pattern") && self.x6_lit(1, "item") => {
                self.pos += 2;
                let e = self.x6_of_primary("banner pattern item")?;
                Ok(Some(node(Op::BannerItem, vec![e], Ty::Item)))
            }
            "all" | "persistent" | "the" => {
                let mut i = 0;
                if matches!(lw.as_str(), "all" | "the") {
                    if self.x6_lit(i, "the") {
                        i += 1;
                    }
                    if !self.x6_lit(i, "persistent") {
                        return Ok(None);
                    }
                    i += 1;
                }
                if !(self.x6_lit(i, "data")
                    && matches!(self.x6_word(i + 1).as_deref(), Some("keys" | "tags")))
                {
                    return Ok(None);
                }
                self.pos += i + 2;
                let e = self.x6_of_primary("persistent data keys")?;
                Ok(Some(node(Op::DataKeys, vec![e], Ty::List)))
            }
            _ => Ok(None),
        }
    }

    /// After `max`: the words of a property (`honey level`, `brushing stage`, `age`).
    fn x6_prop_words(&mut self) -> Result<Option<Prop>, String> {
        let mut i = 0;
        if matches!(self.x6_word(0).as_deref(), Some("beehive" | "bee")) {
            i += 1;
            if matches!(self.x6_word(i).as_deref(), Some("hive" | "nest")) {
                i += 1;
            }
        }
        let Some(first) = self.x6_word(i) else {
            return Ok(None);
        };
        self.pos += i + 1;
        let r = self.x6_prop_from(&first)?;
        if r.is_none() {
            self.pos -= i + 1;
        }
        Ok(r)
    }

    /// The word `first` was just read: finish `honey level`, `brushing stage|progress`, `age`.
    fn x6_prop_from(&mut self, first: &str) -> Result<Option<Prop>, String> {
        Ok(match first {
            "honey" if self.x6_lit(0, "level") => {
                self.pos += 1;
                Some(Prop::Honey)
            }
            "brushing" if matches!(self.x6_word(0).as_deref(), Some("stage" | "progress")) => {
                self.pos += 1;
                Some(Prop::Brushing)
            }
            "age" if self.x6_lit(0, "of") => Some(Prop::Age),
            _ => None,
        })
    }
}

/// Does the line start with one of our list expressions (`first 2 elements of ...`)?
pub fn is_list_expr(toks: &[Tok]) -> bool {
    let w = word(toks.first());
    match w.as_deref() {
        Some("first" | "last") => {
            matches!(toks.get(1), Some(Tok::Num(_) | Tok::Var(_)))
                && word(toks.get(2)).as_deref() == Some("elements")
        }
        Some("elements") => matches!(toks.get(1), Some(Tok::Num(_) | Tok::Var(_))) && is(toks.get(2), "to"),
        Some("dequeue") => toks
            .iter()
            .any(|t| matches!(t, Tok::Word(w) if w.eq_ignore_ascii_case("elements"))),
        Some("all" | "the" | "persistent") => {
            let i = usize::from(is(toks.get(1), "persistent") || is(toks.get(1), "the"))
                + usize::from(w.as_deref() != Some("persistent"));
            let i = i.min(toks.len());
            toks[i.saturating_sub(1)..]
                .windows(2)
                .take(2)
                .any(|p| is(p.first(), "data") && matches!(word(p.get(1)).as_deref(), Some("keys" | "tags")))
        }
        _ => false,
    }
}

// ---------------------------------------------------------------- effects

/// `set/add to/remove from/reset` of a block property. `None`: not ours.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Option<Result<Effect, String>> {
    let verb = word(toks.first())?;
    let (change, target, value) = match verb.as_str() {
        "set" => {
            let to = toks.iter().position(|t| t.is_lit("to"))?;
            (Change::Set, toks.get(1..to)?, toks.get(to + 1..))
        }
        "add" | "remove" => {
            let kw = if verb == "add" { "to" } else { "from" };
            let at = toks.iter().rposition(|t| t.is_lit(kw))?;
            let ch = if verb == "add" {
                Change::Add
            } else {
                Change::Remove
            };
            (ch, toks.get(at + 1..)?, toks.get(1..at))
        }
        "reset" | "clear" => (Change::Reset, toks.get(1..)?, None),
        _ => return None,
    };
    let first = word(target.first())?;
    if !HEADS.contains(&first.as_str()) {
        return None;
    }
    let Ok(Expr::W6C4(n)) = parse_expr(target, env) else {
        return None;
    };
    if !n.changeable() {
        return None;
    }
    let value = match (change, value) {
        (Change::Reset, _) => None,
        (_, Some(v)) if !v.is_empty() => match parse_expr(v, env) {
            Ok(e) => Some(e),
            Err(m) => return Some(Err(m)),
        },
        _ => return Some(Err("missing the value".into())),
    };
    Some(Ok(Effect::W6C4(Box::new(W6Eff {
        change,
        target: *n,
        value,
    }))))
}

// ---------------------------------------------------------------- engine

fn block_of(v: &Value, host: &mut dyn Host) -> Option<BlockPos> {
    match v {
        Value::Location { x, y, z, world } => {
            let world = world.clone().or_else(|| host.default_world());
            crate::block_ops::pos_of(*x, *y, *z, world)
        }
        _ => None,
    }
}

/// The key a data target is stored under; see `expr_numbers_metadata::data_key`.
fn data_prefix(id: &str) -> String {
    format!("\u{1}pd:{id}:")
}

impl Engine {
    #[cold]
    #[inline(never)]
    pub(crate) fn w6c4_expr(
        &mut self,
        n: &Node,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        let arg = |i: usize| n.args.get(i).ok_or_else(|| "missing value".to_string());
        Ok(match n.op {
            Op::Prop(p, max) => {
                let v = self.eval(arg(0)?, ex, host)?;
                let Some(at) = block_of(&v, host) else {
                    return Ok(Value::None);
                };
                let Some((id, state, first)) = host.c6_block_state(&at) else {
                    return Ok(Value::None);
                };
                let got = if max {
                    prop_max(&id, p)
                } else {
                    prop_value(&id, state, first, p)
                };
                got.map_or(Value::None, |n| Value::Num(n as f64))
            }
            Op::First | Op::Last | Op::Range => {
                let (list, nums) = match n.op {
                    Op::Range => (arg(2)?, 2),
                    _ => (arg(1)?, 1),
                };
                let mut b = [0i64; 2];
                for (i, slot) in b.iter_mut().enumerate().take(nums) {
                    let x = self.num(arg(i)?, ex, host)?;
                    if x.fract() != 0.0 || x < 0.0 {
                        return Ok(Value::None);
                    }
                    *slot = x as i64;
                }
                let Expr::Var(v) = list else {
                    return Ok(Value::None);
                };
                let items: Vec<Value> = self
                    .list_items(v, ex, host)?
                    .into_iter()
                    .map(|(_, x)| x)
                    .collect();
                Value::List(match n.op {
                    Op::First => take_ends(&items, b[0] as usize, true),
                    Op::Last => take_ends(&items, b[0] as usize, false),
                    _ => range(&items, b[0], b[1]),
                })
            }
            Op::Dequeue => {
                let Expr::Var(v) = arg(0)? else {
                    return Ok(Value::None);
                };
                let count = match n.args.get(1) {
                    Some(e) => Some(self.num(e, ex, host)?.max(0.0) as usize),
                    None => None,
                };
                let k = self.key(v, ex, host)?.to_string();
                let cur = self.get_var(v, ex, host)?;
                let Value::Queue(mut q) = cur else {
                    return Ok(if count.is_some() {
                        Value::List(Vec::new())
                    } else {
                        Value::None
                    });
                };
                let take = count.unwrap_or(1).min(q.len());
                let out: Vec<Value> = q.drain(..take).collect();
                self.store(v.local, &k, Value::Queue(q), ex, host);
                match count {
                    Some(_) => Value::List(out),
                    None => out.into_iter().next().unwrap_or(Value::None),
                }
            }
            Op::BannerItem => {
                let v = self.eval(arg(0)?, ex, host)?;
                match banner_pattern_item(&v.to_string()) {
                    Some(id) => Value::Item {
                        id: id.into(),
                        count: 1,
                        meta: None,
                    },
                    None => Value::None,
                }
            }
            Op::DataKeys => {
                let t = self.eval(arg(0)?, ex, host)?;
                let Some(id) = crate::expr_numbers_metadata::target_id(&t) else {
                    return Ok(Value::None);
                };
                let prefix = data_prefix(&id);
                let mut keys: Vec<String> = self
                    .vars
                    .keys_with_prefix(&prefix)
                    .into_iter()
                    .filter_map(|k| k.strip_prefix(&prefix).map(str::to_string))
                    .collect();
                keys.sort();
                Value::List(keys.into_iter().map(Value::Text).collect())
            }
        })
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn w6c4_effect(
        &mut self,
        e: &W6Eff,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        let Op::Prop(p, _) = e.target.op else {
            return Ok(());
        };
        let Some(b) = e.target.args.first() else {
            return Ok(());
        };
        let v = self.eval(b, ex, host)?;
        let Some(at) = block_of(&v, host) else {
            return Ok(());
        };
        let Some((id, state, first)) = host.c6_block_state(&at) else {
            return Ok(());
        };
        let (Some(cur), Some(max)) = (prop_value(&id, state, first, p), prop_max(&id, p)) else {
            return Ok(());
        };
        let new = match (e.change, &e.value) {
            (Change::Reset, _) => 0.0,
            (ch, Some(val)) => {
                let x = self.num(val, ex, host)?;
                match ch {
                    Change::Set => x,
                    Change::Add => cur as f64 + x,
                    _ => cur as f64 - x,
                }
            }
            _ => return Ok(()),
        };
        let n = (new.round() as i64).clamp(0, max);
        let s = encode_state(&id, first, state, &[(p.key().to_string(), n.to_string())])?;
        host.c6_set(Set::BlockState(&at, s));
        Ok(())
    }
}
