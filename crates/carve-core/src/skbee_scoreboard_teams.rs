//! SkBee-style scoreboards: objectives, scores, teams and per-player sidebar
//! boards (SkBee's "fastboard").
//!
//! Carve keeps its own copy of the objectives, scores and teams in the global
//! variables (under keys that start with a control character, so scripts cannot
//! reach them by accident) and applies every change to the server scoreboard
//! with [`Host::sb_edit`]: the plugin uses the scoreboard calls of the plugin
//! API, and a host without them runs the `/scoreboard` and `/team` commands
//! ([`SbEdit::command`]). Reads come from the copy. Because the copy is saved
//! with the global variables, it survives restarts, like the server's own scoreboard.
//!
//! Boards are per player and not saved. The host draws them (`Host::board_show`).
//!
//! The lines here have free-form words in them, so they are parsed by hand:
//! `parse_effect` and `parse_cond` run before the pattern registry, and
//! `parse_expr_at` is called from the expression parser.

use std::collections::BTreeMap;

use crate::compile::{Cond, Effect};
use crate::engine::{Engine, Exec, online};
use crate::expr::{Env, Expr, Part, Ty, parse_expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::{PlayerRef, Value};
use crate::vars::Variables;

/// Most lines a sidebar can show.
pub const MAX_BOARD_LINES: usize = 15;

// ---------- syntax tree ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreOp {
    Set,
    Add,
    Remove,
}

/// A team setting that `set ... of team X to Y` can change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamProp {
    Prefix,
    Suffix,
    DisplayName,
    Color,
    FriendlyFire,
    SeeInvisibles,
    NametagVisibility,
    CollisionRule,
    DeathMessageVisibility,
}

impl TeamProp {
    fn from_words(w: &[&str]) -> Option<TeamProp> {
        Some(match w {
            ["prefix"] => TeamProp::Prefix,
            ["suffix"] => TeamProp::Suffix,
            ["display", "name"] => TeamProp::DisplayName,
            ["color"] | ["colour"] => TeamProp::Color,
            ["friendly", "fire"] => TeamProp::FriendlyFire,
            ["see", "friendly", "invisibles"] => TeamProp::SeeInvisibles,
            ["nametag", "visibility"] | ["name", "tag", "visibility"] => TeamProp::NametagVisibility,
            ["collision", "rule"] => TeamProp::CollisionRule,
            ["death", "message", "visibility"] => TeamProp::DeathMessageVisibility,
            _ => return None,
        })
    }

    fn is_bool(self) -> bool {
        matches!(self, TeamProp::FriendlyFire | TeamProp::SeeInvisibles)
    }

    /// Key in the stored copy.
    fn key(self) -> &'static str {
        match self {
            TeamProp::Prefix => "prefix",
            TeamProp::Suffix => "suffix",
            TeamProp::DisplayName => "display",
            TeamProp::Color => "color",
            TeamProp::FriendlyFire => "friendlyfire",
            TeamProp::SeeInvisibles => "seeinvisibles",
            TeamProp::NametagVisibility => "nametag",
            TeamProp::CollisionRule => "collision",
            TeamProp::DeathMessageVisibility => "deathmessage",
        }
    }
}

/// One change to the server scoreboard or its teams, already checked. A host
/// with a direct API applies it; the default turns it into a vanilla command
/// ([`SbEdit::command`]).
#[derive(Debug, Clone, PartialEq)]
pub enum SbEdit {
    AddObjective {
        name: String,
        criteria: String,
        display: Option<String>,
    },
    RemoveObjective(String),
    SetScore {
        holder: String,
        objective: String,
        value: i32,
    },
    /// One objective, or all of them.
    ResetScore {
        holder: String,
        objective: Option<String>,
    },
    AddTeam(String),
    RemoveTeam(String),
    EmptyTeam(String),
    Join {
        team: String,
        holder: String,
    },
    Leave {
        holder: String,
    },
    /// `arg` is the plain text (prefix, suffix, display name), the colour
    /// (`red`, `reset`), `true` / `false`, or the vanilla word for the rule.
    TeamModify {
        team: String,
        prop: TeamProp,
        arg: String,
    },
}

impl SbEdit {
    /// The vanilla command that does the same.
    pub fn command(&self) -> String {
        match self {
            SbEdit::AddObjective {
                name,
                criteria,
                display,
            } => match display {
                Some(d) => format!("scoreboard objectives add {name} {criteria} {}", json_text(d)),
                None => format!("scoreboard objectives add {name} {criteria}"),
            },
            SbEdit::RemoveObjective(name) => format!("scoreboard objectives remove {name}"),
            SbEdit::SetScore {
                holder,
                objective,
                value,
            } => format!("scoreboard players set {holder} {objective} {value}"),
            SbEdit::ResetScore {
                holder,
                objective: Some(o),
            } => format!("scoreboard players reset {holder} {o}"),
            SbEdit::ResetScore {
                holder,
                objective: None,
            } => format!("scoreboard players reset {holder}"),
            SbEdit::AddTeam(t) => format!("team add {t}"),
            SbEdit::RemoveTeam(t) => format!("team remove {t}"),
            SbEdit::EmptyTeam(t) => format!("team empty {t}"),
            SbEdit::Join { team, holder } => format!("team join {team} {holder}"),
            SbEdit::Leave { holder } => format!("team leave {holder}"),
            SbEdit::TeamModify { team, prop, arg } => {
                let rest = match prop {
                    TeamProp::Prefix => format!("prefix {}", json_text(arg)),
                    TeamProp::Suffix => format!("suffix {}", json_text(arg)),
                    TeamProp::DisplayName => format!("displayName {}", json_text(arg)),
                    TeamProp::Color => format!("color {arg}"),
                    TeamProp::FriendlyFire => format!("friendlyFire {arg}"),
                    TeamProp::SeeInvisibles => format!("seeFriendlyInvisibles {arg}"),
                    TeamProp::NametagVisibility => format!("nametagVisibility {arg}"),
                    TeamProp::DeathMessageVisibility => format!("deathMessageVisibility {arg}"),
                    TeamProp::CollisionRule => format!("collisionRule {arg}"),
                };
                format!("team modify {team} {rest}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SbFx {
    RegisterObjective {
        name: Expr,
        criteria: Option<Expr>,
        display: Option<Expr>,
    },
    UnregisterObjective(Expr),
    Score {
        op: ScoreOp,
        holder: Expr,
        objective: Expr,
        value: Expr,
    },
    /// One objective, or all of them.
    ResetScore {
        holder: Expr,
        objective: Option<Expr>,
    },
    RegisterTeam(Expr),
    UnregisterTeam(Expr),
    ClearTeam(Expr),
    Join {
        who: Expr,
        team: Expr,
    },
    /// Leave the named team, or whatever team they are in.
    Leave {
        who: Expr,
        team: Option<Expr>,
    },
    TeamSet {
        team: Expr,
        prop: TeamProp,
        value: Expr,
    },
    BoardTitle {
        player: Expr,
        text: Expr,
    },
    BoardLine {
        player: Expr,
        line: Expr,
        /// `None` removes the line.
        text: Option<Expr>,
    },
    BoardLines {
        player: Expr,
        lines: Vec<Expr>,
    },
    BoardClear(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SbCond {
    ObjectiveExists(Expr),
    TeamExists(Expr),
    InTeam { who: Expr, team: Expr },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SbExpr {
    Score { holder: Expr, objective: Expr },
    TeamOf(Expr),
    Team(TeamField, Expr),
    ObjectiveCriteria(Expr),
    ObjectiveDisplay(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamField {
    Prop(TeamProp),
    Members,
    Size,
}

impl SbExpr {
    pub fn ty(&self) -> Ty {
        match self {
            SbExpr::Score { .. } | SbExpr::Team(TeamField::Size, _) => Ty::Number,
            SbExpr::Team(TeamField::Members, _) => Ty::List,
            SbExpr::Team(TeamField::Prop(p), _) if p.is_bool() => Ty::Boolean,
            _ => Ty::Text,
        }
    }
}

// ---------- parsing ----------

fn is(t: &Tok, lit: &str) -> bool {
    t.is_lit(lit)
}

fn word_at(toks: &[Tok], i: usize) -> Option<String> {
    match toks.get(i) {
        Some(Tok::Word(w)) => Some(w.to_ascii_lowercase()),
        _ => None,
    }
}

fn find(toks: &[Tok], from: usize, lit: &str) -> Option<usize> {
    toks.iter()
        .enumerate()
        .skip(from)
        .find(|(_, t)| is(t, lit))
        .map(|(i, _)| i)
}

fn find_seq(toks: &[Tok], from: usize, seq: &[&str]) -> Option<usize> {
    (from..toks.len()).find(|&i| {
        seq.iter()
            .enumerate()
            .all(|(k, l)| toks.get(i + k).is_some_and(|t| is(t, l)))
    })
}

fn expr_of(toks: &[Tok], env: &Env, want: &[Ty]) -> Option<Expr> {
    let e = parse_expr(toks, env).ok()?;
    let got = e.ty(env);
    (got == Ty::Object || want.contains(&got)).then_some(e)
}

fn text(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Text])
}

fn number(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Number])
}

/// A score holder or team member: a player or a name.
fn holder(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Player, Ty::Text])
}

/// A player, or the name of one.
fn player(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Player, Ty::Text])
}

fn lit_text(s: &str) -> Expr {
    Expr::Text(vec![Part::Lit(s.to_string())])
}

/// Skip a leading `the` / `a` / `an` and an optional `new`.
fn skip_articles(toks: &[Tok], mut i: usize, allow_new: bool) -> usize {
    if toks
        .get(i)
        .is_some_and(|t| is(t, "the") || is(t, "a") || is(t, "an"))
    {
        i += 1;
    }
    if allow_new && toks.get(i).is_some_and(|t| is(t, "new")) {
        i += 1;
    }
    i
}

/// Parse a line as a scoreboard effect. `Ok(None)` means it is not one.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    // Cheap exit: every form here has one of these words.
    if !toks
        .iter()
        .any(|t| is(t, "objective") || is(t, "team") || is(t, "score") || is(t, "scores") || is(t, "board"))
    {
        return Ok(None);
    }
    let Some(first) = word_at(toks, 0) else {
        return Ok(None);
    };
    let fx = match first.as_str() {
        "register" | "create" => register(toks, env)?,
        "remove" => match add_remove(toks, env, false)? {
            Some(f) => Some(f),
            None => unset(toks, env)?,
        },
        "unregister" | "delete" | "clear" | "empty" | "reset" | "hide" => unset(toks, env)?,
        "set" => set(toks, env)?,
        "add" | "subtract" => add_remove(toks, env, first == "add")?,
        _ => None,
    };
    Ok(fx.map(|f| Effect::Sbt(Box::new(f))))
}

/// `register [a] [new] objective "x" [with criteria "dummy"] [named "Shown"]`,
/// `register [a] [new] team "x"`.
fn register(toks: &[Tok], env: &Env) -> Result<Option<SbFx>, String> {
    let i = skip_articles(toks, 1, true);
    if toks.get(i).is_some_and(|t| is(t, "team")) {
        let rest = toks.get(i + 1..).unwrap_or_default();
        let rest = strip_named(rest);
        return Ok(text(rest, env).map(SbFx::RegisterTeam));
    }
    if !toks.get(i).is_some_and(|t| is(t, "objective")) {
        return Ok(None);
    }
    let rest = toks.get(i + 1..).unwrap_or_default();
    let rest = strip_named(rest);
    // Cut at the optional parts, which can come in either order.
    let markers: [(&[&str], usize); 3] = [
        (&["with", "criteria"], 2),
        (&["named"], 1),
        (&["with", "display", "name"], 3),
    ];
    let mut cuts: Vec<(usize, usize, usize)> = Vec::new();
    for (k, (seq, len)) in markers.iter().enumerate() {
        if let Some(p) = find_seq(rest, 0, seq) {
            cuts.push((p, *len, k));
        }
    }
    cuts.sort_unstable();
    let name_end = cuts.first().map_or(rest.len(), |c| c.0);
    let Some(name) = rest.get(..name_end).and_then(|t| text(t, env)) else {
        return Ok(None);
    };
    let (mut criteria, mut display) = (None, None);
    for (n, (start, len, k)) in cuts.iter().enumerate() {
        let end = cuts.get(n + 1).map_or(rest.len(), |c| c.0);
        let Some(e) = rest.get(start + len..end).and_then(|t| text(t, env)) else {
            return Ok(None);
        };
        if *k == 0 {
            criteria = Some(e);
        } else {
            display = Some(e);
        }
    }
    Ok(Some(SbFx::RegisterObjective {
        name,
        criteria,
        display,
    }))
}

/// `named "x"` after the noun: drop the `named`.
fn strip_named(rest: &[Tok]) -> &[Tok] {
    match rest.first() {
        Some(t) if is(t, "named") => rest.get(1..).unwrap_or_default(),
        _ => rest,
    }
}

/// `unregister objective x`, `delete team x`, `clear team x`, `reset score of ...`,
/// `delete board of p`, `remove p from team x`, `delete line 2 of board of p`.
fn unset(toks: &[Tok], env: &Env) -> Result<Option<SbFx>, String> {
    let verb = word_at(toks, 0).unwrap_or_default();
    let i = skip_articles(toks, 1, false);
    let rest = toks.get(i + 1..).unwrap_or_default();
    match word_at(toks, i).as_deref() {
        Some("objective") if matches!(verb.as_str(), "unregister" | "delete" | "remove") => {
            Ok(text(rest, env).map(SbFx::UnregisterObjective))
        }
        Some("team") if matches!(verb.as_str(), "unregister" | "delete") => {
            Ok(text(rest, env).map(SbFx::UnregisterTeam))
        }
        Some("team") if matches!(verb.as_str(), "clear" | "empty") => {
            Ok(text(rest, env).map(SbFx::ClearTeam))
        }
        Some("team")
            if matches!(verb.as_str(), "reset" | "remove") && rest.first().is_some_and(|t| is(t, "of")) =>
        {
            // `reset team of <who>`
            Ok(holder(rest.get(1..).unwrap_or_default(), env).map(|who| SbFx::Leave { who, team: None }))
        }
        Some("score" | "scores") if verb == "reset" || verb == "clear" => reset_score(rest, env),
        Some("all")
            if (verb == "reset" || verb == "clear")
                && word_at(toks, i + 1).is_some_and(|w| w == "scores") =>
        {
            reset_score(toks.get(i + 2..).unwrap_or_default(), env)
        }
        Some("board") if matches!(verb.as_str(), "delete" | "remove" | "clear" | "reset" | "hide") => {
            Ok(board_owner(rest, env).map(SbFx::BoardClear))
        }
        Some("line") if matches!(verb.as_str(), "delete" | "remove" | "clear" | "reset") => {
            let Some(of) = find(toks, i, "of") else {
                return Ok(None);
            };
            let (Some(line), Some(p)) = (
                number(toks.get(i + 1..of).unwrap_or_default(), env),
                board_ref(toks.get(of + 1..).unwrap_or_default(), env),
            ) else {
                return Ok(None);
            };
            Ok(Some(SbFx::BoardLine {
                player: p,
                line,
                text: None,
            }))
        }
        _ => {
            // `remove <who> from team <team>`
            if verb == "remove"
                && let Some(f) = find(toks, 1, "from")
                && toks.get(f + 1).is_some_and(|t| is(t, "team"))
            {
                let (Some(who), Some(team)) = (
                    holder(toks.get(1..f).unwrap_or_default(), env),
                    text(toks.get(f + 2..).unwrap_or_default(), env),
                ) else {
                    return Ok(None);
                };
                return Ok(Some(SbFx::Leave {
                    who,
                    team: Some(team),
                }));
            }
            Ok(None)
        }
    }
}

/// After `reset [the] score(s)`: `of <who> [in [objective] <obj>]`.
fn reset_score(rest: &[Tok], env: &Env) -> Result<Option<SbFx>, String> {
    if !rest.first().is_some_and(|t| is(t, "of")) {
        return Ok(None);
    }
    let rest = rest.get(1..).unwrap_or_default();
    match find(rest, 0, "in") {
        Some(p) => {
            let obj = objective_tail(rest.get(p + 1..).unwrap_or_default(), env);
            let who = holder(rest.get(..p).unwrap_or_default(), env);
            Ok(who.zip(obj).map(|(holder, o)| SbFx::ResetScore {
                holder,
                objective: Some(o),
            }))
        }
        None => Ok(holder(rest, env).map(|holder| SbFx::ResetScore {
            holder,
            objective: None,
        })),
    }
}

/// `[objective] <text>`
fn objective_tail(toks: &[Tok], env: &Env) -> Option<Expr> {
    let toks = match toks.first() {
        Some(t) if is(t, "objective") => toks.get(1..)?,
        _ => toks,
    };
    text(toks, env)
}

/// `score of <who> in [objective] <obj>` as a tail (after the word `score`).
/// Returns holder and objective.
fn score_ref(toks: &[Tok], env: &Env) -> Option<(Expr, Expr)> {
    let rest = match toks.first() {
        Some(t) if is(t, "of") => toks.get(1..)?,
        _ => return None,
    };
    let p = find(rest, 0, "in")?;
    let who = holder(rest.get(..p)?, env)?;
    let obj = objective_tail(rest.get(p + 1..)?, env)?;
    Some((who, obj))
}

/// `[the] board of <p>` or `<p>'s board`: returns the player expression.
fn board_ref(toks: &[Tok], env: &Env) -> Option<Expr> {
    let i = skip_articles(toks, 0, false);
    let i = if toks.get(i).is_some_and(|t| is(t, "board")) {
        i
    } else {
        return poss_board(toks, env);
    };
    board_owner(toks.get(i + 1..)?, env)
}

/// After the word `board`: `of <p>`.
fn board_owner(rest: &[Tok], env: &Env) -> Option<Expr> {
    let rest = match rest.first() {
        Some(t) if is(t, "of") => rest.get(1..)?,
        _ => return None,
    };
    player(rest, env)
}

/// `<p>'s board`
fn poss_board(toks: &[Tok], env: &Env) -> Option<Expr> {
    let n = toks.len();
    if n < 3 || !is(toks.get(n - 1)?, "board") || !is(toks.get(n - 2)?, "'s") {
        return None;
    }
    player(toks.get(..n - 2)?, env)
}

fn set(toks: &[Tok], env: &Env) -> Result<Option<SbFx>, String> {
    let i = skip_articles(toks, 1, false);
    // `set [the] score of <who> in [objective] <obj> to <n>`
    if word_at(toks, i).as_deref() == Some("score") {
        let Some(to) = toks.iter().rposition(|t| is(t, "to")) else {
            return Ok(None);
        };
        let Some((holder, objective)) = score_ref(toks.get(i + 1..to).unwrap_or_default(), env) else {
            return Ok(None);
        };
        let Some(value) = number(toks.get(to + 1..).unwrap_or_default(), env) else {
            return Ok(None);
        };
        return Ok(Some(SbFx::Score {
            op: ScoreOp::Set,
            holder,
            objective,
            value,
        }));
    }
    // `set [the] [team] <prop> of [team] <team> to <value>`
    if let Some(fx) = set_team(toks, i, env)? {
        return Ok(Some(fx));
    }
    set_board(toks, i, env)
}

fn set_team(toks: &[Tok], mut i: usize, env: &Env) -> Result<Option<SbFx>, String> {
    let Some(of) = find(toks, i, "of") else {
        return Ok(None);
    };
    let had_team = toks.get(i).is_some_and(|t| is(t, "team"));
    if had_team {
        i += 1;
    }
    let words: Option<Vec<String>> = (i..of).map(|k| word_at(toks, k)).collect();
    let Some(words) = words else {
        return Ok(None);
    };
    let w: Vec<&str> = words.iter().map(String::as_str).collect();
    let Some(prop) = TeamProp::from_words(&w) else {
        return Ok(None);
    };
    // Need the word `team` in front of the property or right after `of`.
    let mut t0 = of + 1;
    let named = toks.get(t0).is_some_and(|t| is(t, "team"));
    if named {
        t0 += 1;
    }
    if !had_team && !named {
        return Ok(None);
    }
    let Some(to) = find(toks, t0, "to") else {
        return Ok(None);
    };
    let Some(team) = text(toks.get(t0..to).unwrap_or_default(), env) else {
        return Ok(None);
    };
    let vt = toks.get(to + 1..).unwrap_or_default();
    let value = match prop {
        TeamProp::FriendlyFire | TeamProp::SeeInvisibles => expr_of(vt, env, &[Ty::Boolean]),
        TeamProp::Color
        | TeamProp::NametagVisibility
        | TeamProp::CollisionRule
        | TeamProp::DeathMessageVisibility => {
            // Bare words (`dark red`, `hide for other teams`) are a literal.
            let bare: Option<Vec<String>> = vt
                .iter()
                .map(|t| match t {
                    Tok::Word(w) => Some(w.to_ascii_lowercase()),
                    _ => None,
                })
                .collect();
            match bare {
                Some(b) if !b.is_empty() => Some(lit_text(&b.join("_"))),
                _ => text(vt, env),
            }
        }
        _ => text(vt, env),
    };
    let Some(value) = value else {
        return Ok(None);
    };
    Ok(Some(SbFx::TeamSet { team, prop, value }))
}

fn set_board(toks: &[Tok], i: usize, env: &Env) -> Result<Option<SbFx>, String> {
    let Some(of) = find(toks, i, "of") else {
        return Ok(None);
    };
    let Some(to) = toks.iter().rposition(|t| is(t, "to")) else {
        return Ok(None);
    };
    if to < of {
        return Ok(None);
    }
    let what = toks.get(i..of).unwrap_or_default();
    let words: Vec<String> = what
        .iter()
        .filter_map(|t| match t {
            Tok::Word(w) => Some(w.to_ascii_lowercase()),
            _ => None,
        })
        .collect();
    let owner_toks = toks.get(of + 1..to).unwrap_or_default();
    let value_toks = toks.get(to + 1..).unwrap_or_default();
    let w: Vec<&str> = words.iter().map(String::as_str).collect();
    match w.as_slice() {
        ["title"] | ["name"] if what.len() == 1 => {
            let Some(p) = board_ref(owner_toks, env) else {
                return Ok(None);
            };
            Ok(text(value_toks, env).map(|text| SbFx::BoardTitle { player: p, text }))
        }
        ["lines"] | ["all", "lines"] if what.len() == w.len() => {
            let Some(p) = board_ref(owner_toks, env) else {
                return Ok(None);
            };
            let mut lines = Vec::new();
            for part in value_toks.split(|t| is(t, ",") || is(t, "and")) {
                match text(part, env) {
                    Some(e) => lines.push(e),
                    None => return Ok(None),
                }
            }
            Ok(Some(SbFx::BoardLines { player: p, lines }))
        }
        ["line", ..] if what.len() >= 2 => {
            let Some(p) = board_ref(owner_toks, env) else {
                return Ok(None);
            };
            // what = `line <n>`
            let (Some(line), Some(t)) = (
                number(what.get(1..).unwrap_or_default(), env),
                text(value_toks, env),
            ) else {
                return Ok(None);
            };
            Ok(Some(SbFx::BoardLine {
                player: p,
                line,
                text: Some(t),
            }))
        }
        _ => Ok(None),
    }
}

/// `add <n> to [the] score of ...`, `add <who> to team <t>`,
/// `remove/subtract <n> from [the] score of ...`.
fn add_remove(toks: &[Tok], env: &Env, add: bool) -> Result<Option<SbFx>, String> {
    let sep = if add { "to" } else { "from" };
    let Some(s) = find(toks, 1, sep) else {
        return Ok(None);
    };
    let i = skip_articles(toks, s + 1, false);
    match word_at(toks, i).as_deref() {
        Some("score") => {
            let Some(value) = number(toks.get(1..s).unwrap_or_default(), env) else {
                return Ok(None);
            };
            let Some((holder, objective)) = score_ref(toks.get(i + 1..).unwrap_or_default(), env) else {
                return Ok(None);
            };
            let op = if add { ScoreOp::Add } else { ScoreOp::Remove };
            Ok(Some(SbFx::Score {
                op,
                holder,
                objective,
                value,
            }))
        }
        Some("team") if add => {
            let (Some(who), Some(team)) = (
                holder(toks.get(1..s).unwrap_or_default(), env),
                text(toks.get(i + 1..).unwrap_or_default(), env),
            ) else {
                return Ok(None);
            };
            Ok(Some(SbFx::Join { who, team }))
        }
        _ => Ok(None),
    }
}

/// Conditions: `objective x exists`, `team x exists`, `<who> is in team x`.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    if !toks.iter().any(|t| is(t, "team") || is(t, "objective")) {
        return None;
    }
    let first = word_at(toks, 0)?;
    let n = toks.len();
    if first == "objective" || first == "team" {
        let last = toks.get(n.checked_sub(1)?)?;
        let (end, neg) = if is(last, "exist") && n >= 3 && is(toks.get(n - 2)?, "doesn't") {
            (n - 2, true)
        } else if is(last, "exist") && n >= 4 && is(toks.get(n - 2)?, "not") && is(toks.get(n - 3)?, "does") {
            (n - 3, true)
        } else if is(last, "exists") || is(last, "exist") {
            (n - 1, false)
        } else {
            return None;
        };
        let e = text(toks.get(1..end)?, env)?;
        let c = if first == "objective" {
            SbCond::ObjectiveExists(e)
        } else {
            SbCond::TeamExists(e)
        };
        return Some(Cond::Sb(Box::new(c), neg));
    }
    // `<who> is [not] in team <t>` / `<who> isn't in team <t>`
    let isat = toks.iter().position(|t| is(t, "is") || is(t, "isn't"))?;
    let mut k = isat + 1;
    let mut neg = is(toks.get(isat)?, "isn't");
    if toks.get(k).is_some_and(|t| is(t, "not")) {
        neg = true;
        k += 1;
    }
    if !(toks.get(k).is_some_and(|t| is(t, "in")) && toks.get(k + 1).is_some_and(|t| is(t, "team"))) {
        return None;
    }
    let who = holder(toks.get(..isat)?, env)?;
    let team = text(toks.get(k + 2..)?, env)?;
    Some(Cond::Sb(Box::new(SbCond::InTeam { who, team }), neg))
}

/// Words that can start a scoreboard expression. Checked first so other
/// words cost nothing.
pub fn is_head(lw: &str) -> bool {
    matches!(
        lw,
        "score"
            | "team"
            | "prefix"
            | "suffix"
            | "color"
            | "colour"
            | "members"
            | "entries"
            | "objective"
            | "criteria"
    )
}

/// Does this start like `members of team x`? Used so `loop` and `set {x::*} to`
/// treat it as a list.
pub fn is_list_expr(toks: &[Tok]) -> bool {
    let is_members = |t: Option<&Tok>| t.is_some_and(|t| is(t, "members") || is(t, "entries"));
    (is_members(toks.first()) && toks.get(1).is_some_and(|t| is(t, "of")))
        || (toks.first().is_some_and(|t| is(t, "team")) && is_members(toks.get(1)))
}

/// What an expression parser must offer this module.
pub trait Operands {
    fn peek_tok(&self, n: usize) -> Option<&Tok>;
    fn advance(&mut self, n: usize);
    /// The token `n` ahead. Asking past the end tells the matcher that more
    /// words might complete the expression.
    /// One value (a primary expression).
    fn operand(&mut self) -> Result<Expr, String>;
}

/// Parse a scoreboard expression whose first word (`lw`) was just read.
/// `None` when it is not one (nothing is consumed then).
pub fn parse_expr_at<P: Operands>(p: &mut P, lw: &str) -> Option<Result<Expr, String>> {
    let wrap = |e: SbExpr| Expr::Sb(Box::new(e));
    let lit = |p: &P, n: usize, w: &str| p.peek_tok(n).is_some_and(|t| is(t, w));
    match lw {
        "score" if lit(p, 0, "of") => {
            p.advance(1);
            let r = (|| {
                let holder = p.operand()?;
                if !p.peek_tok(0).is_some_and(|t| is(t, "in")) {
                    return Err("expected 'in <objective>', like 'score of player in \"kills\"'".to_string());
                }
                p.advance(1);
                if p.peek_tok(0).is_some_and(|t| is(t, "objective")) {
                    p.advance(1);
                }
                let objective = p.operand()?;
                Ok(wrap(SbExpr::Score { holder, objective }))
            })();
            Some(r)
        }
        "criteria" | "objective" => {
            // `objective criteria of <o>` / `criteria of objective <o>` /
            // `objective display name of <o>`
            let display = lw == "objective" && lit(p, 0, "display") && lit(p, 1, "name") && lit(p, 2, "of");
            let crit_a = lw == "objective" && lit(p, 0, "criteria") && lit(p, 1, "of");
            let crit_b = lw == "criteria" && lit(p, 0, "of") && lit(p, 1, "objective");
            if display || crit_a || crit_b {
                let skip = if display {
                    3
                } else if crit_a {
                    2
                } else {
                    1
                };
                p.advance(skip);
                if crit_b {
                    p.advance(1);
                }
                let r = p.operand().map(|o| {
                    wrap(if display {
                        SbExpr::ObjectiveDisplay(o)
                    } else {
                        SbExpr::ObjectiveCriteria(o)
                    })
                });
                return Some(r);
            }
            None
        }
        "team" | "prefix" | "suffix" | "color" | "colour" | "members" | "entries" => team_expr(p, lw, wrap),
        _ => None,
    }
}

fn team_expr<P: Operands>(
    p: &mut P,
    lw: &str,
    wrap: impl Fn(SbExpr) -> Expr,
) -> Option<Result<Expr, String>> {
    // `team of <who>`
    if lw == "team" && p.peek_tok(0).is_some_and(|t| is(t, "of")) {
        p.advance(1);
        return Some(p.operand().map(|e| wrap(SbExpr::TeamOf(e))));
    }
    // Collect the property words up to `of`.
    let mut words: Vec<String> = Vec::new();
    let mut n = 0;
    if lw != "team" {
        words.push(lw.to_string());
    }
    loop {
        match p.peek_tok(n) {
            Some(t) if is(t, "of") => break,
            Some(Tok::Word(w)) if words.len() < 4 => {
                words.push(w.to_ascii_lowercase());
                n += 1;
            }
            _ => return None,
        }
    }
    let w: Vec<&str> = words.iter().map(String::as_str).collect();
    let field = match w.as_slice() {
        ["members"] | ["entries"] | ["players"] => TeamField::Members,
        ["size"] => TeamField::Size,
        other => TeamField::Prop(TeamProp::from_words(other)?),
    };
    // `team <prop> of <t>` or `<prop> of team <t>`
    let after = n + 1;
    let named = p.peek_tok(after).is_some_and(|t| is(t, "team"));
    if lw != "team" && !named {
        return None;
    }
    p.advance(after + usize::from(named));
    Some(p.operand().map(|t| wrap(SbExpr::Team(field, t))))
}

// ---------- the stored copy ----------

const SEP: char = '\u{1}';

fn k_obj(name: &str) -> String {
    format!("{SEP}sb{SEP}o{SEP}{name}")
}
fn k_obj_criteria(name: &str) -> String {
    format!("{SEP}sb{SEP}c{SEP}{name}")
}
fn k_score_prefix(obj: &str) -> String {
    format!("{SEP}sb{SEP}s{SEP}{obj}{SEP}")
}
fn k_score(obj: &str, holder: &str) -> String {
    format!("{}{holder}", k_score_prefix(obj))
}
fn k_team(name: &str) -> String {
    format!("{SEP}sb{SEP}t{SEP}{name}")
}
fn k_team_prop(team: &str, prop: &str) -> String {
    format!("{SEP}sb{SEP}p{SEP}{team}{SEP}{prop}")
}
fn k_member_prefix(team: &str) -> String {
    format!("{SEP}sb{SEP}m{SEP}{team}{SEP}")
}
fn k_member(team: &str, who: &str) -> String {
    format!("{}{who}", k_member_prefix(team))
}
fn k_member_of(who: &str) -> String {
    format!("{SEP}sb{SEP}r{SEP}{who}")
}

/// A name the server accepts for an objective or team.
fn valid_name(kind: &str, s: &str) -> Result<(), String> {
    let ok = !s.is_empty()
        && s.len() <= 64
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+'));
    if ok {
        Ok(())
    } else {
        Err(format!(
            "'{s}' is not a valid {kind} name (use letters, numbers and _ - . + only, up to 64 characters)"
        ))
    }
}

/// A score holder or team member name (one word, no spaces).
fn valid_holder(s: &str) -> Result<(), String> {
    if s.is_empty()
        || s.len() > 64
        || s.chars()
            .any(|c| c.is_whitespace() || c == '"' || c == '*' || c == '@')
    {
        Err(format!(
            "'{s}' can't be used as a score holder or team member (no spaces)"
        ))
    } else {
        Ok(())
    }
}

/// `{"text":"..."}` for command arguments.
fn json_text(s: &str) -> String {
    let t = crate::colors::colorize(s);
    serde_json::json!({ "text": t }).to_string()
}

const COLORS: [&str; 17] = [
    "black",
    "dark_blue",
    "dark_green",
    "dark_aqua",
    "dark_red",
    "dark_purple",
    "gold",
    "gray",
    "dark_gray",
    "blue",
    "green",
    "aqua",
    "red",
    "light_purple",
    "yellow",
    "white",
    "reset",
];

fn norm_word(s: &str) -> String {
    s.trim().to_ascii_lowercase().replace([' ', '-'], "_")
}

fn team_color(s: &str) -> Result<String, String> {
    let n = norm_word(s);
    let n = match n.as_str() {
        "pink" => "light_purple".to_string(),
        "purple" => "dark_purple".to_string(),
        "grey" => "gray".to_string(),
        "dark_grey" => "dark_gray".to_string(),
        _ => n,
    };
    if COLORS.contains(&n.as_str()) {
        Ok(n)
    } else {
        Err(format!(
            "'{s}' is not a team color (try red, dark blue, light purple, ... or reset)"
        ))
    }
}

fn visibility(s: &str) -> Result<&'static str, String> {
    Ok(match norm_word(s).as_str() {
        "always" | "true" | "visible" => "always",
        "never" | "false" | "hidden" => "never",
        "hide_for_other_teams" => "hideForOtherTeams",
        "hide_for_own_team" => "hideForOwnTeam",
        _ => {
            return Err(format!(
                "'{s}' is not a name tag visibility (always, never, hide for other teams, hide for own team)"
            ));
        }
    })
}

fn collision(s: &str) -> Result<&'static str, String> {
    Ok(match norm_word(s).as_str() {
        "always" => "always",
        "never" => "never",
        "push_other_teams" => "pushOtherTeams",
        "push_own_team" => "pushOwnTeam",
        _ => {
            return Err(format!(
                "'{s}' is not a collision rule (always, never, push other teams, push own team)"
            ));
        }
    })
}

fn clamp_score(n: f64) -> i32 {
    if n.is_nan() {
        0
    } else {
        n.round().clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32
    }
}

fn get_text(vars: &Variables, key: &str) -> Option<String> {
    match vars.get_ref(key) {
        Some(Value::Text(t)) => Some(t.clone()),
        _ => None,
    }
}

/// Default for a team setting that was never changed.
fn prop_default(prop: TeamProp, team: &str) -> Value {
    match prop {
        TeamProp::Prefix | TeamProp::Suffix => Value::Text(String::new()),
        TeamProp::DisplayName => Value::Text(team.to_string()),
        TeamProp::Color => Value::Text("reset".into()),
        TeamProp::FriendlyFire => Value::Bool(true),
        TeamProp::SeeInvisibles => Value::Bool(true),
        TeamProp::NametagVisibility => Value::Text("always".into()),
        TeamProp::CollisionRule => Value::Text("always".into()),
        TeamProp::DeathMessageVisibility => Value::Text("always".into()),
    }
}

// ---------- a sidebar board ----------

#[derive(Debug, Default, Clone)]
pub struct Board {
    title: String,
    /// Line number (1 = top) to text.
    lines: BTreeMap<usize, String>,
}

impl Board {
    fn rendered(&self) -> Vec<String> {
        let last = self.lines.keys().next_back().copied().unwrap_or(0);
        (1..=last)
            .map(|i| self.lines.get(&i).cloned().unwrap_or_default())
            .collect()
    }
}

/// Per-player boards, by player uuid.
pub type Boards = std::collections::HashMap<String, Board>;

// ---------- running ----------

impl Engine {
    fn sb_text(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<String, String> {
        let v = self.eval(e, ex, host)?;
        Ok(match v {
            Value::Player(p) => p.name,
            Value::None => return Err("a scoreboard value is not set".into()),
            other => other.to_string(),
        })
    }

    fn sb_holder(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<String, String> {
        let v = self.eval(e, ex, host)?;
        let name = match &v {
            Value::Player(p) => online(&v, &mut ex.seen, host).map_or_else(|| p.name.clone(), |p| p.name),
            Value::None => return Err("the score holder is not set".into()),
            other => other.to_string(),
        };
        valid_holder(&name)?;
        Ok(name)
    }

    fn sb_online(
        &mut self,
        e: &Expr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Option<PlayerRef>, String> {
        let v = self.eval(e, ex, host)?;
        Ok(online(&v, &mut ex.seen, host))
    }

    pub(crate) fn sbt_effect(&mut self, fx: &SbFx, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        match fx {
            SbFx::RegisterObjective {
                name,
                criteria,
                display,
            } => {
                let name = self.sb_text(name, ex, host)?;
                valid_name("objective", &name)?;
                let criteria = match criteria {
                    Some(c) => self.sb_text(c, ex, host)?,
                    None => "dummy".to_string(),
                };
                let shown = match display {
                    Some(d) => Some(self.sb_text(d, ex, host)?),
                    None => None,
                };
                let crit = norm_word(&criteria).replace('_', "");
                let criteria = if crit.is_empty() || crit == "dummy" {
                    "dummy".to_string()
                } else {
                    criteria
                };
                if self.vars.get_ref(&k_obj(&name)).is_some() {
                    return Err(format!("objective '{name}' is already registered"));
                }
                host.sb_edit(&SbEdit::AddObjective {
                    name: name.clone(),
                    criteria: criteria.clone(),
                    display: shown.clone(),
                });
                self.vars
                    .set(k_obj(&name), Value::Text(shown.unwrap_or_else(|| name.clone())));
                self.vars.set(k_obj_criteria(&name), Value::Text(criteria));
                self.mark_dirty(host);
            }
            SbFx::UnregisterObjective(name) => {
                let name = self.sb_text(name, ex, host)?;
                valid_name("objective", &name)?;
                host.sb_edit(&SbEdit::RemoveObjective(name.clone()));
                self.vars.delete(&k_obj(&name));
                self.vars.delete(&k_obj_criteria(&name));
                for k in self.vars.keys_with_prefix(&k_score_prefix(&name)) {
                    self.vars.delete(&k);
                }
                self.mark_dirty(host);
            }
            SbFx::Score {
                op,
                holder,
                objective,
                value,
            } => {
                let who = self.sb_holder(holder, ex, host)?;
                let obj = self.sb_text(objective, ex, host)?;
                valid_name("objective", &obj)?;
                let n = self.num(value, ex, host)?;
                let key = k_score(&obj, &who);
                let old = match self.vars.get_ref(&key) {
                    Some(Value::Num(o)) => *o,
                    _ => 0.0,
                };
                let new = match op {
                    ScoreOp::Set => n,
                    ScoreOp::Add => old + n,
                    ScoreOp::Remove => old - n,
                };
                let new = f64::from(clamp_score(new));
                if self.vars.get_ref(&k_obj(&obj)).is_none() {
                    host.log(
                        crate::host::LogLevel::Warn,
                        &format!(
                            "objective '{obj}' was not registered by Carve; the server may refuse the score"
                        ),
                    );
                }
                host.sb_edit(&SbEdit::SetScore {
                    holder: who.clone(),
                    objective: obj.clone(),
                    value: new as i32,
                });
                self.vars.set(key, Value::Num(new));
                self.mark_dirty(host);
            }
            SbFx::ResetScore { holder, objective } => {
                let who = self.sb_holder(holder, ex, host)?;
                match objective {
                    Some(o) => {
                        let obj = self.sb_text(o, ex, host)?;
                        valid_name("objective", &obj)?;
                        host.sb_edit(&SbEdit::ResetScore {
                            holder: who.clone(),
                            objective: Some(obj.clone()),
                        });
                        self.vars.delete(&k_score(&obj, &who));
                    }
                    None => {
                        host.sb_edit(&SbEdit::ResetScore {
                            holder: who.clone(),
                            objective: None,
                        });
                        let objs: Vec<String> = self
                            .vars
                            .keys_with_prefix(&format!("{SEP}sb{SEP}o{SEP}"))
                            .into_iter()
                            .filter_map(|k| k.rsplit(SEP).next().map(str::to_string))
                            .collect();
                        for o in objs {
                            self.vars.delete(&k_score(&o, &who));
                        }
                    }
                }
                self.mark_dirty(host);
            }
            SbFx::RegisterTeam(name) => {
                let name = self.sb_text(name, ex, host)?;
                valid_name("team", &name)?;
                if self.vars.get_ref(&k_team(&name)).is_some() {
                    return Err(format!("team '{name}' is already registered"));
                }
                host.sb_edit(&SbEdit::AddTeam(name.clone()));
                self.vars.set(k_team(&name), Value::Text(name.clone()));
                self.mark_dirty(host);
            }
            SbFx::UnregisterTeam(name) => {
                let name = self.sb_text(name, ex, host)?;
                valid_name("team", &name)?;
                host.sb_edit(&SbEdit::RemoveTeam(name.clone()));
                self.team_forget(&name);
                self.vars.delete(&k_team(&name));
                self.mark_dirty(host);
            }
            SbFx::ClearTeam(name) => {
                let name = self.sb_text(name, ex, host)?;
                valid_name("team", &name)?;
                host.sb_edit(&SbEdit::EmptyTeam(name.clone()));
                self.team_clear_members(&name);
                self.mark_dirty(host);
            }
            SbFx::Join { who, team } => {
                let who = self.sb_holder(who, ex, host)?;
                let team = self.sb_text(team, ex, host)?;
                valid_name("team", &team)?;
                if self.vars.get_ref(&k_team(&team)).is_none() {
                    return Err(format!(
                        "team '{team}' is not registered (use 'register team \"{team}\"' first)"
                    ));
                }
                if let Some(old) = get_text(&self.vars, &k_member_of(&who)) {
                    self.vars.delete(&k_member(&old, &who));
                }
                host.sb_edit(&SbEdit::Join {
                    team: team.clone(),
                    holder: who.clone(),
                });
                self.vars.set(k_member(&team, &who), Value::Text(who.clone()));
                self.vars.set(k_member_of(&who), Value::Text(team));
                self.mark_dirty(host);
            }
            SbFx::Leave { who, team } => {
                let who = self.sb_holder(who, ex, host)?;
                let current = get_text(&self.vars, &k_member_of(&who));
                if let Some(t) = team {
                    let t = self.sb_text(t, ex, host)?;
                    if current.as_deref() != Some(t.as_str()) {
                        return Ok(());
                    }
                }
                if let Some(old) = current {
                    host.sb_edit(&SbEdit::Leave { holder: who.clone() });
                    self.vars.delete(&k_member(&old, &who));
                    self.vars.delete(&k_member_of(&who));
                    self.mark_dirty(host);
                }
            }
            SbFx::TeamSet { team, prop, value } => {
                let team = self.sb_text(team, ex, host)?;
                valid_name("team", &team)?;
                if self.vars.get_ref(&k_team(&team)).is_none() {
                    return Err(format!("team '{team}' is not registered"));
                }
                let v = self.eval(value, ex, host)?;
                let (arg, stored) = match prop {
                    TeamProp::Prefix | TeamProp::Suffix | TeamProp::DisplayName => {
                        let t = match v {
                            Value::None => return Err("the value is not set".into()),
                            other => other.to_string(),
                        };
                        (t.clone(), Value::Text(t))
                    }
                    TeamProp::Color => {
                        let c = team_color(&v.to_string())?;
                        (c.clone(), Value::Text(c))
                    }
                    TeamProp::FriendlyFire | TeamProp::SeeInvisibles => {
                        let b = match v {
                            Value::Bool(b) => b,
                            other => {
                                return Err(format!("expected true or false, got {}", other.type_name()));
                            }
                        };
                        (b.to_string(), Value::Bool(b))
                    }
                    TeamProp::NametagVisibility | TeamProp::DeathMessageVisibility => {
                        let s = visibility(&v.to_string())?;
                        (s.to_string(), Value::Text(norm_word(&v.to_string())))
                    }
                    TeamProp::CollisionRule => {
                        let s = collision(&v.to_string())?;
                        (s.to_string(), Value::Text(norm_word(&v.to_string())))
                    }
                };
                host.sb_edit(&SbEdit::TeamModify {
                    team: team.clone(),
                    prop: *prop,
                    arg,
                });
                self.vars.set(k_team_prop(&team, prop.key()), stored);
                self.mark_dirty(host);
            }
            SbFx::BoardTitle { player, text } => {
                let t = self.sb_text(text, ex, host)?;
                if let Some(p) = self.sb_online(player, ex, host)? {
                    let b = self.boards.entry(p.uuid.clone()).or_default();
                    b.title = t;
                    self.board_draw(&p, host);
                }
            }
            SbFx::BoardLine { player, line, text } => {
                let n = self.num(line, ex, host)?;
                if n.fract() != 0.0 || n < 1.0 || n > MAX_BOARD_LINES as f64 {
                    return Err(format!(
                        "board line must be a whole number from 1 to {MAX_BOARD_LINES}"
                    ));
                }
                let n = n as usize;
                let t = match text {
                    Some(t) => Some(self.sb_text(t, ex, host)?),
                    None => None,
                };
                if let Some(p) = self.sb_online(player, ex, host)? {
                    let b = self.boards.entry(p.uuid.clone()).or_default();
                    match t {
                        Some(t) => {
                            b.lines.insert(n, t);
                        }
                        None => {
                            b.lines.remove(&n);
                        }
                    }
                    self.board_draw(&p, host);
                }
            }
            SbFx::BoardLines { player, lines } => {
                if lines.len() > MAX_BOARD_LINES {
                    return Err(format!(
                        "a board has at most {MAX_BOARD_LINES} lines, got {}",
                        lines.len()
                    ));
                }
                let mut out = BTreeMap::new();
                for (i, l) in lines.iter().enumerate() {
                    out.insert(i + 1, self.sb_text(l, ex, host)?);
                }
                if let Some(p) = self.sb_online(player, ex, host)? {
                    self.boards.entry(p.uuid.clone()).or_default().lines = out;
                    self.board_draw(&p, host);
                }
            }
            SbFx::BoardClear(player) => {
                if let Some(p) = self.sb_online(player, ex, host)? {
                    self.boards.remove(&p.uuid);
                    host.board_clear(&p);
                }
            }
        }
        Ok(())
    }

    fn board_draw(&mut self, p: &PlayerRef, host: &mut dyn Host) {
        if let Some(b) = self.boards.get(&p.uuid) {
            let title = if b.title.is_empty() { " " } else { &b.title };
            host.board_show(p, title, &b.rendered());
        }
    }

    fn team_clear_members(&mut self, team: &str) {
        for k in self.vars.keys_with_prefix(&k_member_prefix(team)) {
            if let Some(Value::Text(who)) = self.vars.get_ref(&k).cloned() {
                self.vars.delete(&k_member_of(&who));
            }
            self.vars.delete(&k);
        }
    }

    fn team_forget(&mut self, team: &str) {
        self.team_clear_members(team);
        for k in self
            .vars
            .keys_with_prefix(&format!("{SEP}sb{SEP}p{SEP}{team}{SEP}"))
        {
            self.vars.delete(&k);
        }
    }

    pub(crate) fn sb_cond(&mut self, c: &SbCond, ex: &mut Exec, host: &mut dyn Host) -> Result<bool, String> {
        Ok(match c {
            SbCond::ObjectiveExists(o) => {
                let o = self.sb_text(o, ex, host)?;
                self.vars.get_ref(&k_obj(&o)).is_some()
            }
            SbCond::TeamExists(t) => {
                let t = self.sb_text(t, ex, host)?;
                self.vars.get_ref(&k_team(&t)).is_some()
            }
            SbCond::InTeam { who, team } => {
                let who = self.sb_holder(who, ex, host)?;
                let team = self.sb_text(team, ex, host)?;
                get_text(&self.vars, &k_member_of(&who)).is_some_and(|t| t == team)
            }
        })
    }

    pub(crate) fn sb_eval(
        &mut self,
        e: &SbExpr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        Ok(match e {
            SbExpr::Score { holder, objective } => {
                let who = self.sb_holder(holder, ex, host)?;
                let obj = self.sb_text(objective, ex, host)?;
                self.vars
                    .get_ref(&k_score(&obj, &who))
                    .cloned()
                    .unwrap_or(Value::None)
            }
            SbExpr::TeamOf(who) => {
                let who = self.sb_holder(who, ex, host)?;
                get_text(&self.vars, &k_member_of(&who)).map_or(Value::None, Value::Text)
            }
            SbExpr::ObjectiveCriteria(o) => {
                let o = self.sb_text(o, ex, host)?;
                get_text(&self.vars, &k_obj_criteria(&o)).map_or(Value::None, Value::Text)
            }
            SbExpr::ObjectiveDisplay(o) => {
                let o = self.sb_text(o, ex, host)?;
                get_text(&self.vars, &k_obj(&o)).map_or(Value::None, Value::Text)
            }
            SbExpr::Team(field, team) => {
                let team = self.sb_text(team, ex, host)?;
                if self.vars.get_ref(&k_team(&team)).is_none() {
                    return Ok(Value::None);
                }
                match field {
                    TeamField::Members => Value::List(
                        self.vars
                            .list(&k_member_prefix(&team))
                            .into_iter()
                            .map(|(_, v)| v)
                            .collect(),
                    ),
                    TeamField::Size => Value::Num(self.vars.list(&k_member_prefix(&team)).len() as f64),
                    TeamField::Prop(p) => match self.vars.get_ref(&k_team_prop(&team, p.key())) {
                        Some(Value::Bool(b)) => Value::Bool(*b),
                        Some(v) if !p.is_bool() => {
                            // The stored visibility and collision words read back with spaces.
                            match (p, v) {
                                (
                                    TeamProp::NametagVisibility
                                    | TeamProp::DeathMessageVisibility
                                    | TeamProp::CollisionRule
                                    | TeamProp::Color,
                                    Value::Text(t),
                                ) => Value::Text(t.replace('_', " ")),
                                _ => v.clone(),
                            }
                        }
                        _ => match prop_default(*p, &team) {
                            Value::Text(t) if matches!(p, TeamProp::Color) => Value::Text(t),
                            v => v,
                        },
                    },
                }
            }
        })
    }
}
