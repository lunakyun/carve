//! Skript features that were missing: inventory values (a chest menu you build
//! in a variable and open later), editing the held item, `sorted ... by`,
//! regex replace, `quit message`, `push ... forward` / `away from`,
//! `set velocity of ... to <vector>` and `{t}'s minutes`.
//!
//! Like `inventories.rs`, the lines are parsed by hand. The compiler calls
//! [`parse_effect`] and [`parse_cond`] before its other parsers, and the
//! expression parser calls `P::gaps_head`. The runtime is in
//! `engine/gaps_eval.rs`. Every node is boxed, so the hot enums stay small.

use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Map, Value as Json, json};

use crate::compile::{Cond, Effect};
use crate::expr::{Env, Expr, P, Ty, VarRef, parse_expr, parse_varref};
use crate::inventories::{InvTarget, inv_target, inv_target_len};
use crate::lexer::Tok;
use crate::named_types::T_INVENTORY;
use crate::value::Value;

/// The inventory type, for `Expr::ty`.
pub const INV_TY: Ty = Ty::T1(T_INVENTORY);

// ---------- inventory values ----------

/// A chest inventory value. Copies share the `id`, so a menu opened from a
/// variable can be updated through that variable later.
#[derive(Debug, Clone, PartialEq)]
pub struct InvVal {
    /// Never 0 for a built inventory. 0: a menu that was not built from a value.
    pub id: u64,
    pub rows: u8,
    pub title: String,
    /// `rows * 9` slots. `Value::None` is an empty slot, the rest are items.
    pub slots: Vec<Value>,
    /// The open menu of this player (uuid) holds the contents, not `slots`
    /// (`event-inventory`).
    pub viewer: Option<String>,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// A fresh inventory id. Seeded from the clock, so ids saved in
/// the saved variables do not come back after a restart.
pub fn next_id(clock_ms: i64) -> u64 {
    if NEXT_ID.load(Ordering::Relaxed) == 0 {
        let seed = (clock_ms.max(1) as u64).wrapping_mul(1024);
        let _ = NEXT_ID.compare_exchange(0, seed, Ordering::Relaxed, Ordering::Relaxed);
    }
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

impl InvVal {
    pub fn new(id: u64, rows: u8, title: String) -> Self {
        InvVal {
            id,
            rows,
            title,
            slots: vec![Value::None; usize::from(rows) * 9],
            viewer: None,
        }
    }

    pub fn size(&self) -> usize {
        usize::from(self.rows) * 9
    }

    /// Same inventory: the same id, or for unnamed menus the same viewer.
    pub fn same(&self, o: &InvVal) -> bool {
        if self.id != 0 || o.id != 0 {
            return self.id == o.id;
        }
        self.viewer.is_some() && self.viewer == o.viewer
    }

    pub fn push_to(&self, out: &mut String) {
        out.push_str("chest inventory named \"");
        out.push_str(&self.title);
        out.push('"');
    }

    pub fn to_json(&self) -> Json {
        let mut slots = Map::new();
        for (i, v) in self.slots.iter().enumerate() {
            if let Some(j) = item_json(v) {
                slots.insert(i.to_string(), j);
            }
        }
        json!({"inventory": {"id": self.id, "rows": self.rows, "title": self.title, "slots": slots}})
    }

    pub fn from_json(o: &Map<String, Json>) -> Option<Value> {
        let i = o.get("inventory")?.as_object()?;
        let rows = i.get("rows")?.as_u64()?.clamp(1, 6) as u8;
        let mut inv = InvVal::new(
            i.get("id").and_then(Json::as_u64).unwrap_or(0),
            rows,
            i.get("title")
                .and_then(Json::as_str)
                .unwrap_or("Chest")
                .to_string(),
        );
        if let Some(s) = i.get("slots").and_then(Json::as_object) {
            for (k, v) in s {
                if let (Ok(n), Some(item)) = (k.parse::<usize>(), item_from_json(v))
                    && let Some(slot) = inv.slots.get_mut(n)
                {
                    *slot = item;
                }
            }
        }
        Some(Value::Inventory(Box::new(inv)))
    }
}

fn item_json(v: &Value) -> Option<Json> {
    let Value::Item { id, count, meta } = v else {
        return None;
    };
    Some(match meta {
        Some(m) => json!({"id": id, "count": count, "meta": m.to_json()}),
        None => json!({"id": id, "count": count}),
    })
}

fn item_from_json(j: &Json) -> Option<Value> {
    let o = j.as_object()?;
    Some(Value::Item {
        id: o.get("id")?.as_str()?.to_string(),
        count: o.get("count").and_then(Json::as_u64).unwrap_or(1).clamp(1, 99) as u32,
        meta: o.get("meta").and_then(crate::item_meta::ItemMeta::from_json),
    })
}

/// An item for a slot: `None` for nothing or air.
pub fn slot_item(v: Value) -> Result<Value, String> {
    match v {
        Value::None => Ok(Value::None),
        Value::Item { ref id, .. } if id == "air" => Ok(Value::None),
        Value::Item { id, count, meta } => Ok(Value::Item {
            id,
            count: count.clamp(1, 99),
            meta,
        }),
        other => Err(format!("expected an item, got {} ({other})", other.type_name())),
    }
}

// ---------- syntax tree ----------

#[derive(Debug, Clone, PartialEq)]
pub enum GapExpr {
    /// `chest inventory with 3 rows named "x"`
    NewChest { rows: Option<Expr>, title: Option<Expr> },
    /// `event-inventory` (false) / `clicked inventory` (true: `<none>` when
    /// the click was in the player's own inventory below the menu).
    EventInv(bool),
    /// `slot N of <inventory value>`
    Slot { slot: Expr, inv: Expr },
    /// `items in <inventory value>`
    Items(Expr),
    /// `items in player's inventory`
    PlayerItems(InvTarget),
    /// `x with all regex "a+" replaced with "b"`
    Regex {
        text: Expr,
        pat: Expr,
        with: Expr,
        all: bool,
    },
}

impl GapExpr {
    pub fn ty(&self) -> Ty {
        match self {
            GapExpr::NewChest { .. } | GapExpr::EventInv(_) => INV_TY,
            GapExpr::Slot { .. } => Ty::Item,
            GapExpr::Items(_) | GapExpr::PlayerItems(_) => Ty::List,
            GapExpr::Regex { .. } => Ty::Text,
        }
    }
}

/// What to change on the held item.
#[derive(Debug, Clone, PartialEq)]
pub enum HeldOp {
    /// `set name of player's tool to "x"`, `reset name of ...` (None).
    Name(Option<Expr>),
    /// `set lore of player's tool to "a" and "b"`, `clear lore of ...` (empty).
    Lore(Vec<Expr>),
    /// `add "line" to lore of player's tool`
    AddLore(Vec<Expr>),
    /// `enchant player's tool with sharpness 5 and unbreaking 3`
    Enchant(Vec<Expr>),
    /// `disenchant player's tool`
    Disenchant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GapEff {
    SetSlot {
        slot: Expr,
        inv: Expr,
        item: Expr,
    },
    Open {
        inv: Expr,
        to: Expr,
    },
    /// `clear {_gui}`; on a variable that holds no inventory it deletes it.
    Clear(Expr),
    Held {
        who: Expr,
        op: HeldOp,
    },
    QuitMessage(Expr),
    /// `replace all regex "a" in {x} with "b"`
    Regex {
        pat: Expr,
        var: VarRef,
        with: Expr,
        all: bool,
    },
    /// `set velocity of <entity> to <vector>`
    SetVelocity {
        who: Expr,
        to: Expr,
    },
    /// `push <entity> forward` (false) / `backward` (true).
    PushFacing {
        who: Expr,
        back: bool,
        speed: Option<Expr>,
    },
    /// `push <entity> away from <place>` / `towards <place>` (true).
    PushFrom {
        who: Expr,
        from: Expr,
        toward: bool,
        speed: Option<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GapCond {
    pub inv: Expr,
    pub item: Expr,
    pub negate: bool,
}

fn gap(e: GapExpr) -> Expr {
    Expr::Gap(Box::new(e))
}

fn eff(e: GapEff) -> Effect {
    Effect::Gap(Box::new(e))
}

fn is(t: Option<&Tok>, w: &str) -> bool {
    t.is_some_and(|t| t.is_lit(w))
}

fn pos_of(toks: &[Tok], from: usize, words: &[&str]) -> Option<usize> {
    (from..toks.len()).find(|&i| words.iter().any(|w| is(toks.get(i), w)))
}

fn rpos_of(toks: &[Tok], words: &[&str]) -> Option<usize> {
    (0..toks.len())
        .rev()
        .find(|&i| words.iter().any(|w| is(toks.get(i), w)))
}

fn sub(toks: &[Tok], a: usize, b: usize, env: &Env) -> Option<Expr> {
    let t = toks.get(a..b)?;
    if t.is_empty() {
        return None;
    }
    parse_expr(t, env).ok()
}

fn player_like(e: &Expr, env: &Env) -> bool {
    matches!(e.ty(env), Ty::Player | Ty::Object)
}

/// `{_gui}`, `event-inventory`, `clicked inventory`, `the event inventory`:
/// the whole token list must be the inventory.
fn inv_value(toks: &[Tok], env: &Env) -> Option<Expr> {
    let toks = match toks {
        [t, rest @ ..] if t.is_lit("the") => rest,
        t => t,
    };
    match toks {
        [Tok::Var(raw)] => {
            let v = parse_varref(raw, env).ok()?;
            (!v.list).then_some(Expr::Var(v))
        }
        [w] if w.is_lit("event-inventory") && env.has_inv && !env.in_function => {
            Some(gap(GapExpr::EventInv(false)))
        }
        [a, b] if b.is_lit("inventory") && env.has_inv && !env.in_function => {
            if a.is_lit("clicked") {
                Some(gap(GapExpr::EventInv(true)))
            } else if a.is_lit("event") {
                Some(gap(GapExpr::EventInv(false)))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// `player's tool`, `{_p}'s held item`, `tool of player`, `held item of player`.
/// Gives the player expression.
fn held(toks: &[Tok], env: &Env) -> Option<Expr> {
    let toks = match toks {
        [t, rest @ ..] if t.is_lit("the") => rest,
        t => t,
    };
    let tails: &[&[&str]] = &[
        &["tool"],
        &["held", "item"],
        &["item", "in", "hand"],
        &["main", "hand", "item"],
    ];
    if let Some(s) = toks.iter().position(|t| t.is_lit("'s")) {
        let tail = toks.get(s + 1..)?;
        if tails
            .iter()
            .any(|w| w.len() == tail.len() && w.iter().zip(tail).all(|(w, t)| t.is_lit(w)))
        {
            let p = parse_expr(toks.get(..s)?, env).ok()?;
            return player_like(&p, env).then_some(p);
        }
        return None;
    }
    for w in [&["tool"][..], &["held", "item"]] {
        if toks.len() > w.len() + 1
            && w.iter().zip(toks).all(|(w, t)| t.is_lit(w))
            && is(toks.get(w.len()), "of")
        {
            let p = parse_expr(toks.get(w.len() + 1..)?, env).ok()?;
            return player_like(&p, env).then_some(p);
        }
    }
    None
}

/// `a, b and c`: one expression per part (top level commas and `and`).
fn parts(toks: &[Tok], env: &Env) -> Result<Vec<Expr>, String> {
    let mut out = Vec::new();
    for p in toks.split(|t| t.is_lit(",") || t.is_lit("and")) {
        if p.is_empty() {
            return Err("expected a value between the commas".into());
        }
        out.push(parse_expr(p, env)?);
    }
    Ok(out)
}

/// `with speed 2` / `at speed 2` at the end. Returns where it starts.
fn speed_tail(toks: &[Tok], from: usize, env: &Env) -> Result<(usize, Option<Expr>), String> {
    let Some(w) = pos_of(toks, from, &["with", "at", "using"]) else {
        return Ok((toks.len(), None));
    };
    let mut i = w + 1;
    while is(toks.get(i), "the") || is(toks.get(i), "a") {
        i += 1;
    }
    if !(is(toks.get(i), "speed") || is(toks.get(i), "velocity") || is(toks.get(i), "force")) {
        return Err("expected 'with speed <number>'".into());
    }
    let n = sub(toks, i + 1, toks.len(), env).ok_or("expected a number after 'speed'")?;
    Ok((w, Some(n)))
}

// ---------- effects ----------

/// `Ok(None)`: not one of these lines. `Err`: it is one, but wrong.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    Ok(match first.to_ascii_lowercase().as_str() {
        "set" => set(toks, env)?,
        "open" => open(toks, env),
        "clear" | "reset" | "delete" | "remove" => clear(toks, env),
        "add" => add_lore(toks, env)?,
        "enchant" => enchant(toks, env)?,
        "disenchant" | "unenchant" => held(toks.get(1..).unwrap_or_default(), env).map(|who| GapEff::Held {
            who,
            op: HeldOp::Disenchant,
        }),
        "replace" | "regex" => regex_replace(toks, env)?,
        "push" => push(toks, env)?,
        _ => None,
    }
    .map(eff))
}

fn set(toks: &[Tok], env: &Env) -> Result<Option<GapEff>, String> {
    let Some(to) = rpos_of(toks, &["to"]) else {
        return Ok(None);
    };
    let mut i = 1;
    if is(toks.get(i), "the") {
        i += 1;
    }
    // `set slot N of {_gui} to <item>`
    if is(toks.get(1), "slot")
        && let Some(of) = pos_of(toks, 2, &["of"])
        && of < to
        && let Some(inv) = inv_value(toks.get(of + 1..to).unwrap_or_default(), env)
    {
        let slot = sub(toks, 2, of, env).ok_or("expected a slot number")?;
        let item = sub(toks, to + 1, toks.len(), env).ok_or("expected an item after 'to'")?;
        return Ok(Some(GapEff::SetSlot { slot, inv, item }));
    }
    // `set quit message to "x"`
    if (is(toks.get(i), "quit") || is(toks.get(i), "leave")) && is(toks.get(i + 1), "message") && to == i + 2
    {
        if env.trigger_name != "'on quit'" || env.in_function {
            return Err("'quit message' can only be set in 'on quit'".into());
        }
        let t = sub(toks, to + 1, toks.len(), env).ok_or("expected a text after 'to'")?;
        return Ok(Some(GapEff::QuitMessage(t)));
    }
    // `set velocity of <entity> to <vector>`
    if is(toks.get(i), "velocity") && is(toks.get(i + 1), "of") {
        let Some(who) = sub(toks, i + 2, to, env) else {
            return Ok(None);
        };
        let v = sub(toks, to + 1, toks.len(), env).ok_or("expected a vector after 'to'")?;
        if !matches!(v.ty(env), Ty::Vector | Ty::Object) {
            return Err("the velocity must be a vector, like vector(0, 1, 0)".into());
        }
        return Ok(Some(GapEff::SetVelocity { who, to: v }));
    }
    // `set name/lore of player's tool to ...`
    let mut j = i;
    if is(toks.get(j), "display") || is(toks.get(j), "custom") {
        j += 1;
    }
    let lore = is(toks.get(j), "lore");
    if !(lore || is(toks.get(j), "name")) || !is(toks.get(j + 1), "of") {
        return Ok(None);
    }
    let Some(who) = held(toks.get(j + 2..to).unwrap_or_default(), env) else {
        return Ok(None);
    };
    let rest = toks.get(to + 1..).unwrap_or_default();
    if rest.is_empty() {
        return Err("expected a value after 'to'".into());
    }
    let op = if lore {
        HeldOp::Lore(parts(rest, env)?)
    } else {
        HeldOp::Name(Some(parse_expr(rest, env)?))
    };
    Ok(Some(GapEff::Held { who, op }))
}

/// `open {_gui} to player`. `open chest with ...` stays with `inventories`.
fn open(toks: &[Tok], env: &Env) -> Option<GapEff> {
    if ["chest", "a", "an", "new"].iter().any(|w| is(toks.get(1), w)) {
        return None;
    }
    let t = rpos_of(toks, &["to", "for"])?;
    let inv_toks = toks.get(1..t)?;
    let inv = match inv_value(inv_toks, env) {
        Some(e) => e,
        None => {
            let e = parse_expr(inv_toks, env).ok()?;
            if e.ty(env) != INV_TY {
                return None;
            }
            e
        }
    };
    let to = sub(toks, t + 1, toks.len(), env)?;
    player_like(&to, env).then_some(GapEff::Open { inv, to })
}

/// `clear {_gui}`, `reset name of player's tool`, `clear lore of player's tool`.
fn clear(toks: &[Tok], env: &Env) -> Option<GapEff> {
    if is(toks.first(), "clear")
        && let Some(inv) = inv_value(toks.get(1..)?, env)
    {
        return Some(GapEff::Clear(inv));
    }
    let mut i = 1;
    if is(toks.get(i), "the") {
        i += 1;
    }
    if is(toks.get(i), "display") || is(toks.get(i), "custom") {
        i += 1;
    }
    let lore = is(toks.get(i), "lore");
    if !(lore || is(toks.get(i), "name")) || !is(toks.get(i + 1), "of") {
        return None;
    }
    let who = held(toks.get(i + 2..)?, env)?;
    let op = if lore {
        HeldOp::Lore(Vec::new())
    } else {
        HeldOp::Name(None)
    };
    Some(GapEff::Held { who, op })
}

/// `add "line" to lore of player's tool`.
fn add_lore(toks: &[Tok], env: &Env) -> Result<Option<GapEff>, String> {
    let Some(to) = pos_of(toks, 2, &["to"]) else {
        return Ok(None);
    };
    let mut i = to + 1;
    if is(toks.get(i), "the") {
        i += 1;
    }
    if !(is(toks.get(i), "lore") && is(toks.get(i + 1), "of")) {
        return Ok(None);
    }
    let Some(who) = held(toks.get(i + 2..).unwrap_or_default(), env) else {
        return Ok(None);
    };
    let lines = parts(toks.get(1..to).unwrap_or_default(), env)?;
    Ok(Some(GapEff::Held {
        who,
        op: HeldOp::AddLore(lines),
    }))
}

/// `enchant player's tool with sharpness 5 [and unbreaking 3]`.
fn enchant(toks: &[Tok], env: &Env) -> Result<Option<GapEff>, String> {
    let Some(w) = pos_of(toks, 2, &["with"]) else {
        return Ok(None);
    };
    let Some(who) = held(toks.get(1..w).unwrap_or_default(), env) else {
        return Ok(None);
    };
    let specs = parts(toks.get(w + 1..).unwrap_or_default(), env)?;
    if specs.is_empty() {
        return Err("expected an enchantment like 'sharpness 5'".into());
    }
    Ok(Some(GapEff::Held {
        who,
        op: HeldOp::Enchant(specs),
    }))
}

/// `replace [all|first] regex "p" in {x} with "y"`, `replace all regex "p" with "y" in {x}`,
/// `regex replace "p" in {x} with "y"`.
fn regex_replace(toks: &[Tok], env: &Env) -> Result<Option<GapEff>, String> {
    let mut i = 1;
    let mut all = true;
    if is(toks.first(), "regex") {
        if !is(toks.get(1), "replace") {
            return Ok(None);
        }
        i = 2;
    } else {
        if is(toks.get(i), "all") || is(toks.get(i), "every") {
            i += 1;
        } else if is(toks.get(i), "first") {
            all = false;
            i += 1;
        }
        if !is(toks.get(i), "regex") {
            return Ok(None);
        }
        i += 1;
    }
    let (Some(inw), Some(with)) = (pos_of(toks, i + 1, &["in"]), pos_of(toks, i + 1, &["with"])) else {
        return Err("expected 'replace all regex \"pattern\" in {variable} with \"text\"'".into());
    };
    let (pat, var_toks, with_e) = if inw < with {
        (
            sub(toks, i, inw, env),
            toks.get(inw + 1..with),
            sub(toks, with + 1, toks.len(), env),
        )
    } else {
        (
            sub(toks, i, with, env),
            toks.get(inw + 1..),
            sub(toks, with + 1, inw, env),
        )
    };
    let (Some(pat), Some(with)) = (pat, with_e) else {
        return Err("expected a pattern and a replacement text".into());
    };
    let Some([Tok::Var(raw)]) = var_toks else {
        return Err("regex replace changes a variable, like {_text}".into());
    };
    let var = parse_varref(raw, env)?;
    if var.list {
        return Err("regex replace works on one variable, not a list".into());
    }
    Ok(Some(GapEff::Regex { pat, var, with, all }))
}

/// `push <who> forward[s]|backward[s] [with speed N]`, `push <who> away from|towards <place> [...]`.
fn push(toks: &[Tok], env: &Env) -> Result<Option<GapEff>, String> {
    let Some(d) = pos_of(
        toks,
        2,
        &[
            "forward",
            "forwards",
            "backward",
            "backwards",
            "away",
            "towards",
            "toward",
        ],
    ) else {
        return Ok(None);
    };
    let Some(who) = sub(toks, 1, d, env) else {
        return Ok(None);
    };
    let dw = |w: &str| is(toks.get(d), w);
    if dw("away") || dw("towards") || dw("toward") {
        let from_at = if dw("away") {
            if !is(toks.get(d + 1), "from") {
                return Ok(None);
            }
            d + 2
        } else {
            d + 1
        };
        let (end, speed) = speed_tail(toks, from_at, env)?;
        let from = sub(toks, from_at, end, env).ok_or("expected a location, player or entity")?;
        return Ok(Some(GapEff::PushFrom {
            who,
            from,
            toward: !dw("away"),
            speed,
        }));
    }
    let (end, speed) = speed_tail(toks, d + 1, env)?;
    if end != d + 1 {
        return Err("expected 'with speed <number>' after the direction".into());
    }
    Ok(Some(GapEff::PushFacing {
        who,
        back: dw("backward") || dw("backwards"),
        speed,
    }))
}

// ---------- conditions ----------

/// `{_gui} contains diamond`, `event-inventory doesn't contain 2 diamonds`.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    let c = toks
        .iter()
        .position(|t| t.is_lit("contains") || t.is_lit("contain"))?;
    let (head, negate) = match toks.get(..c)? {
        [h @ .., a, b] if a.is_lit("does") && b.is_lit("not") => (h, true),
        [h @ .., a] if a.is_lit("doesn't") => (h, true),
        h if toks.get(c).is_some_and(|t| t.is_lit("contains")) => (h, false),
        _ => return None,
    };
    let inv = inv_value(head, env)?;
    let item = parse_expr(toks.get(c + 1..)?, env).ok()?;
    if !matches!(item.ty(env), Ty::Item | Ty::Object) {
        return None;
    }
    Some(Cond::Gap(Box::new(GapCond { inv, item, negate })))
}

// ---------- expressions ----------

/// `{t}'s minutes`: the unit, for the `'s` loop of the expression parser.
pub fn span_unit(word: &str) -> Option<i64> {
    match word {
        "ticks" | "seconds" | "minutes" | "hours" | "days" | "weeks" | "milliseconds" => {
            crate::timespans::unit_ms(word)
        }
        _ => None,
    }
}

impl P<'_> {
    fn tok(&self, n: usize) -> Option<&Tok> {
        let t = self.toks.get(self.pos + n);
        if t.is_none() {
            self.hit_end.set(true);
        }
        t
    }

    /// Expression heads of this module. `self.pos` is after the first word `lw`.
    pub(crate) fn gaps_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        match lw {
            "a" | "an" | "new" | "chest" => self.new_chest(lw),
            "event-inventory" if self.env.has_inv && !self.env.in_function => {
                Ok(Some(gap(GapExpr::EventInv(false))))
            }
            "clicked" | "event"
                if self.env.has_inv && !self.env.in_function && is(self.toks.get(self.pos), "inventory") =>
            {
                self.pos += 1;
                Ok(Some(gap(GapExpr::EventInv(lw == "clicked"))))
            }
            "slot" => Ok(self.gap_slot()),
            "items" | "all" => self.items_in(lw),
            _ => Ok(None),
        }
    }

    /// `{_gui}`, `event-inventory` or `clicked inventory` at the current position.
    fn inv_operand(&mut self) -> Option<Expr> {
        let n = if is(self.toks.get(self.pos), "the") { 1 } else { 0 };
        let e = match self.toks.get(self.pos + n)? {
            Tok::Var(raw) => {
                if is(self.toks.get(self.pos + n + 1), "'s") {
                    return None;
                }
                let v = parse_varref(raw, self.env).ok()?;
                if v.list {
                    return None;
                }
                self.pos += n + 1;
                return Some(Expr::Var(v));
            }
            t if t.is_lit("event-inventory") => (1, false),
            t if t.is_lit("clicked") || t.is_lit("event") => {
                if !is(self.tok(n + 1), "inventory") {
                    return None;
                }
                (2, t.is_lit("clicked"))
            }
            _ => return None,
        };
        if !self.env.has_inv || self.env.in_function {
            return None;
        }
        self.pos += n + e.0;
        Some(gap(GapExpr::EventInv(e.1)))
    }

    /// `[a] [new] chest inventory [with N rows] [named "x"]`, in either order.
    fn new_chest(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let mut i = 0;
        if lw != "chest" {
            if is(self.toks.get(self.pos), "new") {
                i += 1;
            }
            if !is(self.tok(i), "chest") {
                return Ok(None);
            }
            i += 1;
        }
        let kind = self.tok(i);
        if !(is(kind, "inventory") || is(kind, "gui") || is(kind, "menu")) {
            return Ok(None);
        }
        self.pos += i + 1;
        let (mut rows, mut title) = (None, None);
        loop {
            let next = self.toks.get(self.pos);
            if is(next, "with") && (is(self.tok(1), "name") || is(self.tok(1), "title")) && title.is_none() {
                self.pos += 2;
                title = Some(self.primary()?);
            } else if is(next, "with") && rows.is_none() {
                self.pos += 1;
                let r = self.primary()?;
                if !Ty::Number.accepts(r.ty(self.env)) {
                    return Err("the number of rows must be a number".into());
                }
                if !(is(self.tok(0), "row") || is(self.tok(0), "rows")) {
                    return Err("expected 'with <number> rows'".into());
                }
                self.pos += 1;
                rows = Some(r);
            } else if (is(next, "named") || is(next, "titled") || is(next, "called")) && title.is_none() {
                self.pos += 1;
                title = Some(self.primary()?);
            } else {
                break;
            }
        }
        Ok(Some(gap(GapExpr::NewChest { rows, title })))
    }

    /// `slot N of {_gui}`. `None` (position kept) for other `slot` lines.
    fn gap_slot(&mut self) -> Option<Expr> {
        let save = self.pos;
        let found = (|| {
            let slot = self.primary().ok()?;
            if !Ty::Number.accepts(slot.ty(self.env)) || !is(self.toks.get(self.pos), "of") {
                return None;
            }
            self.pos += 1;
            let inv = self.inv_operand()?;
            Some(gap(GapExpr::Slot { slot, inv }))
        })();
        if found.is_none() {
            self.pos = save;
        }
        found
    }

    /// `[all] [the] items in {_gui}` / `... in player's inventory`.
    fn items_in(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let save = self.pos;
        let mut i = 0;
        if lw == "all" {
            if is(self.toks.get(self.pos), "the") {
                i += 1;
            }
            if !is(self.tok(i), "items") {
                return Ok(None);
            }
            i += 1;
        }
        let w = self.tok(i);
        if !(is(w, "in") || is(w, "of") || is(w, "inside") || is(w, "within")) {
            return Ok(None);
        }
        self.pos += i + 1;
        if let Some(inv) = self.inv_operand() {
            return Ok(Some(gap(GapExpr::Items(inv))));
        }
        let rest = self.toks.get(self.pos..).unwrap_or_default();
        if let Some(len) = inv_target_len(rest)
            && let Some(t) = inv_target(rest.get(..len).unwrap_or_default(), self.env)
        {
            self.pos += len;
            return Ok(Some(gap(GapExpr::PlayerItems(t))));
        }
        let mut n = 0;
        if is(rest.get(n), "the") {
            n += 1;
        }
        let current = is(rest.get(n), "current");
        if current {
            n += 1;
        }
        if is(rest.get(n), "inventory") && is(rest.get(n + 1), "of") {
            self.pos += n + 2;
            let who = self.primary()?;
            if !player_like(&who, self.env) {
                return Err("the inventory belongs to a player".into());
            }
            return Ok(Some(gap(GapExpr::PlayerItems(InvTarget { who, current }))));
        }
        if !rest.iter().any(|t| t.is_lit("inventory")) {
            self.hit_end.set(true);
        }
        self.pos = save;
        Ok(None)
    }

    /// After `with [all|first]`: `regex "p" replaced with "x"`. `self.pos` is at `regex`.
    pub(crate) fn regex_replaced(&mut self, text: Expr, all: bool) -> Option<Expr> {
        if !is(self.toks.get(self.pos), "regex") {
            return None;
        }
        self.pos += 1;
        let pat = self.primary_base().ok()?;
        if !is(self.tok(0), "replaced") {
            return None;
        }
        self.pos += 1;
        if !(is(self.tok(0), "with") || is(self.tok(0), "by")) {
            return None;
        }
        self.pos += 1;
        let with = self.primary_base().ok()?;
        Some(gap(GapExpr::Regex { text, pat, with, all }))
    }
}

// ---------- regex replace ----------

/// Replace the first or every match of `pattern` in `text`. The replacement is
/// plain text (no `$1` groups: the regex engine has no captures).
pub fn regex_replace_text(text: &str, pattern: &str, with: &str, all: bool) -> Result<String, String> {
    use crate::conditions_entity_block::regex::Regex;
    let re = Regex::new(pattern).map_err(|e| format!("invalid regex '{pattern}': {e}"))?;
    let t: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at <= t.len() {
        let Some((s, e)) = re.find_span(&t, at)? else {
            break;
        };
        out.extend(t.get(at..s).unwrap_or_default());
        out.push_str(with);
        if out.len() > crate::engine::MAX_TEXT_BYTES {
            return Err(format!(
                "this replace would make a text longer than {} characters",
                crate::engine::MAX_TEXT_BYTES
            ));
        }
        if e == s {
            // An empty match: keep one character so the loop moves on.
            if let Some(c) = t.get(s) {
                out.push(*c);
            }
            at = s + 1;
        } else {
            at = e;
        }
        if !all {
            break;
        }
    }
    out.extend(t.get(at.min(t.len())..).unwrap_or_default());
    Ok(out)
}
