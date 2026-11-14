//! Inventories and GUIs: open a chest menu, read and write slots, clear,
//! add and remove items, and the `on inventory click` event values.
//!
//! The lines have free-form words in them, so they are parsed by hand here
//! instead of by the pattern registry. `parse_effect` and `parse_cond` are
//! called from the compiler before the registry is tried. The `Engine`
//! methods at the bottom run them. Everything rare lives behind boxes, so the
//! hot `Effect` and `Expr` enums do not grow.

use crate::compile::{Cond, Effect};
use crate::engine::{Engine, Exec};
use crate::expr::{Env, Expr, Ty, parse_expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::Value;

/// What the clicked inventory slot held, for `on inventory click`.
#[derive(Debug, Clone, PartialEq)]
pub struct InvClick {
    pub slot: i32,
    /// Item id (no `minecraft:`) and amount, `None` for an empty slot.
    pub item: Option<(String, u32)>,
    /// `left`, `right`, `shift left`, ...
    pub click: String,
}

/// Which inventory a line talks about.
#[derive(Debug, Clone, PartialEq)]
pub struct InvTarget {
    pub who: Expr,
    /// `current inventory`: the open menu if there is one.
    pub current: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InvEff {
    Open {
        rows: Expr,
        title: Option<Expr>,
        to: Expr,
    },
    SetSlot {
        slot: Expr,
        inv: InvTarget,
        item: Expr,
    },
    Close(Expr),
    Clear(InvTarget),
    Remove {
        item: Expr,
        who: Expr,
    },
    Add {
        item: Expr,
        who: Expr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InvExpr {
    Slot { slot: Expr, inv: InvTarget },
    ClickedSlot,
    ClickedItem,
    ClickType,
}

impl InvExpr {
    pub fn ty(&self) -> Ty {
        match self {
            InvExpr::Slot { .. } | InvExpr::ClickedItem => Ty::Item,
            InvExpr::ClickedSlot => Ty::Number,
            InvExpr::ClickType => Ty::Text,
        }
    }
}

const MAX_ROWS: f64 = 6.0;
const MAX_SLOT: f64 = 1_000.0;

// ---------- parsing ----------

fn is(t: &Tok, lit: &str) -> bool {
    t.is_lit(lit)
}

fn find(toks: &[Tok], from: usize, lits: &[&str]) -> Option<usize> {
    toks.iter()
        .enumerate()
        .skip(from)
        .find(|(_, t)| lits.iter().any(|l| is(t, l)))
        .map(|(i, _)| i)
}

fn expr_of(toks: &[Tok], env: &Env, want: &[Ty]) -> Option<Expr> {
    if toks.is_empty() {
        return None;
    }
    let e = parse_expr(toks, env).ok()?;
    let got = e.ty(env);
    (got == Ty::Object || want.contains(&got)).then_some(e)
}

fn strict(toks: &[Tok], env: &Env, want: Ty) -> Option<Expr> {
    if toks.is_empty() {
        return None;
    }
    let e = parse_expr(toks, env).ok()?;
    (e.ty(env) == want).then_some(e)
}

/// `<player>'s [current] inventory` or `[current] inventory of <player>`.
pub fn inv_target(toks: &[Tok], env: &Env) -> Option<InvTarget> {
    // `<player>'s [current] inventory`
    if let [head @ .., s, inv] = toks
        && is(s, "'s")
        && is(inv, "inventory")
    {
        return Some(InvTarget {
            who: expr_of(head, env, &[Ty::Player])?,
            current: false,
        });
    }
    if let [head @ .., s, c, inv] = toks
        && is(s, "'s")
        && is(c, "current")
        && is(inv, "inventory")
    {
        return Some(InvTarget {
            who: expr_of(head, env, &[Ty::Player])?,
            current: true,
        });
    }
    // `[the] [current] inventory of <player>`
    let mut i = 0;
    if toks.get(i).is_some_and(|t| is(t, "the")) {
        i += 1;
    }
    let mut current = false;
    if toks.get(i).is_some_and(|t| is(t, "current")) {
        current = true;
        i += 1;
    }
    if toks.get(i).is_some_and(|t| is(t, "inventory")) && toks.get(i + 1).is_some_and(|t| is(t, "of")) {
        return Some(InvTarget {
            who: expr_of(toks.get(i + 2..)?, env, &[Ty::Player])?,
            current,
        });
    }
    None
}

/// Where the `<player>'s [current] inventory` part of a `slot N of ...`
/// expression ends, in `rest` (the tokens after `of`). `None` when the
/// words are not an inventory.
pub fn inv_target_len(rest: &[Tok]) -> Option<usize> {
    let i = rest.iter().position(|t| is(t, "'s"))?;
    match (rest.get(i + 1), rest.get(i + 2)) {
        (Some(a), _) if is(a, "inventory") => Some(i + 2),
        (Some(a), Some(b)) if is(a, "current") && is(b, "inventory") => Some(i + 3),
        _ => None,
    }
}

/// `Ok(None)`: not an inventory effect line. `Err`: it is one, but wrong.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    let eff = match first.to_ascii_lowercase().as_str() {
        "open" => open(toks, env)?,
        "set" => set_slot(toks, env),
        "close" => inv_target(toks.get(1..).unwrap_or_default(), env).map(|t| InvEff::Close(t.who)),
        "clear" => inv_target(toks.get(1..).unwrap_or_default(), env).map(InvEff::Clear),
        "remove" => item_move(toks, env, "from"),
        "add" => item_move(toks, env, "to"),
        _ => None,
    };
    Ok(eff.map(|e| Effect::Inv(Box::new(e))))
}

/// `open [a] chest [inventory|gui|menu] with 3 rows [named "x"] to <player>`.
fn open(toks: &[Tok], env: &Env) -> Result<Option<InvEff>, String> {
    let Some(c) = find(toks, 1, &["chest"]) else {
        return Ok(None);
    };
    let filler = |t: &Tok| ["a", "an", "the", "new"].iter().any(|w| is(t, w));
    if !toks.get(1..c).unwrap_or_default().iter().all(filler) {
        return Ok(None);
    }
    let Some(w) = find(toks, c + 1, &["with"]) else {
        return Ok(None);
    };
    if !toks
        .get(c + 1..w)
        .unwrap_or_default()
        .iter()
        .all(|t| ["inventory", "gui", "menu"].iter().any(|x| is(t, x)))
    {
        return Ok(None);
    }
    let Some(r) = find(toks, w + 1, &["row", "rows"]) else {
        return Ok(None);
    };
    let Some(rows) = toks.get(w + 1..r).and_then(|t| expr_of(t, env, &[Ty::Number])) else {
        return Ok(None);
    };
    if let Expr::Num(n) = rows
        && !(1.0..=MAX_ROWS).contains(&n)
    {
        return Err("a chest has 1 to 6 rows".into());
    }
    let rest = toks.get(r + 1..).unwrap_or_default();
    let Some(t) = rest.iter().rposition(|t| is(t, "to") || is(t, "for")) else {
        return Ok(None);
    };
    let Some(to) = rest.get(t + 1..).and_then(|x| expr_of(x, env, &[Ty::Player])) else {
        return Ok(None);
    };
    let mid = rest.get(..t).unwrap_or_default();
    let title_toks = match mid {
        [] => None,
        [a, b, rest @ ..] if is(a, "with") && (is(b, "title") || is(b, "name")) => Some(rest),
        [a, rest @ ..] if is(a, "named") || is(a, "titled") || is(a, "called") => Some(rest),
        _ => return Ok(None),
    };
    let title = match title_toks {
        Some(t) => Some(expr_of(t, env, &[Ty::Text]).ok_or("the chest title must be text")?),
        None => None,
    };
    Ok(Some(InvEff::Open { rows, title, to }))
}

/// `set slot N of <player>'s [current] inventory to <item>`.
fn set_slot(toks: &[Tok], env: &Env) -> Option<InvEff> {
    if !toks.get(1).is_some_and(|t| is(t, "slot")) {
        return None;
    }
    let o = find(toks, 2, &["of"])?;
    let t = toks.iter().rposition(|t| is(t, "to"))?;
    if t < o {
        return None;
    }
    Some(InvEff::SetSlot {
        slot: expr_of(toks.get(2..o)?, env, &[Ty::Number])?,
        inv: inv_target(toks.get(o + 1..t)?, env)?,
        item: expr_of(toks.get(t + 1..)?, env, &[Ty::Item])?,
    })
}

/// `remove 2 diamonds from <player>[`'s inventory`]`, `add <item> to <player>'s inventory`.
///
/// A bare variable as the target is left to the list effects, so this only
/// takes a player-typed target or one with the word `inventory`.
fn item_move(toks: &[Tok], env: &Env, sep: &str) -> Option<InvEff> {
    let remove = sep == "from";
    let s = toks.iter().rposition(|t| is(t, sep))?;
    let item_toks = toks.get(1..s)?;
    let target = toks.get(s + 1..)?;
    let (who, has_inv) = match inv_target(target, env) {
        Some(t) if !t.current => (t.who, true),
        Some(_) => return None,
        // Bare `player` only: `add 1 to {x}` and lists belong to other effects.
        None if !remove => return None,
        None => (strict(target, env, Ty::Player)?, false),
    };
    let item = if has_inv {
        expr_of(item_toks, env, &[Ty::Item])?
    } else {
        strict(item_toks, env, Ty::Item)?
    };
    Some(if remove {
        InvEff::Remove { item, who }
    } else {
        InvEff::Add { item, who }
    })
}

/// `<player>'s inventory (does not|doesn't) contain <item>`.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    let c = toks.iter().position(|t| is(t, "contain") || is(t, "contains"))?;
    let (head, negate) = match toks.get(..c)? {
        [h @ .., a, b] if is(a, "does") && is(b, "not") => (h, true),
        [h @ .., a] if is(a, "doesn't") || is(a, "don't") => (h, true),
        _ => return None,
    };
    let inv = inv_target(head, env)?;
    if inv.current {
        return None;
    }
    Some(Cond::HasItem {
        player: inv.who,
        item: expr_of(toks.get(c + 1..)?, env, &[Ty::Item])?,
        negate,
    })
}

// ---------- running ----------

fn slot_index(n: f64) -> Result<i64, String> {
    if !n.is_finite() || !(0.0..=MAX_SLOT).contains(&n.floor()) {
        return Err(format!("slot {n} is out of range, use 0 or more"));
    }
    Ok(n.floor() as i64)
}

fn item_of(v: Value) -> Result<Option<(String, u32)>, String> {
    match v {
        Value::None => Ok(None),
        Value::Item { id, .. } if id == "air" => Ok(None),
        Value::Item { id, count, .. } => Ok(Some((id, count.max(1)))),
        other => Err(format!("expected an item, got {} ({other})", other.type_name())),
    }
}

impl Engine {
    #[inline(never)]
    pub(crate) fn inv_effect(
        &mut self,
        e: &InvEff,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        match e {
            InvEff::Open { rows, title, to } => {
                let rows = self.num(rows, ex, host)?;
                if !rows.is_finite() || !(1.0..=MAX_ROWS).contains(&rows.floor()) {
                    return Err("a chest has 1 to 6 rows".into());
                }
                let title = match title {
                    Some(t) => self.text(t, ex, host)?,
                    None => "Chest".to_string(),
                };
                if let Some(p) = self.player(to, ex, host)? {
                    host.open_chest(&p, rows.floor() as u8, &title);
                }
            }
            InvEff::SetSlot { slot, inv, item } => {
                let slot = slot_index(self.num(slot, ex, host)?)?;
                let item = item_of(self.eval(item, ex, host)?)?;
                if let Some(p) = self.player(&inv.who, ex, host)? {
                    host.inv_set(
                        &p,
                        inv.current,
                        slot,
                        item.as_ref().map(|(i, c)| (i.as_str(), *c)),
                    );
                }
            }
            InvEff::Close(who) => {
                if let Some(p) = self.player(who, ex, host)? {
                    host.close_inventory(&p);
                }
            }
            InvEff::Clear(inv) => {
                if let Some(p) = self.player(&inv.who, ex, host)? {
                    host.inv_clear(&p, inv.current);
                }
            }
            InvEff::Remove { item, who } => {
                let item = item_of(self.eval(item, ex, host)?)?;
                if let (Some((id, count)), Some(p)) = (item, self.player(who, ex, host)?) {
                    host.inv_remove(&p, &id, count);
                }
            }
            InvEff::Add { item, who } => {
                let item = item_of(self.eval(item, ex, host)?)?;
                if let (Some((id, count)), Some(p)) = (item, self.player(who, ex, host)?) {
                    host.give_item(&p, &id, count);
                }
            }
        }
        Ok(())
    }

    #[inline(never)]
    pub(crate) fn inv_expr(
        &mut self,
        e: &InvExpr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        Ok(match e {
            InvExpr::Slot { slot, inv } => {
                let slot = slot_index(self.num(slot, ex, host)?)?;
                match self.player(&inv.who, ex, host)? {
                    Some(p) => host
                        .inv_get(&p, inv.current, slot)
                        .map_or(Value::None, |(id, count)| Value::Item {
                            id,
                            count,
                            meta: None,
                        }),
                    None => Value::None,
                }
            }
            InvExpr::ClickedSlot => ex
                .ctx
                .inv
                .as_ref()
                .map_or(Value::None, |c| Value::Num(f64::from(c.slot))),
            InvExpr::ClickedItem => {
                ex.ctx
                    .inv
                    .as_ref()
                    .and_then(|c| c.item.clone())
                    .map_or(Value::None, |(id, count)| Value::Item {
                        id,
                        count,
                        meta: None,
                    })
            }
            InvExpr::ClickType => ex
                .ctx
                .inv
                .as_ref()
                .map_or(Value::None, |c| Value::Text(c.click.clone())),
        })
    }
}
