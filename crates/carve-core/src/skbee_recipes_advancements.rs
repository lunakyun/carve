//! SkBee-style recipes, advancements and statistics.
//!
//! Lines here have free-form words (`diamond sword`, `where a is stick`), so
//! they are parsed by hand, like `world_effects`. The compiler calls
//! `parse_effect` and `parse_cond` first. Recipe lines are fully static: the
//! id, shape and items are checked when the script loads.

use crate::compile::{Cond, Effect};
use crate::engine::{Engine, Exec, online};
use crate::expr::{Env, Expr, Ty, parse_expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::Value;

/// A recipe ingredient: an item id or a tag, both with a namespace.
#[derive(Debug, Clone, PartialEq)]
pub enum Ingredient {
    Item(String),
    Tag(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookKind {
    Smelting,
    Blasting,
    Smoking,
    Campfire,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecipeKind {
    /// Rows of 1 to 3 characters; a space is an empty slot.
    Shaped {
        rows: Vec<String>,
        key: Vec<(char, Ingredient)>,
    },
    Shapeless(Vec<Ingredient>),
    Cooking {
        kind: CookKind,
        ingredient: Ingredient,
        experience: f32,
        ticks: u32,
    },
}

/// A recipe to register, already checked.
#[derive(Debug, Clone, PartialEq)]
pub struct RecipeDef {
    /// Full id with a namespace (`carve:name` when the script gave none).
    pub id: String,
    /// Result item id with namespace.
    pub result: String,
    pub count: u32,
    pub kind: RecipeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatOp {
    Set,
    Add,
    Remove,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkbFx {
    /// Grant or revoke an advancement, or one criterion of it.
    Advancement {
        id: Expr,
        criterion: Option<Expr>,
        who: Expr,
        grant: bool,
    },
    Recipe(RecipeDef),
    /// Unlock (`give`) or lock a recipe in the recipe book.
    Discover { recipe: Expr, who: Expr, give: bool },
    Stat {
        name: Expr,
        who: Expr,
        value: Expr,
        op: StatOp,
    },
}

/// `player has advancement "x"`.
#[derive(Debug, Clone, PartialEq)]
pub struct SkbCond {
    pub who: Expr,
    pub id: Expr,
    pub negate: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkbExpr {
    /// `statistic "mined:stone" of player`.
    Stat(Expr, Expr),
    /// `advancement id` in `on advancement done`, `recipe id` in `on recipe discover`.
    EventId,
}

pub const STAT_CATEGORIES: &[&str] = &[
    "mined",
    "crafted",
    "used",
    "broken",
    "picked_up",
    "dropped",
    "killed",
    "killed_by",
    "custom",
];

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

fn text_expr(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Text])
}

fn player_expr(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Player])
}

/// Add `minecraft:` when there is no namespace, lowercase, no spaces.
pub fn namespaced(s: &str, default_ns: &str) -> String {
    let s: String = s.trim().to_ascii_lowercase().replace(' ', "_");
    if s.contains(':') { s } else { format!("{default_ns}:{s}") }
}

/// Entry point for effect lines. `Ok(None)` means "not mine".
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    let second = match toks.get(1) {
        Some(Tok::Word(w)) => w.to_ascii_lowercase(),
        _ => String::new(),
    };
    let fx = match first.to_ascii_lowercase().as_str() {
        "grant" | "award" => advancement(toks, env, true)?,
        "revoke" | "take" if second == "advancement" || second == "criterion" => {
            advancement(toks, env, false)?
        }
        "register" | "create" if toks.iter().any(|t| is(t, "recipe")) => recipe(toks, env)?,
        "discover" | "unlock" | "give" if second == "recipe" => discover(toks, env, true),
        "undiscover" | "lock" | "take" | "revoke" if second == "recipe" => discover(toks, env, false),
        "set" if second == "statistic" => stat(toks, env, StatOp::Set)?,
        "add" if toks.iter().any(|t| is(t, "statistic")) => stat(toks, env, StatOp::Add)?,
        "remove" if toks.iter().any(|t| is(t, "statistic")) => stat(toks, env, StatOp::Remove)?,
        _ => None,
    };
    Ok(fx.map(|f| Effect::Skb(Box::new(f))))
}

/// `grant advancement X to P`, `revoke criterion C of advancement X from P`.
fn advancement(toks: &[Tok], env: &Env, grant: bool) -> Result<Option<SkbFx>, String> {
    let mut i = 1;
    if toks.get(i).is_some_and(|t| is(t, "the")) {
        i += 1;
    }
    let mut criterion = None;
    if toks.get(i).is_some_and(|t| is(t, "criterion")) {
        let Some(of) = find(toks, i, &["of"]) else {
            return Ok(None);
        };
        criterion = text_expr(&toks[i + 1..of], env);
        if criterion.is_none() {
            return Err("expected the criterion name, like: grant criterion \"mine_stone\" of advancement \"story/mine_stone\" to player".into());
        }
        i = of + 1;
        if toks.get(i).is_some_and(|t| is(t, "the")) {
            i += 1;
        }
    }
    if !toks.get(i).is_some_and(|t| is(t, "advancement")) {
        return Ok(None);
    }
    let sep = if grant { "to" } else { "from" };
    let Some(at) = find(toks, i + 1, &[sep]) else {
        return Err(format!(
            "expected '{sep} <player>' after the advancement, like: {} advancement \"story/mine_stone\" {sep} player",
            if grant { "grant" } else { "revoke" }
        ));
    };
    let (Some(id), Some(who)) = (text_expr(&toks[i + 1..at], env), player_expr(&toks[at + 1..], env)) else {
        return Err("could not read the advancement id or the player".into());
    };
    Ok(Some(SkbFx::Advancement {
        id,
        criterion,
        who,
        grant,
    }))
}

fn discover(toks: &[Tok], env: &Env, give: bool) -> Option<SkbFx> {
    let at = find(toks, 2, if give { &["for", "to"] } else { &["for", "from"] })?;
    Some(SkbFx::Discover {
        recipe: text_expr(&toks[2..at], env)?,
        who: player_expr(&toks[at + 1..], env)?,
        give,
    })
}

fn stat(toks: &[Tok], env: &Env, op: StatOp) -> Result<Option<SkbFx>, String> {
    let bad = || "statistics look like: set statistic \"custom:jump\" of player to 5".to_string();
    let (name, who, value) = match op {
        StatOp::Set => {
            let Some(of) = find(toks, 2, &["of"]) else {
                return Err(bad());
            };
            let Some(to) = find(toks, of, &["to"]) else {
                return Err(bad());
            };
            (&toks[2..of], &toks[of + 1..to], &toks[to + 1..])
        }
        StatOp::Add | StatOp::Remove => {
            let sep = if op == StatOp::Add { "to" } else { "from" };
            let Some(s) = find(toks, 1, &[sep]) else {
                return Ok(None);
            };
            if !toks.get(s + 1).is_some_and(|t| is(t, "statistic")) {
                return Ok(None);
            }
            let Some(of) = find(toks, s + 2, &["of"]) else {
                return Err(bad());
            };
            (&toks[s + 2..of], &toks[of + 1..], &toks[1..s])
        }
    };
    let (Some(name), Some(who), Some(value)) = (
        text_expr(name, env),
        player_expr(who, env),
        expr_of(value, env, &[Ty::Number]),
    ) else {
        return Err(bad());
    };
    Ok(Some(SkbFx::Stat { name, who, value, op }))
}

/// One ingredient: an item literal (`diamond`), or text (`"minecraft:diamond"`, `"#minecraft:logs"`).
fn ingredient(toks: &[Tok], env: &Env) -> Result<(Ingredient, u32), String> {
    if let [Tok::Str(s)] = toks {
        let s = s.trim();
        return Ok(match s.strip_prefix('#') {
            Some(tag) => (Ingredient::Tag(namespaced(tag, "minecraft")), 1),
            None => (Ingredient::Item(namespaced(s, "minecraft")), 1),
        });
    }
    match parse_expr(toks, env) {
        Ok(Expr::Item { id, count }) => Ok((Ingredient::Item(namespaced(&id, "minecraft")), count.max(1))),
        _ => Err("expected an item like 'diamond' or text like \"minecraft:diamond\" (use \"#minecraft:logs\" for a tag)".into()),
    }
}

fn split_top<'a>(toks: &'a [Tok], seps: &[&str]) -> Vec<&'a [Tok]> {
    toks.split(|t| seps.iter().any(|s| is(t, s))).collect()
}

/// `register shaped recipe "id" for <item> with shape "aa", "a " where a is diamond`
/// and the shapeless and cooking forms.
fn recipe(toks: &[Tok], env: &Env) -> Result<Option<SkbFx>, String> {
    let Some(rec) = find(toks, 1, &["recipe"]) else {
        return Ok(None);
    };
    let kind_word = toks[1..rec]
        .iter()
        .filter_map(|t| match t {
            Tok::Word(w) => Some(w.to_ascii_lowercase()),
            _ => None,
        })
        .filter(|w| w != "a" && w != "new" && w != "custom")
        .collect::<Vec<_>>()
        .join(" ");
    let ck = match kind_word.as_str() {
        "shaped" => None,
        "shapeless" => None,
        "furnace" | "smelting" => Some(CookKind::Smelting),
        "blasting" | "blast furnace" => Some(CookKind::Blasting),
        "smoking" | "smoker" => Some(CookKind::Smoking),
        "campfire" => Some(CookKind::Campfire),
        "smithing" | "smithing table" => {
            return Err("smithing recipes can't be registered: Pumpkin's plugin API has no smithing recipe type".into());
        }
        "" => return Ok(None),
        _ => return Ok(None),
    };
    let Some(Tok::Str(id)) = toks.get(rec + 1) else {
        return Err("recipes need a plain text id first, like: register shaped recipe \"my_sword\" for diamond sword ...".into());
    };
    if id.trim().is_empty() || id.chars().any(|c| c.is_whitespace()) {
        return Err("recipe ids can't be empty or hold spaces".into());
    }
    let id = namespaced(id, "carve");
    if !toks.get(rec + 2).is_some_and(|t| is(t, "for")) {
        return Err("expected 'for <result item>' after the recipe id".into());
    }
    let body = &toks[rec + 3..];
    let Some(tail) = find(body, 0, &["with", "using", "from"]) else {
        return Err("expected 'with shape ...' (shaped) or 'using ...' (shapeless, cooking)".into());
    };
    let result = match parse_expr(&body[..tail], env) {
        Ok(Expr::Item { id, count }) => (namespaced(&id, "minecraft"), count.max(1)),
        _ => return Err("the recipe result must be an item, like 'diamond sword' or '2 torches'".into()),
    };
    let rest = &body[tail..];
    let kind = if let Some(k) = ck {
        cooking(rest, env, k)?
    } else if kind_word == "shaped" {
        shaped(rest, env)?
    } else {
        shapeless(rest, env)?
    };
    Ok(Some(SkbFx::Recipe(RecipeDef {
        id,
        result: result.0,
        count: result.1,
        kind,
    })))
}

fn shaped(rest: &[Tok], env: &Env) -> Result<RecipeKind, String> {
    // `with shape "aa", "a " where a is diamond and b is stick`
    let usage = "shaped recipes look like: register shaped recipe \"id\" for diamond sword with shape \"a\", \"a\", \"s\" where a is diamond and s is stick";
    if !(rest.first().is_some_and(|t| is(t, "with")) && rest.get(1).is_some_and(|t| is(t, "shape"))) {
        return Err(usage.into());
    }
    let Some(wh) = find(rest, 2, &["where"]) else {
        return Err(usage.into());
    };
    let mut rows = Vec::new();
    for part in split_top(&rest[2..wh], &[","]) {
        match part {
            [Tok::Str(s)] => rows.push(s.clone()),
            _ => return Err(usage.into()),
        }
    }
    if rows.is_empty() || rows.len() > 3 {
        return Err("a shape has 1 to 3 rows".into());
    }
    let width = rows[0].chars().count();
    if width == 0 || width > 3 || rows.iter().any(|r| r.chars().count() != width) {
        return Err("every shape row must have the same length, 1 to 3 characters (use a space for an empty slot)".into());
    }
    let mut key: Vec<(char, Ingredient)> = Vec::new();
    for part in split_top(&rest[wh + 1..], &["and", ","]) {
        let sym = match part.first() {
            Some(Tok::Word(w)) if w.chars().count() == 1 => w.chars().next(),
            Some(Tok::Str(s)) if s.chars().count() == 1 => s.chars().next(),
            Some(Tok::Num(n)) if (0.0..10.0).contains(n) && n.fract() == 0.0 => {
                char::from_digit(*n as u32, 10)
            }
            _ => None,
        };
        let (Some(sym), Some(sep)) = (sym, part.get(1)) else {
            return Err(usage.into());
        };
        if !(is(sep, "is") || is(sep, "=") || is(sep, "as")) || sym == ' ' {
            return Err(usage.into());
        }
        let (ing, _) = ingredient(&part[2..], env)?;
        if key.iter().any(|(c, _)| *c == sym) {
            return Err(format!("'{sym}' is defined twice"));
        }
        key.push((sym, ing));
    }
    for c in rows.iter().flat_map(|r| r.chars()) {
        if c != ' ' && !key.iter().any(|(k, _)| *k == c) {
            return Err(format!("the shape uses '{c}' but 'where' does not say what it is"));
        }
    }
    Ok(RecipeKind::Shaped { rows, key })
}

fn shapeless(rest: &[Tok], env: &Env) -> Result<RecipeKind, String> {
    if !rest.first().is_some_and(|t| is(t, "using")) {
        return Err("shapeless recipes look like: register shapeless recipe \"id\" for 2 torches using coal and stick".into());
    }
    let mut out = Vec::new();
    for part in split_top(&rest[1..], &["and", ","]) {
        let (ing, n) = ingredient(part, env)?;
        for _ in 0..n {
            out.push(ing.clone());
        }
    }
    if out.is_empty() || out.len() > 9 {
        return Err("a shapeless recipe takes 1 to 9 ingredients".into());
    }
    Ok(RecipeKind::Shapeless(out))
}

fn cooking(rest: &[Tok], env: &Env, kind: CookKind) -> Result<RecipeKind, String> {
    let usage = "cooking recipes look like: register furnace recipe \"id\" for iron ingot using raw iron [with xp 0.7] [cooking time 10 seconds]";
    if !rest.first().is_some_and(|t| is(t, "using") || is(t, "from")) {
        return Err(usage.into());
    }
    let end = find(rest, 1, &["with", "cooking", "in"]).unwrap_or(rest.len());
    let (ingredient, _) = ingredient(&rest[1..end], env)?;
    let mut experience = 0.0f32;
    let mut ticks = match kind {
        CookKind::Smelting | CookKind::Campfire => 200,
        CookKind::Blasting | CookKind::Smoking => 100,
    } as u32;
    let mut i = end;
    while i < rest.len() {
        let t = &rest[i];
        if is(t, "with") && rest.get(i + 1).is_some_and(|t| is(t, "xp") || is(t, "exp") || is(t, "experience")) {
            let Some(Tok::Num(n)) = rest.get(i + 2) else {
                return Err(usage.into());
            };
            experience = n.clamp(0.0, 1000.0) as f32;
            i += 3;
        } else if is(t, "cooking") || is(t, "in") {
            let mut j = i + 1;
            if rest.get(j).is_some_and(|t| is(t, "time")) {
                j += 1;
            }
            let Some(Tok::Num(n)) = rest.get(j) else {
                return Err(usage.into());
            };
            let unit = match rest.get(j + 1) {
                Some(Tok::Word(w)) => w.to_ascii_lowercase(),
                _ => "ticks".into(),
            };
            let mult = match unit.as_str() {
                "tick" | "ticks" => 1.0,
                "second" | "seconds" => 20.0,
                _ => return Err("cooking time is in ticks or seconds".into()),
            };
            ticks = (n * mult).clamp(1.0, 1_000_000.0) as u32;
            i = j + 2;
        } else {
            return Err(usage.into());
        }
    }
    Ok(RecipeKind::Cooking {
        kind,
        ingredient,
        experience,
        ticks,
    })
}

/// `player has [completed] advancement "x"` and the negations.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    let adv = find(toks, 1, &["advancement"])?;
    let h = find(toks, 0, &["has", "have"])?;
    if h > adv {
        return None;
    }
    let before = &toks[..h];
    let (subject, negate) = match before.split_last() {
        Some((n, s)) if is(n, "not") => match s.split_last() {
            Some((d, s)) if is(d, "does") || is(d, "do") => (s, true),
            _ => (s, true),
        },
        Some((n, s)) if is(n, "doesn't") || is(n, "don't") || is(n, "hasn't") || is(n, "haven't") => (s, true),
        _ => (before, false),
    };
    let negate = negate
        || toks
            .get(h + 1)
            .is_some_and(|t| is(t, "not"));
    let start = h + 1 + usize::from(toks.get(h + 1).is_some_and(|t| is(t, "not")));
    if !toks[start..adv].iter().all(|t| {
        is(t, "completed") || is(t, "done") || is(t, "achieved") || is(t, "earned") || is(t, "got") || is(t, "the")
    }) {
        return None;
    }
    Some(Cond::Skb(Box::new(SkbCond {
        who: player_expr(subject, env)?,
        id: text_expr(&toks[adv + 1..], env)?,
        negate,
    })))
}

// ---------- running ----------

/// `mined:stone`, `custom:jump`, `play time` -> (category, name).
pub fn split_stat(s: &str) -> Result<(String, String), String> {
    let s = s.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    if let Some((c, n)) = s.split_once(':')
        && STAT_CATEGORIES.contains(&c)
    {
        let n = n.strip_prefix("minecraft:").unwrap_or(n);
        return Ok((c.to_string(), n.to_string()));
    }
    let n = s.strip_prefix("minecraft:").unwrap_or(&s);
    if n.is_empty() {
        return Err("empty statistic name".into());
    }
    Ok(("custom".to_string(), n.to_string()))
}

impl Engine {
    #[inline(never)]
    pub(crate) fn skb_effect(&mut self, fx: &SkbFx, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        match fx {
            SkbFx::Advancement {
                id,
                criterion,
                who,
                grant,
            } => {
                let id = namespaced(&self.text(id, ex, host)?, "minecraft");
                let crit = match criterion {
                    Some(c) => Some(self.text(c, ex, host)?),
                    None => None,
                };
                if let Some(p) = self.player(who, ex, host)? {
                    host.advancement(&p, &id, crit.as_deref(), *grant);
                }
            }
            SkbFx::Recipe(def) => {
                if let Err(e) = host.register_recipe(def) {
                    return Err(format!("recipe '{}': {e}", def.id));
                }
            }
            SkbFx::Discover { recipe, who, give } => {
                let id = namespaced(&self.text(recipe, ex, host)?, "minecraft");
                if let Some(p) = self.player(who, ex, host)? {
                    host.discover_recipe(&p, &id, *give);
                }
            }
            SkbFx::Stat { name, who, value, op } => {
                let (cat, name) = split_stat(&self.text(name, ex, host)?)?;
                let n = self.num(value, ex, host)?;
                let n = if n.is_finite() { n.clamp(-2_147_483_648.0, 2_147_483_647.0) as i64 } else { 0 };
                if let Some(p) = self.player(who, ex, host)? {
                    let new = match op {
                        StatOp::Set => n,
                        StatOp::Add | StatOp::Remove => {
                            let cur = host.statistic(&p, &cat, &name).unwrap_or(0);
                            let d = if *op == StatOp::Add { n } else { -n };
                            (cur + d).clamp(0, i32::MAX as i64)
                        }
                    };
                    host.set_statistic(&p, &cat, &name, new.max(0));
                }
            }
        }
        Ok(())
    }

    #[inline(never)]
    pub(crate) fn skb_cond(&mut self, c: &SkbCond, ex: &mut Exec, host: &mut dyn Host) -> Result<bool, String> {
        let id = namespaced(&self.text(&c.id, ex, host)?, "minecraft");
        let Some(p) = self.player(&c.who, ex, host)? else {
            return Ok(false);
        };
        Ok(host.has_advancement(&p, &id) != c.negate)
    }

    #[inline(never)]
    pub(crate) fn skb_eval(&mut self, e: &SkbExpr, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        match e {
            SkbExpr::EventId => Ok(match &ex.ctx.message {
                Some(m) => Value::Text(m.clone()),
                None => Value::None,
            }),
            SkbExpr::Stat(name, who) => {
                let (cat, name) = split_stat(&self.text(name, ex, host)?)?;
                let v = self.eval(who, ex, host)?;
                let Some(p) = online(&v, &mut ex.seen, host) else {
                    return Ok(Value::None);
                };
                Ok(host
                    .statistic(&p, &cat, &name)
                    .map(|n| Value::Num(n as f64))
                    .unwrap_or(Value::None))
            }
        }
    }
}
