//! SkBee-style bounds: named boxes in a world with owners, members and flags.
//!
//! A bound is stored as its id (text) when a script holds it in a variable.
//! Lookups use a grid index per world, so a point query touches one cell.
//! `on bound enter` / `on bound exit` are found by checking each player's
//! block position (polled from `Engine::tick`, or pushed with
//! `Engine::bounds_moved`). A player is only re-checked when the block
//! under them changes.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::compile::{Cond, Effect, EventKind};
use crate::engine::{Engine, EventCtx, Exec, online};
use crate::expr::{Env, Expr, Ty, parse_expr};
use crate::host::{Host, LogLevel};
use crate::lexer::Tok;
use crate::value::{PlayerRef, Value};

/// World name used when a location has none.
pub const DEFAULT_WORLD: &str = "world";
/// Players are checked every this many ticks.
pub const POLL_TICKS: u64 = 2;
/// Index cell edge is `1 << CELL_SHIFT` blocks.
const CELL_SHIFT: i32 = 6;
/// A bound that covers more cells than this is kept in a short "big" list.
const MAX_CELLS: i64 = 1024;
const SAVE_DEBOUNCE_MS: u64 = 5_000;

// ---------- data ----------

#[derive(Debug, Clone, PartialEq)]
pub struct Bound {
    pub id: String,
    pub world: String,
    /// Lesser corner, block coordinates, inclusive.
    pub min: [i32; 3],
    /// Greater corner, block coordinates, inclusive.
    pub max: [i32; 3],
    pub owners: Vec<PlayerRef>,
    pub members: Vec<PlayerRef>,
    pub flags: BTreeMap<String, bool>,
    /// Temporary bounds are not saved.
    pub temporary: bool,
}

fn block(v: f64) -> i32 {
    v.floor() as i32
}

impl Bound {
    pub fn contains(&self, x: f64, y: f64, z: f64) -> bool {
        let p = [block(x), block(y), block(z)];
        (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i])
    }

    /// Number of blocks inside.
    pub fn volume(&self) -> f64 {
        (0..3)
            .map(|i| f64::from(self.max[i]) - f64::from(self.min[i]) + 1.0)
            .product()
    }
}

#[derive(Debug, Default)]
struct WorldGrid {
    cells: HashMap<(i32, i32), Vec<usize>>,
    big: Vec<usize>,
}

#[derive(Debug, Default)]
struct Track {
    blk: (i32, i32, i32),
    world: String,
    inside: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Bounds {
    list: Vec<Bound>,
    by_id: HashMap<String, usize>,
    grid: HashMap<String, WorldGrid>,
    tracker: HashMap<String, Track>,
    dirty_since: Option<u64>,
    /// What the store holds now: lower-case id to the JSON of that bound.
    persisted: BTreeMap<String, String>,
}

impl Bounds {
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn all(&self) -> &[Bound] {
        &self.list
    }

    pub fn get(&self, id: &str) -> Option<&Bound> {
        self.by_id
            .get(&id.to_ascii_lowercase())
            .and_then(|i| self.list.get(*i))
    }

    fn get_mut(&mut self, id: &str) -> Option<&mut Bound> {
        let i = *self.by_id.get(&id.to_ascii_lowercase())?;
        self.list.get_mut(i)
    }

    /// Add a bound, replacing one with the same id.
    pub fn create(&mut self, b: Bound) {
        match self.by_id.get(&b.id.to_ascii_lowercase()) {
            Some(i) => self.list[*i] = b,
            None => self.list.push(b),
        }
        self.reindex();
    }

    pub fn delete(&mut self, id: &str) -> bool {
        let Some(i) = self.by_id.get(&id.to_ascii_lowercase()).copied() else {
            return false;
        };
        self.list.remove(i);
        self.reindex();
        true
    }

    fn reindex(&mut self) {
        self.by_id.clear();
        self.grid.clear();
        for (i, b) in self.list.iter().enumerate() {
            self.by_id.insert(b.id.to_ascii_lowercase(), i);
            let g = self.grid.entry(b.world.clone()).or_default();
            let (x0, x1) = (b.min[0] >> CELL_SHIFT, b.max[0] >> CELL_SHIFT);
            let (z0, z1) = (b.min[2] >> CELL_SHIFT, b.max[2] >> CELL_SHIFT);
            let cells = (i64::from(x1) - i64::from(x0) + 1) * (i64::from(z1) - i64::from(z0) + 1);
            if cells > MAX_CELLS {
                g.big.push(i);
                continue;
            }
            for cx in x0..=x1 {
                for cz in z0..=z1 {
                    g.cells.entry((cx, cz)).or_default().push(i);
                }
            }
        }
    }

    /// Indexes of the bounds that hold this point, in creation order.
    fn at(&self, world: &str, x: f64, y: f64, z: f64) -> Vec<usize> {
        let Some(g) = self.grid.get(world) else {
            return Vec::new();
        };
        let cell = (block(x) >> CELL_SHIFT, block(z) >> CELL_SHIFT);
        let mut out: Vec<usize> = g
            .cells
            .get(&cell)
            .into_iter()
            .flatten()
            .chain(g.big.iter())
            .copied()
            .filter(|i| self.list.get(*i).is_some_and(|b| b.contains(x, y, z)))
            .collect();
        out.sort_unstable();
        out
    }

    /// Ids of the bounds that hold this point.
    pub fn ids_at(&self, world: &str, x: f64, y: f64, z: f64) -> Vec<String> {
        self.at(world, x, y, z)
            .into_iter()
            .filter_map(|i| self.list.get(i).map(|b| b.id.clone()))
            .collect()
    }

    fn touch(&mut self, now: u64) {
        if self.dirty_since.is_none() {
            self.dirty_since = Some(now);
        }
    }

    // ---- saving ----

    fn bound_json(b: &Bound) -> serde_json::Value {
        use serde_json::{Value as J, json};
        let people = |v: &[PlayerRef]| -> J { v.iter().map(|p| json!([p.uuid, p.name])).collect() };
        json!({
            "id": b.id,
            "world": b.world,
            "min": b.min,
            "max": b.max,
            "owners": people(&b.owners),
            "members": people(&b.members),
            "flags": b.flags,
        })
    }

    /// The whole list as the old `bounds.json` file.
    fn to_json(&self) -> String {
        let items: Vec<serde_json::Value> = self
            .list
            .iter()
            .filter(|b| !b.temporary)
            .map(Self::bound_json)
            .collect();
        serde_json::to_string_pretty(&serde_json::json!({ "bounds": items })).unwrap_or_default()
    }

    /// The bounds that are saved now, as `(lower-case id, JSON)`. The `n` field keeps the creation order.
    fn rows(&self) -> BTreeMap<String, String> {
        self.list
            .iter()
            .filter(|b| !b.temporary)
            .enumerate()
            .map(|(n, b)| {
                let mut j = Self::bound_json(b);
                j["n"] = n.into();
                (b.id.to_ascii_lowercase(), j.to_string())
            })
            .collect()
    }

    /// Bounds that were added, changed or deleted since the store was last
    /// told, as `(lower-case id, JSON bytes or None for deleted)`.
    pub fn take_changes(&mut self) -> Vec<(String, Option<Vec<u8>>)> {
        self.dirty_since = None;
        let now = self.rows();
        let mut out: Vec<(String, Option<Vec<u8>>)> = Vec::new();
        for (id, json) in &now {
            if self.persisted.get(id) != Some(json) {
                out.push((id.clone(), Some(json.clone().into_bytes())));
            }
        }
        for id in self.persisted.keys() {
            if !now.contains_key(id) {
                out.push((id.clone(), None));
            }
        }
        self.persisted = now;
        out
    }

    /// Like [`Bounds::take_changes`], once bounds changed at least 5 seconds before `now_ms`.
    pub fn changes_due(&mut self, now_ms: u64) -> Option<Vec<(String, Option<Vec<u8>>)>> {
        let since = self.dirty_since?;
        (now_ms.saturating_sub(since) >= SAVE_DEBOUNCE_MS).then(|| self.take_changes())
    }

    /// Every saved bound, for a first import into the store.
    pub fn entries(&self) -> Vec<(String, Vec<u8>)> {
        self.rows()
            .into_iter()
            .map(|(k, v)| (k, v.into_bytes()))
            .collect()
    }

    /// Rebuild from the store's rows (any order). Rows that don't parse are
    /// listed in the second part.
    pub fn from_entries(rows: impl IntoIterator<Item = (String, Vec<u8>)>) -> (Bounds, Vec<String>) {
        let mut items: Vec<(i64, Bound)> = Vec::new();
        let mut bad = Vec::new();
        for (id, bytes) in rows {
            let parsed = serde_json::from_slice::<serde_json::Value>(&bytes)
                .map_err(|e| e.to_string())
                .and_then(|v| {
                    let n = v.get("n").and_then(serde_json::Value::as_i64).unwrap_or(i64::MAX);
                    Self::parse_bound(&v).map(|b| (n, b))
                });
            match parsed {
                Ok(it) => items.push(it),
                Err(_) => bad.push(id),
            }
        }
        items.sort_by_key(|(n, _)| *n);
        let mut out = Bounds {
            list: items.into_iter().map(|(_, b)| b).collect(),
            ..Bounds::default()
        };
        out.reindex();
        out.persisted = out.rows();
        (out, bad)
    }

    /// JSON to write if bounds changed at least 5 seconds before `now_ms`.
    pub fn save_due(&mut self, now_ms: u64) -> Option<String> {
        let since = self.dirty_since?;
        if now_ms.saturating_sub(since) >= SAVE_DEBOUNCE_MS {
            self.dirty_since = None;
            Some(self.to_json())
        } else {
            None
        }
    }

    /// JSON to write now (unload), if anything changed.
    pub fn save_now(&mut self) -> Option<String> {
        self.dirty_since.take().map(|_| self.to_json())
    }

    /// One bound from its JSON object (the old file's entries and the store's rows look alike).
    fn parse_bound(it: &serde_json::Value) -> Result<Bound, String> {
        let id = it
            .get("id")
            .and_then(|x| x.as_str())
            .ok_or("bounds file: bound without id")?;
        let corner = |k: &str| -> Result<[i32; 3], String> {
            let a = it
                .get(k)
                .and_then(|x| x.as_array())
                .ok_or(format!("bound '{id}': missing {k}"))?;
            let n = |i: usize| a.get(i).and_then(serde_json::Value::as_i64).map(|n| n as i32);
            match (n(0), n(1), n(2)) {
                (Some(x), Some(y), Some(z)) => Ok([x, y, z]),
                _ => Err(format!("bound '{id}': bad {k}")),
            }
        };
        let people = |k: &str| -> Vec<PlayerRef> {
            it.get(k)
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|p| {
                            let p = p.as_array()?;
                            Some(PlayerRef {
                                uuid: p.first()?.as_str()?.to_string(),
                                name: p.get(1)?.as_str()?.to_string(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        let flags = it
            .get("flags")
            .and_then(|x| x.as_object())
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| Some((k.clone(), v.as_bool()?)))
                    .collect()
            })
            .unwrap_or_default();
        Ok(Bound {
            id: id.to_string(),
            world: it
                .get("world")
                .and_then(|x| x.as_str())
                .unwrap_or(DEFAULT_WORLD)
                .to_string(),
            min: corner("min")?,
            max: corner("max")?,
            owners: people("owners"),
            members: people("members"),
            flags,
            temporary: false,
        })
    }

    /// Read the old `bounds.json` file.
    pub fn from_json(s: &str) -> Result<Bounds, String> {
        let v: serde_json::Value = serde_json::from_str(s).map_err(|e| format!("bounds file: {e}"))?;
        let items = v
            .get("bounds")
            .and_then(|b| b.as_array())
            .ok_or("bounds file: missing 'bounds' list")?;
        let mut out = Bounds::default();
        for it in items {
            out.list.push(Self::parse_bound(it)?);
        }
        out.reindex();
        Ok(out)
    }
}

// ---------- syntax tree ----------

#[derive(Debug, Clone, PartialEq)]
pub enum BoundExpr {
    /// `bound "id"`: the id, or `<none>` when there is no such bound.
    Named(Expr),
    /// `bound at <location>`
    At(Expr),
    /// `event-bound`
    Event,
    /// `bounds at <location>` (list)
    BoundsAt(Expr),
    /// `all bounds` (list)
    All,
    /// `owners of bound X` (true) / `members of bound X` (false); a list.
    People(bool, Expr),
    /// `lesser corner of bound X` (false) / `greater corner of bound X` (true)
    Corner(bool, Expr),
    /// `world of bound X`
    WorldOf(Expr),
    /// `flag "pvp" of bound X`
    Flag(Expr, Expr),
    /// `volume of bound X`
    Volume(Expr),
}

impl BoundExpr {
    pub fn ty(&self) -> Ty {
        match self {
            BoundExpr::Named(_) | BoundExpr::At(_) | BoundExpr::Event => Ty::Text,
            BoundExpr::BoundsAt(_) | BoundExpr::All | BoundExpr::People(..) => Ty::List,
            BoundExpr::Corner(..) => Ty::Location,
            BoundExpr::WorldOf(_) => Ty::World,
            BoundExpr::Flag(..) => Ty::Boolean,
            BoundExpr::Volume(_) => Ty::Number,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoundCond {
    /// `<location or player> is [not] within bound X`
    Within {
        subject: Expr,
        bound: Expr,
        negate: bool,
    },
    /// `bound X exists`, `bound at <loc> does not exist`
    Exists { value: Expr, negate: bool },
    /// `<player> is [not] an owner / a member of bound X`
    Role {
        player: Expr,
        bound: Expr,
        owner: bool,
        negate: bool,
    },
    /// `bound X has flag "pvp"`
    HasFlag { bound: Expr, flag: Expr, negate: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoundEff {
    /// `create bound with id X within A and B`
    Create {
        id: Expr,
        a: Expr,
        b: Expr,
        temporary: bool,
    },
    Delete(Expr),
    /// `set owner of bound X to <player>` (true) / `set member ...`
    SetPerson {
        owner: bool,
        bound: Expr,
        player: Expr,
    },
    /// `add <player> to owners of bound X`
    AddPerson {
        owner: bool,
        bound: Expr,
        player: Expr,
    },
    /// `remove <player> from owners of bound X`
    RemovePerson {
        owner: bool,
        bound: Expr,
        player: Expr,
    },
    /// `set flag "pvp" of bound X to true`; `value: None` removes the flag.
    SetFlag {
        bound: Expr,
        flag: Expr,
        value: Option<Expr>,
    },
    /// `set lesser corner of bound X to <location>`
    SetCorner {
        greater: bool,
        bound: Expr,
        loc: Expr,
    },
}

// ---------- parsing ----------

fn is(t: &Tok, lit: &str) -> bool {
    t.is_lit(lit)
}

fn has_bound_word(toks: &[Tok]) -> bool {
    toks.iter().any(|t| is(t, "bound") || is(t, "bounds"))
}

fn find(toks: &[Tok], from: usize, lits: &[&str]) -> Option<usize> {
    let mut depth = 0i32;
    for (i, t) in toks.iter().enumerate().skip(from) {
        match t {
            Tok::Sym("(") => depth += 1,
            Tok::Sym(")") => depth -= 1,
            _ if depth == 0 && lits.iter().any(|l| is(t, l)) => return Some(i),
            _ => {}
        }
    }
    None
}

fn expr_of(toks: &[Tok], env: &Env, want: &[Ty]) -> Option<Expr> {
    if toks.is_empty() {
        return None;
    }
    let e = parse_expr(toks, env).ok()?;
    let got = e.ty(env);
    (got == Ty::Object || want.contains(&got)).then_some(e)
}

fn place(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Location, Ty::Player])
}

fn location(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Location])
}

fn player(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Player])
}

fn text(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Text])
}

/// `bound "id"`, `the bound with id "id"`, `bound at <loc>`, `event-bound`.
fn bound_ref(toks: &[Tok], env: &Env) -> Option<Expr> {
    let toks = match toks.first() {
        Some(t) if is(t, "the") => toks.get(1..)?,
        _ => toks,
    };
    if toks.len() == 1 && is(&toks[0], "event-bound") {
        return Some(Expr::Bound(Box::new(BoundExpr::Event)));
    }
    expr_of(toks, env, &[Ty::Text])
}

/// Skip `with id` / `id` / `named` / `called` after `bound`.
pub fn skip_id_words(toks: &[Tok], pos: usize) -> usize {
    let mut p = pos;
    if toks.get(p).is_some_and(|t| is(t, "with")) {
        p += 1;
    }
    if toks
        .get(p)
        .is_some_and(|t| is(t, "id") || is(t, "named") || is(t, "called") || is(t, "name"))
    {
        p += 1;
    }
    p
}

fn bound_expr_ref(toks: &[Tok], env: &Env) -> Option<Expr> {
    bound_ref(toks, env)
}

pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    if !has_bound_word(toks) {
        return Ok(None);
    }
    let first = first.to_ascii_lowercase();
    let eff = match first.as_str() {
        "create" | "make" if toks.iter().any(|t| is(t, "within") || is(t, "between")) => create(toks, env)?,
        "delete" | "remove" | "clear" | "reset" | "unset" => remove_like(toks, env),
        "add" => add_remove(toks, env, true),
        "set" => set(toks, env),
        _ => None,
    };
    Ok(eff.map(|b| Effect::Bound(Box::new(b))))
}

/// `create [a] [new] [temporary] bound with id X within A and B`
fn create(toks: &[Tok], env: &Env) -> Result<Option<BoundEff>, String> {
    let Some(b) = toks.iter().position(|t| is(t, "bound")) else {
        return Ok(None);
    };
    let skipped: Vec<&Tok> = toks.get(1..b).unwrap_or_default().iter().collect();
    if !skipped
        .iter()
        .all(|t| is(t, "a") || is(t, "an") || is(t, "new") || is(t, "temporary") || is(t, "temp"))
    {
        return Ok(None);
    }
    let temporary = skipped.iter().any(|t| is(t, "temporary") || is(t, "temp"));
    let Some(w) = find(toks, b + 1, &["within", "between"]) else {
        return Ok(None);
    };
    let id_start = skip_id_words(toks, b + 1);
    let Some(id) = toks.get(id_start..w).and_then(|t| text(t, env)) else {
        return Ok(None);
    };
    let rest = toks.get(w + 1..).unwrap_or_default();
    let Some(and) = find(rest, 0, &["and"]) else {
        return Err("expected 'create bound with id <text> within <location> and <location>'".into());
    };
    let (Some(a), Some(c)) = (location(&rest[..and], env), location(&rest[and + 1..], env)) else {
        return Err("a bound needs two locations: 'within <location> and <location>'".into());
    };
    Ok(Some(BoundEff::Create {
        id,
        a,
        b: c,
        temporary,
    }))
}

/// `delete bound X`, `delete flag "pvp" of bound X`, `remove bound X`.
fn remove_like(toks: &[Tok], env: &Env) -> Option<BoundEff> {
    if is(toks.get(1)?, "flag") {
        let of = find(toks, 2, &["of"])?;
        let flag = text(toks.get(2..of)?, env)?;
        let bound = bound_expr_ref(toks.get(of + 1..)?, env)?;
        return Some(BoundEff::SetFlag {
            bound,
            flag,
            value: None,
        });
    }
    // `remove <player> from owners of bound X`
    if find(toks, 1, &["from"]).is_some()
        && let Some(r) = add_remove(toks, env, false)
    {
        return Some(r);
    }
    let rest = toks.get(1..)?;
    let rest = match rest.first() {
        Some(t) if is(t, "the") || is(t, "a") => rest.get(1..)?,
        _ => rest,
    };
    if !is(rest.first()?, "bound") {
        return None;
    }
    Some(BoundEff::Delete(bound_expr_ref(rest, env)?))
}

/// `add P to owners of bound X` (add) / `remove P from members of bound X`.
fn add_remove(toks: &[Tok], env: &Env, add: bool) -> Option<BoundEff> {
    let sep = find(toks, 1, if add { &["to"] } else { &["from"] })?;
    let who = player(toks.get(1..sep)?, env)?;
    let tgt = toks.get(sep + 1..)?;
    let (owner, rest) = people_head(tgt)?;
    let bound = bound_expr_ref(rest, env)?;
    Some(if add {
        BoundEff::AddPerson {
            owner,
            bound,
            player: who,
        }
    } else {
        BoundEff::RemovePerson {
            owner,
            bound,
            player: who,
        }
    })
}

/// `owners of`, `members of`, `owner of` ... returns (is_owner, tokens after `of`).
fn people_head(toks: &[Tok]) -> Option<(bool, &[Tok])> {
    let w = match toks.first()? {
        Tok::Word(w) => w.to_ascii_lowercase(),
        _ => return None,
    };
    let owner = match w.as_str() {
        "owners" | "owner" => true,
        "members" | "member" => false,
        _ => return None,
    };
    if !is(toks.get(1)?, "of") {
        return None;
    }
    Some((owner, toks.get(2..)?))
}

fn set(toks: &[Tok], env: &Env) -> Option<BoundEff> {
    let to = find(toks, 1, &["to"])?;
    let (target, value) = (toks.get(1..to)?, toks.get(to + 1..)?);
    if let Some((owner, rest)) = people_head(target) {
        return Some(BoundEff::SetPerson {
            owner,
            bound: bound_expr_ref(rest, env)?,
            player: player(value, env)?,
        });
    }
    if is(target.first()?, "flag") {
        let of = find(target, 1, &["of"])?;
        return Some(BoundEff::SetFlag {
            flag: text(target.get(1..of)?, env)?,
            bound: bound_expr_ref(target.get(of + 1..)?, env)?,
            value: Some(expr_of(value, env, &[Ty::Boolean])?),
        });
    }
    let greater = match target.first()? {
        t if is(t, "greater") => true,
        t if is(t, "lesser") => false,
        _ => return None,
    };
    if !is(target.get(1)?, "corner") || !is(target.get(2)?, "of") {
        return None;
    }
    Some(BoundEff::SetCorner {
        greater,
        bound: bound_expr_ref(target.get(3..)?, env)?,
        loc: location(value, env)?,
    })
}

/// Conditions that mention a bound.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    if !has_bound_word(toks) {
        return None;
    }
    let c = cond(toks, env)?;
    Some(Cond::Bound(Box::new(c)))
}

fn cond(toks: &[Tok], env: &Env) -> Option<BoundCond> {
    // `bound X exists` / `bound X does not exist`
    if let Some(last) = toks.last()
        && (is(last, "exist") || is(last, "exists"))
        && toks.len() > 2
    {
        let mut head = toks.get(..toks.len() - 1)?;
        let mut negate = false;
        if let Some(n) = head.last()
            && is(n, "not")
        {
            head = head.get(..head.len() - 1)?;
            negate = true;
        }
        if head.last().is_some_and(|t| is(t, "does") || is(t, "do")) {
            head = head.get(..head.len() - 1)?;
        } else if head.last().is_some_and(|t| is(t, "doesn't") || is(t, "don't")) {
            head = head.get(..head.len() - 1)?;
            negate = true;
        }
        let value = bound_expr_ref(head, env)?;
        if !matches!(value, Expr::Bound(_)) {
            return None;
        }
        return Some(BoundCond::Exists { value, negate });
    }
    // `bound X has flag "pvp"`, `bound X does not have flag "pvp"`
    if is(toks.first()?, "bound") || (is(toks.first()?, "the") && toks.get(1).is_some_and(|t| is(t, "bound")))
    {
        let h = find(toks, 1, &["has", "have", "doesn't", "does", "don't", "do"])?;
        let mut negate = false;
        let mut p = h;
        if is(&toks[p], "doesn't") || is(&toks[p], "don't") {
            negate = true;
            p += 1;
        } else if is(&toks[p], "does") || is(&toks[p], "do") {
            p += 1;
            if toks.get(p).is_some_and(|t| is(t, "not")) {
                negate = true;
                p += 1;
            }
        } else {
            p += 1;
        }
        if toks.get(p).is_some_and(|t| is(t, "have") || is(t, "has")) {
            p += 1;
        }
        if toks.get(p).is_some_and(|t| is(t, "the")) {
            p += 1;
        }
        if !toks.get(p).is_some_and(|t| is(t, "flag")) {
            return None;
        }
        return Some(BoundCond::HasFlag {
            bound: bound_expr_ref(toks.get(..h)?, env)?,
            flag: text(toks.get(p + 1..)?, env)?,
            negate,
        });
    }
    // `X is [not] within bound B` / `X is [not] an owner of bound B`
    let i = find(toks, 0, &["is", "are"])?;
    let subject = toks.get(..i)?;
    let mut rest = toks.get(i + 1..)?;
    let mut negate = false;
    if rest.first().is_some_and(|t| is(t, "not")) {
        negate = true;
        rest = rest.get(1..)?;
    }
    if rest.first().is_some_and(|t| is(t, "a") || is(t, "an")) {
        rest = rest.get(1..)?;
    }
    let w = match rest.first()? {
        Tok::Word(w) => w.to_ascii_lowercase(),
        _ => return None,
    };
    match w.as_str() {
        "within" | "in" | "inside" => {
            let b = rest.get(1..)?;
            if !is(b.first()?, "bound") && !is(b.first()?, "the") {
                return None;
            }
            Some(BoundCond::Within {
                subject: place(subject, env)?,
                bound: bound_expr_ref(b, env)?,
                negate,
            })
        }
        "owner" | "member" => {
            if !is(rest.get(1)?, "of") {
                return None;
            }
            Some(BoundCond::Role {
                player: player(subject, env)?,
                bound: bound_expr_ref(rest.get(2..)?, env)?,
                owner: w == "owner",
                negate,
            })
        }
        _ => None,
    }
}

// ---------- events ----------

/// `bound enter`, `player exit bound`, ... Returns the event kind.
pub fn parse_event(s: &str) -> Option<EventKind> {
    let words: Vec<&str> = s
        .split_whitespace()
        .filter(|w| !matches!(*w, "player" | "of" | "a" | "the"))
        .collect();
    match words.as_slice() {
        ["bound", "enter"] | ["enter", "bound"] | ["bound", "entry"] => Some(EventKind::BoundEnter),
        ["bound", "exit" | "leave"] | ["exit" | "leave", "bound"] | ["bound", "exit" | "leave", ..] => {
            Some(EventKind::BoundExit)
        }
        _ => None,
    }
}

// ---------- list heads for `loop` and `set {x::*} to` ----------

/// Does this token list start with a bound list expression?
pub fn is_list_head(toks: &[Tok]) -> bool {
    let w = |i: usize| match toks.get(i) {
        Some(Tok::Word(w)) => w.to_ascii_lowercase(),
        _ => String::new(),
    };
    match w(0).as_str() {
        "all" => w(1) == "bounds",
        "bounds" => w(1) == "at",
        "owners" | "members" => w(1) == "of",
        _ => false,
    }
}

// ---------- running ----------

fn world_of(w: Option<String>) -> String {
    w.unwrap_or_else(|| DEFAULT_WORLD.to_string())
}

fn same_person(list: &[PlayerRef], p: &PlayerRef) -> bool {
    list.iter().any(|x| x.uuid == p.uuid)
}

impl Engine {
    /// Is there anything to check each tick?
    pub fn bounds_wanted(&self) -> bool {
        !self.bounds.is_empty()
            && (self.table().has_event(EventKind::BoundEnter) || self.table().has_event(EventKind::BoundExit))
    }

    /// Check every online player. Called from `tick`.
    pub fn bounds_poll(&mut self, host: &mut dyn Host) {
        let players = host.online_players();
        let mut seen: HashSet<String> = HashSet::with_capacity(players.len());
        for p in &players {
            seen.insert(p.uuid.clone());
            if let Some((x, y, z, w)) = host.player_location(p) {
                self.bounds_moved(p, x, y, z, &w, host);
            }
        }
        self.bounds.tracker.retain(|k, _| seen.contains(k));
    }

    /// A player is at this point now. Fires `on bound exit` and `on bound
    /// enter` for bounds they left and entered since the last call. Hosts with
    /// a move event can call this directly instead of waiting for the poll.
    pub fn bounds_moved(&mut self, p: &PlayerRef, x: f64, y: f64, z: f64, world: &str, host: &mut dyn Host) {
        let blk = (block(x), block(y), block(z));
        if let Some(t) = self.bounds.tracker.get(&p.uuid)
            && t.blk == blk
            && t.world == world
        {
            return;
        }
        let now_in = self.bounds.ids_at(world, x, y, z);
        let t = self.bounds.tracker.entry(p.uuid.clone()).or_default();
        let old = std::mem::replace(&mut t.inside, now_in.clone());
        t.blk = blk;
        if t.world != world {
            t.world = world.to_string();
        }
        if old == now_in {
            return;
        }
        let exits = old
            .iter()
            .filter(|id| !now_in.contains(id) && self.bounds.get(id).is_some())
            .cloned()
            .collect::<Vec<_>>();
        let enters = now_in
            .iter()
            .filter(|id| !old.contains(id))
            .cloned()
            .collect::<Vec<_>>();
        for (kind, ids) in [(EventKind::BoundExit, exits), (EventKind::BoundEnter, enters)] {
            if !self.table().has_event(kind) {
                continue;
            }
            for id in ids {
                let mut ctx = EventCtx::player(p.clone());
                ctx.args = vec![Value::Text(id)];
                self.fire(kind, ctx, host);
            }
        }
    }

    /// The id a bound expression names. `bound "x"` gives "x" even when no such
    /// bound exists, so effects can say so.
    fn bound_id(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<Option<String>, String> {
        let e = match e {
            Expr::Bound(b) => match &**b {
                BoundExpr::Named(inner) => inner,
                _ => e,
            },
            _ => e,
        };
        match self.eval(e, ex, host)? {
            Value::Text(t) => Ok(Some(t)),
            Value::None => Ok(None),
            other => Err(format!("expected a bound, got {} ({other})", other.type_name())),
        }
    }

    fn bound_point(
        &mut self,
        e: &Expr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Option<(f64, f64, f64, String)>, String> {
        let v = self.eval(e, ex, host)?;
        match v {
            Value::Location { x, y, z, world } => Ok(Some((x, y, z, world_of(world)))),
            Value::Player(_) | Value::Text(_) => {
                let Some(p) = online(&v, &mut ex.seen, host) else {
                    return Ok(None);
                };
                Ok(host.player_location(&p))
            }
            Value::None => Ok(None),
            other => Err(format!(
                "expected a location or player, got {} ({other})",
                other.type_name()
            )),
        }
    }

    #[inline(never)]
    pub(crate) fn bound_eval(
        &mut self,
        b: &BoundExpr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        Ok(match b {
            BoundExpr::Event => ex.ctx.args.first().cloned().unwrap_or(Value::None),
            BoundExpr::All => Value::List(
                self.bounds
                    .all()
                    .iter()
                    .map(|b| Value::Text(b.id.clone()))
                    .collect(),
            ),
            BoundExpr::Named(e) => match self.bound_id(e, ex, host)? {
                Some(id) => self
                    .bounds
                    .get(&id)
                    .map_or(Value::None, |b| Value::Text(b.id.clone())),
                None => Value::None,
            },
            BoundExpr::At(e) => match self.bound_point(e, ex, host)? {
                Some((x, y, z, w)) => self
                    .bounds
                    .ids_at(&w, x, y, z)
                    .into_iter()
                    .next()
                    .map_or(Value::None, Value::Text),
                None => Value::None,
            },
            BoundExpr::BoundsAt(e) => match self.bound_point(e, ex, host)? {
                Some((x, y, z, w)) => Value::List(
                    self.bounds
                        .ids_at(&w, x, y, z)
                        .into_iter()
                        .map(Value::Text)
                        .collect(),
                ),
                None => Value::List(Vec::new()),
            },
            BoundExpr::People(owner, e) => {
                let id = self.bound_id(e, ex, host)?;
                let list = id.and_then(|id| {
                    self.bounds
                        .get(&id)
                        .map(|b| if *owner { &b.owners } else { &b.members })
                });
                Value::List(
                    list.map(|l| l.iter().cloned().map(Value::Player).collect())
                        .unwrap_or_default(),
                )
            }
            BoundExpr::Corner(greater, e) => {
                let id = self.bound_id(e, ex, host)?;
                match id.and_then(|id| self.bounds.get(&id)) {
                    Some(b) => {
                        let c = if *greater { b.max } else { b.min };
                        Value::Location {
                            x: f64::from(c[0]),
                            y: f64::from(c[1]),
                            z: f64::from(c[2]),
                            world: Some(b.world.clone()),
                        }
                    }
                    None => Value::None,
                }
            }
            BoundExpr::WorldOf(e) => {
                let id = self.bound_id(e, ex, host)?;
                id.and_then(|id| self.bounds.get(&id).map(|b| Value::World(b.world.clone())))
                    .unwrap_or(Value::None)
            }
            BoundExpr::Volume(e) => {
                let id = self.bound_id(e, ex, host)?;
                id.and_then(|id| self.bounds.get(&id).map(|b| Value::Num(b.volume())))
                    .unwrap_or(Value::None)
            }
            BoundExpr::Flag(f, e) => {
                let flag = self.text(f, ex, host)?.to_ascii_lowercase();
                let id = self.bound_id(e, ex, host)?;
                Value::Bool(
                    id.and_then(|id| self.bounds.get(&id).and_then(|b| b.flags.get(&flag).copied()))
                        .unwrap_or(false),
                )
            }
        })
    }

    #[inline(never)]
    pub(crate) fn bound_cond(
        &mut self,
        c: &BoundCond,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<bool, String> {
        Ok(match c {
            BoundCond::Within {
                subject,
                bound,
                negate,
            } => {
                let id = self.bound_id(bound, ex, host)?;
                let pt = self.bound_point(subject, ex, host)?;
                let inside = match (id, pt) {
                    (Some(id), Some((x, y, z, w))) => self
                        .bounds
                        .get(&id)
                        .is_some_and(|b| b.world == w && b.contains(x, y, z)),
                    _ => false,
                };
                inside != *negate
            }
            BoundCond::Exists { value, negate } => {
                let v = self.eval(value, ex, host)?;
                !matches!(v, Value::None) != *negate
            }
            BoundCond::Role {
                player,
                bound,
                owner,
                negate,
            } => {
                let id = self.bound_id(bound, ex, host)?;
                let p = self.player(player, ex, host)?;
                let yes = match (id, p) {
                    (Some(id), Some(p)) => self
                        .bounds
                        .get(&id)
                        .is_some_and(|b| same_person(if *owner { &b.owners } else { &b.members }, &p)),
                    _ => false,
                };
                yes != *negate
            }
            BoundCond::HasFlag { bound, flag, negate } => {
                let flag = self.text(flag, ex, host)?.to_ascii_lowercase();
                let id = self.bound_id(bound, ex, host)?;
                let yes = id
                    .and_then(|id| self.bounds.get(&id).and_then(|b| b.flags.get(&flag).copied()))
                    .unwrap_or(false);
                yes != *negate
            }
        })
    }

    #[inline(never)]
    pub(crate) fn bound_effect(
        &mut self,
        e: &BoundEff,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        match e {
            BoundEff::Create { id, a, b, temporary } => {
                let Some(id) = self.bound_id(id, ex, host)? else {
                    return Ok(());
                };
                let id = id.trim().to_string();
                if id.is_empty() {
                    host.log(LogLevel::Warn, "a bound needs an id");
                    return Ok(());
                }
                let (Some(pa), Some(pb)) = (self.bound_point(a, ex, host)?, self.bound_point(b, ex, host)?)
                else {
                    return Ok(());
                };
                if pa.3 != pb.3 {
                    return Err(format!("both corners of bound '{id}' must be in the same world"));
                }
                let (ca, cb) = (
                    [block(pa.0), block(pa.1), block(pa.2)],
                    [block(pb.0), block(pb.1), block(pb.2)],
                );
                self.bounds.create(Bound {
                    id,
                    world: pa.3,
                    min: [0, 1, 2].map(|i| ca[i].min(cb[i])),
                    max: [0, 1, 2].map(|i| ca[i].max(cb[i])),
                    owners: Vec::new(),
                    members: Vec::new(),
                    flags: BTreeMap::new(),
                    temporary: *temporary,
                });
            }
            BoundEff::Delete(e) => {
                if let Some(id) = self.bound_id(e, ex, host)?
                    && !self.bounds.delete(&id)
                {
                    host.log(LogLevel::Warn, &format!("there is no bound '{id}' to delete"));
                    return Ok(());
                }
            }
            BoundEff::SetPerson { owner, bound, player } | BoundEff::AddPerson { owner, bound, player } => {
                let add = matches!(e, BoundEff::AddPerson { .. });
                let (id, p) = (self.bound_id(bound, ex, host)?, self.player(player, ex, host)?);
                let (Some(id), Some(p)) = (id, p) else {
                    return Ok(());
                };
                let Some(b) = self.bounds.get_mut(&id) else {
                    host.log(LogLevel::Warn, &format!("there is no bound '{id}'"));
                    return Ok(());
                };
                let list = if *owner { &mut b.owners } else { &mut b.members };
                if !add {
                    list.clear();
                }
                if !same_person(list, &p) {
                    list.push(p);
                }
            }
            BoundEff::RemovePerson { owner, bound, player } => {
                let (id, p) = (self.bound_id(bound, ex, host)?, self.player(player, ex, host)?);
                let (Some(id), Some(p)) = (id, p) else {
                    return Ok(());
                };
                if let Some(b) = self.bounds.get_mut(&id) {
                    let list = if *owner { &mut b.owners } else { &mut b.members };
                    list.retain(|x| x.uuid != p.uuid);
                }
            }
            BoundEff::SetFlag { bound, flag, value } => {
                let flag = self.text(flag, ex, host)?.trim().to_ascii_lowercase();
                let v = match value {
                    Some(v) => Some(matches!(self.eval(v, ex, host)?, Value::Bool(true))),
                    None => None,
                };
                let Some(id) = self.bound_id(bound, ex, host)? else {
                    return Ok(());
                };
                let Some(b) = self.bounds.get_mut(&id) else {
                    host.log(LogLevel::Warn, &format!("there is no bound '{id}'"));
                    return Ok(());
                };
                match v {
                    Some(v) => {
                        b.flags.insert(flag, v);
                    }
                    None => {
                        b.flags.remove(&flag);
                    }
                }
            }
            BoundEff::SetCorner { greater, bound, loc } => {
                let (id, pt) = (self.bound_id(bound, ex, host)?, self.bound_point(loc, ex, host)?);
                let (Some(id), Some((x, y, z, w))) = (id, pt) else {
                    return Ok(());
                };
                let Some(b) = self.bounds.get_mut(&id) else {
                    host.log(LogLevel::Warn, &format!("there is no bound '{id}'"));
                    return Ok(());
                };
                if b.world != w {
                    return Err(format!("that location is not in the world of bound '{id}'"));
                }
                let c = [block(x), block(y), block(z)];
                let (mut lo, mut hi) = (b.min, b.max);
                if *greater {
                    hi = c;
                } else {
                    lo = c;
                }
                b.min = [0, 1, 2].map(|i| lo[i].min(hi[i]));
                b.max = [0, 1, 2].map(|i| lo[i].max(hi[i]));
                let nb = b.clone();
                self.bounds.create(nb);
            }
        }
        // Every effect except a failed lookup changes the bounds.
        let now = host.now();
        self.bounds.touch(now);
        Ok(())
    }
}
