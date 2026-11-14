//! Server-wide syntax: player counts, MOTD, version, whitelist, worlds,
//! spawn points, world border and `broadcast ... to world`.
//!
//! Expressions are read by `expr_head` (called from the expression parser);
//! effects and conditions are parsed by hand here, like `world_effects`, and
//! run by the `Engine` methods at the bottom. Everything rare lives in boxed
//! variants so the hot enums in the engine do not grow.

use std::cell::Cell;

use crate::compile::{Cond, Effect};
use crate::engine::{Engine, Exec, online};
use crate::expr::{Env, Expr, Ty, parse_expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::Value;

/// What a server expression asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerQ {
    OnlineCount,
    MaxPlayers,
    Motd,
    Version,
    /// A list of world values.
    Worlds,
    /// Needs `of <world>`.
    Spawn,
    /// `of <world>` is optional.
    BorderSize,
}

impl ServerQ {
    pub fn ty(self) -> Ty {
        match self {
            ServerQ::OnlineCount | ServerQ::MaxPlayers | ServerQ::BorderSize => Ty::Number,
            ServerQ::Motd | ServerQ::Version => Ty::Text,
            ServerQ::Worlds => Ty::List,
            ServerQ::Spawn => Ty::Location,
        }
    }
}

/// Does the phrase take a world after it?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldArg {
    No,
    /// `... of <world>`, and the phrase alone is not this expression.
    Required,
    /// `... [of <world>]`
    Optional,
}

/// A recognised phrase: the query, how many tokens it used, its world argument.
#[derive(Debug, Clone, Copy)]
pub struct Head {
    pub q: ServerQ,
    pub len: usize,
    pub arg: WorldArg,
}

const PHRASES: &[(&[&str], ServerQ, WorldArg)] = &[
    (&["online", "players", "count"], ServerQ::OnlineCount, WorldArg::No),
    (&["online", "player", "count"], ServerQ::OnlineCount, WorldArg::No),
    (&["online", "players", "amount"], ServerQ::OnlineCount, WorldArg::No),
    (&["player", "count"], ServerQ::OnlineCount, WorldArg::No),
    (&["number", "of", "players"], ServerQ::OnlineCount, WorldArg::No),
    (&["number", "of", "online", "players"], ServerQ::OnlineCount, WorldArg::No),
    (&["amount", "of", "players"], ServerQ::OnlineCount, WorldArg::No),
    (&["amount", "of", "online", "players"], ServerQ::OnlineCount, WorldArg::No),
    (&["max", "players"], ServerQ::MaxPlayers, WorldArg::No),
    (&["maximum", "players"], ServerQ::MaxPlayers, WorldArg::No),
    (&["max", "player", "count"], ServerQ::MaxPlayers, WorldArg::No),
    (&["maximum", "player", "count"], ServerQ::MaxPlayers, WorldArg::No),
    (&["max", "amount", "of", "players"], ServerQ::MaxPlayers, WorldArg::No),
    (&["player", "limit"], ServerQ::MaxPlayers, WorldArg::No),
    (&["motd"], ServerQ::Motd, WorldArg::No),
    (&["server", "motd"], ServerQ::Motd, WorldArg::No),
    (&["message", "of", "the", "day"], ServerQ::Motd, WorldArg::No),
    (&["server", "version"], ServerQ::Version, WorldArg::No),
    (&["pumpkin", "version"], ServerQ::Version, WorldArg::No),
    (&["version", "of", "the", "server"], ServerQ::Version, WorldArg::No),
    (&["version", "of", "server"], ServerQ::Version, WorldArg::No),
    (&["all", "worlds"], ServerQ::Worlds, WorldArg::No),
    (&["worlds"], ServerQ::Worlds, WorldArg::No),
    (&["spawn"], ServerQ::Spawn, WorldArg::Required),
    (&["spawn", "point"], ServerQ::Spawn, WorldArg::Required),
    (&["spawn", "location"], ServerQ::Spawn, WorldArg::Required),
    (&["world", "spawn"], ServerQ::Spawn, WorldArg::Required),
    (&["world", "border", "size"], ServerQ::BorderSize, WorldArg::Optional),
    (&["world", "border", "diameter"], ServerQ::BorderSize, WorldArg::Optional),
    (&["world", "border", "width"], ServerQ::BorderSize, WorldArg::Optional),
    (&["border", "size"], ServerQ::BorderSize, WorldArg::Optional),
    (&["border", "diameter"], ServerQ::BorderSize, WorldArg::Optional),
    (&["size", "of", "the", "world", "border"], ServerQ::BorderSize, WorldArg::Optional),
    (&["size", "of", "world", "border"], ServerQ::BorderSize, WorldArg::Optional),
];

/// The phrase that starts at `toks[at]` (whose first word is `first`, lowercase).
/// Sets `hit_end` when the tokens stop in the middle of a longer phrase.
pub fn expr_head(toks: &[Tok], at: usize, first: &str, hit_end: &Cell<bool>) -> Option<Head> {
    let mut found: Option<Head> = None;
    for (words, q, arg) in PHRASES {
        if words.first().copied() != Some(first) {
            continue;
        }
        let n = words.len();
        let mut matched = 1;
        let mut ok = true;
        for (i, w) in words.iter().enumerate().skip(1) {
            match toks.get(at + i) {
                Some(t) if t.is_lit(w) => matched += 1,
                Some(_) => {
                    ok = false;
                    break;
                }
                None => {
                    // Ran out: a longer span may complete the phrase.
                    if matched >= 2 {
                        hit_end.set(true);
                    }
                    ok = false;
                    break;
                }
            }
        }
        if ok && found.is_none_or(|f| f.len < n) {
            found = Some(Head {
                q: *q,
                len: n,
                arg: *arg,
            });
        }
    }
    found
}

/// Is this a world value the host can name?
pub fn world_ty_ok(t: Ty) -> bool {
    matches!(t, Ty::World | Ty::Text | Ty::Object)
}

fn world_name(host: &mut dyn Host, v: Option<Value>) -> Option<String> {
    match v {
        Some(Value::World(n) | Value::Text(n)) => Some(n),
        Some(Value::Location { world: Some(n), .. }) => Some(n),
        Some(_) => None,
        None => host.default_world(),
    }
}

/// Evaluate a server expression. `world` is the evaluated `of <world>`.
pub fn eval(q: ServerQ, world: Option<Value>, host: &mut dyn Host) -> Value {
    match q {
        ServerQ::OnlineCount => Value::Num(host.player_count()),
        ServerQ::MaxPlayers => host.max_players().map_or(Value::None, Value::Num),
        ServerQ::Motd => host.motd().map_or(Value::None, Value::Text),
        ServerQ::Version => host.server_version().map_or(Value::None, Value::Text),
        ServerQ::Worlds => Value::List(host.world_names().into_iter().map(Value::World).collect()),
        ServerQ::Spawn => {
            let Some(w) = world_name(host, world) else {
                return Value::None;
            };
            match host.world_spawn(&w) {
                Some((x, y, z)) => Value::Location {
                    x,
                    y,
                    z,
                    world: Some(w),
                },
                None => Value::None,
            }
        }
        ServerQ::BorderSize => {
            let Some(w) = world_name(host, world) else {
                return Value::None;
            };
            host.border_size(&w).map_or(Value::None, Value::Num)
        }
    }
}

// ---------- effects and conditions ----------

#[derive(Debug, Clone, PartialEq)]
pub enum ServerFx {
    /// `add <who> to the whitelist` (true) / `remove <who> from the whitelist`.
    Whitelist { who: Expr, on: bool },
    /// `broadcast <text> to <world>`.
    BroadcastWorld { text: Expr, world: Expr },
    /// `set world border size [of <world>] to <n> [over <time>]`.
    BorderSize {
        world: Option<Expr>,
        size: Expr,
        over: Option<Expr>,
    },
    /// `teleport <who> to [the] spawn [of <world>]`.
    TeleportSpawn { who: Expr, world: Option<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerCond {
    pub who: Expr,
    pub negate: bool,
}

/// The biggest and smallest border the game accepts.
const MAX_BORDER: f64 = 59_999_968.0;
const MIN_BORDER: f64 = 1.0;
/// Longest border transition we allow, in seconds (one day).
const MAX_BORDER_SECS: f64 = 86_400.0;

fn expr_of(toks: &[Tok], env: &Env, ok: fn(Ty) -> bool) -> Option<Expr> {
    if toks.is_empty() {
        return None;
    }
    let e = parse_expr(toks, env).ok()?;
    ok(e.ty(env)).then_some(e)
}

fn who_ty(t: Ty) -> bool {
    matches!(t, Ty::Player | Ty::Text | Ty::Object)
}

fn player_ty(t: Ty) -> bool {
    matches!(t, Ty::Player | Ty::Object)
}

fn text_ty(t: Ty) -> bool {
    Ty::Text.accepts(t)
}

fn span_ty(t: Ty) -> bool {
    matches!(t, Ty::Timespan | Ty::Number | Ty::Object)
}

fn num_ty(t: Ty) -> bool {
    Ty::Number.accepts(t)
}

/// `whitelist` possibly preceded by `the` and `server`: how many tokens of
/// `rest` it uses, when `rest` is exactly that.
fn is_whitelist_tail(rest: &[Tok]) -> bool {
    let mut it = rest.iter().peekable();
    if it.peek().is_some_and(|t| t.is_lit("the")) {
        it.next();
    }
    if it.peek().is_some_and(|t| t.is_lit("server")) {
        it.next();
    }
    matches!(it.next(), Some(t) if t.is_lit("whitelist")) && it.next().is_none()
}

fn find_lit(toks: &[Tok], from: usize, lit: &str) -> Option<usize> {
    toks.iter()
        .enumerate()
        .skip(from)
        .find(|(_, t)| t.is_lit(lit))
        .map(|(i, _)| i)
}

fn rfind_any(toks: &[Tok], lits: &[&str]) -> Option<usize> {
    toks.iter()
        .enumerate()
        .rev()
        .find(|(_, t)| lits.iter().any(|l| t.is_lit(l)))
        .map(|(i, _)| i)
}

/// `Ok(None)`: not a server-misc effect line. `Err`: it is one, but wrong.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    let fx = match first.to_ascii_lowercase().as_str() {
        "add" => whitelist_move(toks, env, "to", true)?,
        "remove" => whitelist_move(toks, env, "from", false)?,
        "whitelist" => whitelist_word(toks, env, true)?,
        "unwhitelist" => whitelist_word(toks, env, false)?,
        "broadcast" => broadcast_world(toks, env),
        "set" => border_set(toks, env)?,
        "teleport" => teleport_spawn(toks, env)?,
        _ => None,
    };
    Ok(fx.map(|f| Effect::Server(Box::new(f))))
}

fn whitelist_move(toks: &[Tok], env: &Env, prep: &str, on: bool) -> Result<Option<ServerFx>, String> {
    if !toks.last().is_some_and(|t| t.is_lit("whitelist")) {
        return Ok(None);
    }
    let Some(p) = rfind_any(toks, &[prep]) else {
        return Ok(None);
    };
    if !toks.get(p + 1..).is_some_and(is_whitelist_tail) {
        return Ok(None);
    }
    let who = toks.get(1..p).and_then(|t| expr_of(t, env, who_ty));
    match who {
        Some(who) => Ok(Some(ServerFx::Whitelist { who, on })),
        None => Err("whitelist: expected a player or a name, like 'add player to the whitelist'".into()),
    }
}

fn whitelist_word(toks: &[Tok], env: &Env, on: bool) -> Result<Option<ServerFx>, String> {
    if toks.len() < 2 {
        return Ok(None);
    }
    match toks.get(1..).and_then(|t| expr_of(t, env, who_ty)) {
        Some(who) => Ok(Some(ServerFx::Whitelist { who, on })),
        None => Err("whitelist: expected a player or a name, like 'whitelist player'".into()),
    }
}

/// `broadcast <text> (in|to) <world>`. Only taken when the part after the
/// last `in`/`to` is a world; otherwise it is a plain broadcast.
fn broadcast_world(toks: &[Tok], env: &Env) -> Option<ServerFx> {
    // Cheap check first: a world expression has the word `world` or `spawn`
    // style heads; most broadcasts have no `in`/`to` at all.
    let mut i = toks.len();
    while i > 2 {
        i -= 1;
        let t = toks.get(i)?;
        if !(t.is_lit("in") || t.is_lit("to")) {
            continue;
        }
        let Some(right) = toks.get(i + 1..) else {
            continue;
        };
        let Some(world) = expr_of(right, env, |t| t == Ty::World) else {
            continue;
        };
        let Some(text) = toks.get(1..i).and_then(|t| expr_of(t, env, text_ty)) else {
            continue;
        };
        return Some(ServerFx::BroadcastWorld { text, world });
    }
    None
}

/// `set [the] [world] border (size|diameter|width) [of <world>] to <n> [over <time>]`.
fn border_set(toks: &[Tok], env: &Env) -> Result<Option<ServerFx>, String> {
    let mut i = 1;
    if toks.get(i).is_some_and(|t| t.is_lit("the")) {
        i += 1;
    }
    if toks.get(i).is_some_and(|t| t.is_lit("world")) {
        i += 1;
    }
    if !toks.get(i).is_some_and(|t| t.is_lit("border")) {
        return Ok(None);
    }
    i += 1;
    if !toks
        .get(i)
        .is_some_and(|t| t.is_lit("size") || t.is_lit("diameter") || t.is_lit("width"))
    {
        return Ok(None);
    }
    i += 1;
    let usage = "set world border size [of <world>] to <number> [over <timespan>]";
    let world = if toks.get(i).is_some_and(|t| t.is_lit("of")) {
        let to = find_lit(toks, i + 1, "to").ok_or_else(|| format!("expected '{usage}'"))?;
        let w = toks
            .get(i + 1..to)
            .and_then(|t| expr_of(t, env, world_ty_ok))
            .ok_or_else(|| format!("expected a world after 'of' ({usage})"))?;
        i = to;
        Some(w)
    } else {
        None
    };
    if !toks.get(i).is_some_and(|t| t.is_lit("to")) {
        return Err(format!("expected '{usage}'"));
    }
    let rest = toks.get(i + 1..).unwrap_or_default();
    let (size_toks, over) = match rfind_any(rest, &["over", "in", "during"]) {
        Some(j) if j > 0 => {
            let t = rest.get(j + 1..).unwrap_or_default();
            let over = expr_of(t, env, span_ty).ok_or_else(|| format!("expected a timespan after 'over' ({usage})"))?;
            (rest.get(..j).unwrap_or_default(), Some(over))
        }
        _ => (rest, None),
    };
    let size = expr_of(size_toks, env, num_ty).ok_or_else(|| format!("expected a number for the size ({usage})"))?;
    Ok(Some(ServerFx::BorderSize { world, size, over }))
}

/// `teleport <who> to [the] [world] spawn [point] [(of|in) <world>]`.
fn teleport_spawn(toks: &[Tok], env: &Env) -> Result<Option<ServerFx>, String> {
    // The last `to` that is followed by (the) (world) spawn.
    let mut at = None;
    for (i, t) in toks.iter().enumerate().skip(2) {
        if !t.is_lit("to") {
            continue;
        }
        let mut j = i + 1;
        if toks.get(j).is_some_and(|t| t.is_lit("the")) {
            j += 1;
        }
        if toks.get(j).is_some_and(|t| t.is_lit("world")) {
            j += 1;
        }
        if toks.get(j).is_some_and(|t| t.is_lit("spawn")) {
            at = Some((i, j + 1));
        }
    }
    let Some((to, mut j)) = at else {
        return Ok(None);
    };
    if toks.get(j).is_some_and(|t| t.is_lit("point") || t.is_lit("location")) {
        j += 1;
    }
    let world = match toks.get(j) {
        None => None,
        Some(t) if t.is_lit("of") || t.is_lit("in") => {
            let w = toks
                .get(j + 1..)
                .and_then(|t| expr_of(t, env, world_ty_ok))
                .ok_or("expected a world after 'spawn of'")?;
            Some(w)
        }
        Some(_) => return Ok(None),
    };
    let Some(who) = toks.get(1..to).and_then(|t| expr_of(t, env, player_ty)) else {
        return Ok(None);
    };
    Ok(Some(ServerFx::TeleportSpawn { who, world }))
}

/// `<who> is [not] whitelisted`, `<who> is [not] on the whitelist`.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    let last = toks.last()?;
    let end = if last.is_lit("whitelisted") {
        toks.len() - 1
    } else if last.is_lit("whitelist") {
        // `... on [the] [server] whitelist`
        let on = rfind_any(toks, &["on", "in"])?;
        if !toks.get(on + 1..).is_some_and(is_whitelist_tail) {
            return None;
        }
        on
    } else {
        return None;
    };
    let mut head = toks.get(..end)?;
    let mut negate = false;
    if let Some((n, rest)) = head.split_last()
        && n.is_lit("not")
    {
        negate = true;
        head = rest;
    }
    let (verb, subject) = head.split_last()?;
    if verb.is_lit("isn't") || verb.is_lit("aren't") {
        negate = !negate;
    } else if !(verb.is_lit("is") || verb.is_lit("are")) {
        return None;
    }
    cond_of(subject, env, negate)
}

fn cond_of(subject: &[Tok], env: &Env, negate: bool) -> Option<Cond> {
    let who = expr_of(subject, env, who_ty)?;
    Some(Cond::Server(Box::new(ServerCond { who, negate })))
}

// ---------- running ----------

impl Engine {
    /// A player or a name from a value: the name, and the uuid when online.
    fn server_who(&mut self, v: &Value, ex: &mut Exec, host: &mut dyn Host) -> Option<(String, Option<String>)> {
        if let Some(p) = online(v, &mut ex.seen, host) {
            return Some((p.name, Some(p.uuid)));
        }
        match v {
            Value::Player(p) => Some((p.name.clone(), None)),
            Value::Text(t) if !t.is_empty() => Some((t.clone(), None)),
            _ => None,
        }
    }

    pub(crate) fn server_effect(&mut self, f: &ServerFx, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        match f {
            ServerFx::Whitelist { who, on } => {
                let v = self.eval(who, ex, host)?;
                if let Some((name, uuid)) = self.server_who(&v, ex, host) {
                    host.set_whitelisted(&name, uuid.as_deref(), *on);
                }
            }
            ServerFx::BroadcastWorld { text, world } => {
                let msg = self.eval(text, ex, host)?.to_string();
                let w = self.eval(world, ex, host)?;
                if let Some(w) = world_name(host, Some(w)) {
                    host.broadcast_world(&w, &msg);
                }
            }
            ServerFx::BorderSize { world, size, over } => {
                let size = match self.eval(size, ex, host)? {
                    Value::Num(n) if n.is_finite() => n.clamp(MIN_BORDER, MAX_BORDER),
                    other => return Err(format!("world border size needs a number, got {}", other.type_name())),
                };
                let secs = match over {
                    Some(e) => match self.eval(e, ex, host)? {
                        Value::Timespan { millis } => (millis as f64 / 1000.0).clamp(0.0, MAX_BORDER_SECS),
                        Value::Num(n) if n.is_finite() => n.clamp(0.0, MAX_BORDER_SECS),
                        other => return Err(format!("'over' needs a timespan, got {}", other.type_name())),
                    },
                    None => 0.0,
                };
                let w = match world {
                    Some(e) => {
                        let v = self.eval(e, ex, host)?;
                        world_name(host, Some(v))
                    }
                    None => host.default_world(),
                };
                if let Some(w) = w {
                    host.set_border_size(&w, size, secs);
                }
            }
            ServerFx::TeleportSpawn { who, world } => {
                let v = self.eval(who, ex, host)?;
                let Some(p) = online(&v, &mut ex.seen, host) else {
                    return Ok(());
                };
                let w = match world {
                    Some(e) => {
                        let v = self.eval(e, ex, host)?;
                        world_name(host, Some(v))
                    }
                    None => host
                        .player_location(&p)
                        .map(|(_, _, _, w)| w)
                        .or_else(|| host.default_world()),
                };
                if let Some(w) = w
                    && let Some((x, y, z)) = host.world_spawn(&w)
                {
                    host.teleport_to(&p, x, y, z, Some(&w));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn server_cond(&mut self, c: &ServerCond, ex: &mut Exec, host: &mut dyn Host) -> Result<bool, String> {
        let v = self.eval(&c.who, ex, host)?;
        let listed = match self.server_who(&v, ex, host) {
            Some((name, _)) => host.is_whitelisted(&name),
            None => false,
        };
        Ok(listed != c.negate)
    }

    /// Evaluate an `Expr::Server`.
    pub(crate) fn server_expr(
        &mut self,
        q: ServerQ,
        world: &Option<Box<Expr>>,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        let w = match world {
            Some(e) => Some(self.eval(e, ex, host)?),
            None => None,
        };
        Ok(eval(q, w, host))
    }
}
