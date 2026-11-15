//! Scoreboards and boss bars: the sidebar, boss bars, action bar, timed
//! titles and the tab list header and footer.
//!
//! These lines have free-form words in them (`with color red`, `of player's
//! scoreboard`), so they are parsed by hand here instead of by the pattern
//! registry. `parse_effect` is called from the compiler before the registry;
//! the `Engine` method at the bottom runs the result. At run time all of it
//! sits behind one boxed `Effect` variant, so the hot paths do not grow.

use crate::compile::Effect;
use crate::engine::{Engine, Exec};
use crate::expr::{Env, Expr, Ty, parse_expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::{PlayerRef, Value};

/// Boss bar colors, as Minecraft names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

impl BarColor {
    pub fn parse(s: &str) -> Option<BarColor> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "pink" => BarColor::Pink,
            "blue" => BarColor::Blue,
            "red" => BarColor::Red,
            "green" => BarColor::Green,
            "yellow" => BarColor::Yellow,
            "purple" => BarColor::Purple,
            "white" => BarColor::White,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            BarColor::Pink => "pink",
            BarColor::Blue => "blue",
            BarColor::Red => "red",
            BarColor::Green => "green",
            BarColor::Yellow => "yellow",
            BarColor::Purple => "purple",
            BarColor::White => "white",
        }
    }
}

/// Boss bar styles: a solid bar or one cut into 6, 10, 12 or 20 notches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarStyle {
    Solid,
    Notches6,
    Notches10,
    Notches12,
    Notches20,
}

impl BarStyle {
    /// `solid`, `notched 10`, `segmented_6`, `12` and so on.
    pub fn parse(s: &str) -> Option<BarStyle> {
        let k = s.trim().to_ascii_lowercase().replace([' ', '-'], "_");
        let num = k.rsplit('_').next().unwrap_or("");
        let prefixed = k.contains('_') || k.chars().all(|c| c.is_ascii_digit());
        Some(match k.as_str() {
            "solid" | "progress" | "plain" | "none" | "no_division" => BarStyle::Solid,
            _ if prefixed => match num {
                "6" => BarStyle::Notches6,
                "10" => BarStyle::Notches10,
                "12" => BarStyle::Notches12,
                "20" => BarStyle::Notches20,
                _ => return None,
            },
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            BarStyle::Solid => "solid",
            BarStyle::Notches6 => "notched 6",
            BarStyle::Notches10 => "notched 10",
            BarStyle::Notches12 => "notched 12",
            BarStyle::Notches20 => "notched 20",
        }
    }
}

/// The extra switches on a boss bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarFlag {
    DarkenSky,
    Fog,
    Dragon,
}

impl BarFlag {
    fn parse(words: &str) -> Option<BarFlag> {
        Some(match words {
            "darken_sky" | "darken_the_sky" | "darkensky" | "sky" => BarFlag::DarkenSky,
            "fog" | "create_fog" | "fog_effect" => BarFlag::Fog,
            "dragon" | "dragon_bar" | "dragonbar" | "boss_music" => BarFlag::Dragon,
            _ => return None,
        })
    }
}

/// Who a screen effect goes to.
#[derive(Debug, Clone, PartialEq)]
pub enum Who {
    One(Expr),
    /// `all players` or `everyone`.
    All,
}

/// A color or style that is a plain word in the script, or a text to check later.
#[derive(Debug, Clone, PartialEq)]
pub enum Pick<T> {
    Fixed(T),
    Dyn(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BarSpec {
    pub id: Expr,
    pub title: Option<Expr>,
    pub color: Option<Pick<BarColor>>,
    pub style: Option<Pick<BarStyle>>,
    /// 0 to 100.
    pub progress: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SbFx {
    /// `send title ... for 3 seconds`. No times at all means the plain title.
    Title {
        title: Expr,
        subtitle: Option<Expr>,
        to: Who,
        fade_in: Option<Expr>,
        stay: Option<Expr>,
        fade_out: Option<Expr>,
    },
    ActionBar {
        text: Expr,
        to: Who,
    },
    ClearTitle {
        to: Who,
        reset: bool,
    },
    TabList {
        who: Who,
        header: Option<Expr>,
        footer: Option<Expr>,
    },
    TabListReset {
        who: Who,
        header: bool,
        footer: bool,
    },
    SideTitle {
        player: Expr,
        title: Expr,
    },
    SideLine {
        player: Expr,
        line: Expr,
        /// `None` clears the line.
        text: Option<Expr>,
    },
    SideRemove(Expr),
    BarCreate(Box<BarSpec>),
    BarTitle(Expr, Expr),
    /// Percent, 0 to 100.
    BarProgress(Expr, Expr),
    BarColor(Expr, Pick<BarColor>),
    BarStyle(Expr, Pick<BarStyle>),
    BarFlag(Expr, BarFlag, bool),
    BarShow(Expr, Who),
    /// `set bossbar visibility of <id> to true|false`: shown to / hidden from everyone.
    BarVisible(Expr, Expr),
    BarHide(Expr, Who),
    BarRemove(Expr),
}

/// Sidebar lines go from 1 (top) to this.
pub const MAX_LINES: u32 = 15;
const DEFAULT_FADE_IN: u32 = 10;
const DEFAULT_STAY: u32 = 70;
const DEFAULT_FADE_OUT: u32 = 20;
const MAX_TICKS: i64 = 72_000;

// ---------- parsing ----------

fn is(t: &Tok, lit: &str) -> bool {
    t.is_lit(lit)
}

fn lc(t: &Tok) -> Option<String> {
    match t {
        Tok::Word(w) => Some(w.to_ascii_lowercase()),
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

fn span(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Timespan])
}

fn player(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Player])
}

fn who(toks: &[Tok], env: &Env) -> Option<Who> {
    match toks {
        [a, b] if is(a, "all") && is(b, "players") => Some(Who::All),
        [a] if is(a, "everyone") => Some(Who::All),
        _ => player(toks, env).map(Who::One),
    }
}

/// Join `boss bar`, `action bar` and `tab list` into one word each, so the
/// parsers below only see `bossbar`, `actionbar` and `tablist`.
fn normalise(toks: &[Tok]) -> Vec<Tok> {
    let mut out = Vec::with_capacity(toks.len());
    let mut i = 0;
    while i < toks.len() {
        let a = lc(&toks[i]);
        let b = toks.get(i + 1).and_then(lc);
        let joined = match (a.as_deref(), b.as_deref()) {
            (Some("boss"), Some("bar")) => Some("bossbar"),
            (Some("action"), Some("bar")) => Some("actionbar"),
            (Some("tab"), Some("list")) => Some("tablist"),
            _ => None,
        };
        if let Some(j) = joined {
            out.push(Tok::Word(j.into()));
            i += 2;
            continue;
        }
        out.push(match a.as_deref() {
            Some("boss-bar") => Tok::Word("bossbar".into()),
            Some("action-bar") => Tok::Word("actionbar".into()),
            Some("tab-list") => Tok::Word("tablist".into()),
            Some("side-bar") => Tok::Word("sidebar".into()),
            _ => toks[i].clone(),
        });
        i += 1;
    }
    out
}

fn is_sep(t: &Tok) -> bool {
    is(t, "with") || is(t, "and") || is(t, ",")
}

fn trim_seps(mut toks: &[Tok]) -> &[Tok] {
    while let Some((last, rest)) = toks.split_last() {
        if is_sep(last) {
            toks = rest;
        } else {
            break;
        }
    }
    toks
}

/// Cut `toks` at clause markers. `mark(toks, i)` says what clause starts at
/// `i` and where its content starts. Returns the part before the first
/// clause, and each clause with its content.
fn split_clauses<K: Copy>(
    toks: &[Tok],
    mark: impl Fn(&[Tok], usize) -> Option<(K, usize)>,
) -> (&[Tok], Vec<(K, &[Tok])>) {
    let mut starts: Vec<(usize, K, usize)> = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        match mark(toks, i) {
            Some((k, c)) => {
                starts.push((i, k, c));
                i = c.max(i + 1);
            }
            None => i += 1,
        }
    }
    let head_end = starts.first().map_or(toks.len(), |s| s.0);
    let head = trim_seps(&toks[..head_end]);
    let mut out = Vec::new();
    for (n, (_, k, c)) in starts.iter().enumerate() {
        let end = starts.get(n + 1).map_or(toks.len(), |s| s.0);
        out.push((*k, trim_seps(&toks[(*c).min(end)..end])));
    }
    (head, out)
}

/// The words `scoreboard`, `bossbar` and so on, which pick the area.
const KEYS: &[&str] = &[
    "bossbar",
    "boss-bar",
    "boss",
    "scoreboard",
    "sidebar",
    "side-bar",
    "tablist",
    "tab-list",
    "tab",
    "actionbar",
    "action-bar",
    "action",
    "title",
];

const VERBS: &[&str] = &[
    "send", "set", "clear", "reset", "remove", "delete", "hide", "show", "create", "make", "add",
];

/// `Ok(None)`: not one of these lines. `Err`: it is one, but wrong.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    if !VERBS.iter().any(|v| first.eq_ignore_ascii_case(v)) {
        return Ok(None);
    }
    let any_key = toks
        .iter()
        .skip(1)
        .any(|t| matches!(t, Tok::Word(w) if KEYS.iter().any(|k| w.eq_ignore_ascii_case(k))));
    if !any_key {
        return Ok(None);
    }
    let toks = normalise(toks);
    let toks = toks.as_slice();
    let verb = lc(&toks[0]).unwrap_or_default();
    let has = |lit: &str| toks.iter().skip(1).any(|t| is(t, lit));
    let fx = if has("bossbar") {
        bar_line(toks, &verb, env)?
    } else if has("scoreboard") || has("sidebar") {
        side_line(toks, &verb, env)?
    } else if has("tablist") {
        tab_line(toks, &verb, env)?
    } else if verb == "send" && toks.get(1).is_some_and(|t| is(t, "actionbar")) {
        action_line(toks, env)?
    } else if verb == "send" && toks.get(1).is_some_and(|t| is(t, "title")) {
        title_line(toks, env)
    } else if matches!(verb.as_str(), "clear" | "reset") && toks.get(1).is_some_and(|t| is(t, "title")) {
        clear_title_line(toks, env)
    } else {
        None
    };
    Ok(fx.map(|f| Effect::Sb(Box::new(f))))
}

// ----- titles and action bar -----

#[derive(Clone, Copy)]
enum Cl {
    For,
    FadeIn,
    FadeOut,
}

fn title_mark(toks: &[Tok], i: usize) -> Option<(Cl, usize)> {
    let t = &toks[i];
    if is(t, "for") {
        return Some((Cl::For, i + 1));
    }
    if is(t, "fade-in") {
        return Some((Cl::FadeIn, i + 1));
    }
    if is(t, "fade-out") {
        return Some((Cl::FadeOut, i + 1));
    }
    if is(t, "fade") {
        match toks.get(i + 1) {
            Some(n) if is(n, "in") => return Some((Cl::FadeIn, i + 2)),
            Some(n) if is(n, "out") => return Some((Cl::FadeOut, i + 2)),
            _ => {}
        }
    }
    None
}

/// `send title "a" [with subtitle "b"] to <who> [for <time>] [with fade in <time>] ...`.
/// Only lines with times or `all players` come here. The plain form stays with
/// the pattern registry.
fn title_line(toks: &[Tok], env: &Env) -> Option<SbFx> {
    let (head, clauses) = split_clauses(toks.get(2..)?, title_mark);
    let to_i = head.iter().rposition(|t| is(t, "to"))?;
    let to = who(head.get(to_i + 1..)?, env)?;
    if clauses.is_empty() && !matches!(to, Who::All) {
        return None;
    }
    let main = head.get(..to_i)?;
    let sub_i = main
        .windows(2)
        .position(|w| is(&w[0], "with") && is(&w[1], "subtitle"));
    let (title_toks, sub_toks) = match sub_i {
        Some(i) => (trim_seps(&main[..i]), Some(&main[i + 2..])),
        None => (main, None),
    };
    let title = text(title_toks, env)?;
    let subtitle = match sub_toks {
        Some(s) => Some(text(s, env)?),
        None => None,
    };
    let (mut fade_in, mut stay, mut fade_out) = (None, None, None);
    for (k, c) in clauses {
        let e = Some(span(c, env)?);
        match k {
            Cl::For => stay = e,
            Cl::FadeIn => fade_in = e,
            Cl::FadeOut => fade_out = e,
        }
    }
    Some(SbFx::Title {
        title,
        subtitle,
        to,
        fade_in,
        stay,
        fade_out,
    })
}

/// `clear title of <who>`, `reset title of <who>`.
fn clear_title_line(toks: &[Tok], env: &Env) -> Option<SbFx> {
    let rest = toks.get(2..)?;
    let rest = match rest.first() {
        Some(t) if is(t, "of") || is(t, "for") || is(t, "from") => &rest[1..],
        _ => rest,
    };
    Some(SbFx::ClearTitle {
        to: who(rest, env)?,
        reset: is(&toks[0], "reset"),
    })
}

/// `send actionbar "x" to <who>`. The two-word `send action bar` to one player
/// is the registry's job; the same line to `all players` is handled here too.
fn action_line(toks: &[Tok], env: &Env) -> Result<Option<SbFx>, String> {
    if toks
        .get(2)
        .is_some_and(|t| is(t, "component") || is(t, "components"))
    {
        return Ok(None);
    }
    let body = &toks[2..];
    let Some(to_i) = body.iter().rposition(|t| is(t, "to")) else {
        return Err("expected 'send actionbar \"text\" to <player>'".into());
    };
    let (Some(text), Some(to)) = (text(&body[..to_i], env), who(&body[to_i + 1..], env)) else {
        return Err("expected 'send actionbar \"text\" to <player>'".into());
    };
    Ok(Some(SbFx::ActionBar { text, to }))
}

// ----- tab list -----

fn tab_line(toks: &[Tok], verb: &str, env: &Env) -> Result<Option<SbFx>, String> {
    const USAGE: &str =
        "expected 'set tab list header of <player> to \"text\"' (also footer, or 'all players')";
    let t = toks.iter().position(|t| is(t, "tablist")).unwrap_or(0);
    // Left of `tablist`: the verb, `the`, and for `<player>'s tab list` the player.
    let left = &toks[1..t];
    let left = match left.first() {
        Some(x) if is(x, "the") => &left[1..],
        _ => left,
    };
    let mut subject = match left.split_last() {
        Some((last, rest)) if is(last, "'s") && !rest.is_empty() => Some(rest),
        _ if left.is_empty() => None,
        _ => return Err(USAGE.into()),
    };
    // Right of `tablist`: header/footer words, then `of <who>`, then `to <value>`.
    let right = &toks[t + 1..];
    // `tab list name` belongs to the display name effects.
    if right.first().is_some_and(|x| is(x, "name")) {
        return Ok(None);
    }
    let (mut header, mut footer) = (false, false);
    let mut i = 0;
    while let Some(w) = right.get(i).and_then(lc) {
        match w.as_str() {
            "header" => header = true,
            "footer" => footer = true,
            "and" if i > 0 => {}
            _ => break,
        }
        i += 1;
    }
    if !header && !footer {
        header = true;
        footer = true;
    }
    let mut rest = &right[i..];
    let mut value_toks = None;
    if let Some(to_i) = rest.iter().position(|t| is(t, "to")) {
        value_toks = Some(&rest[to_i + 1..]);
        rest = &rest[..to_i];
    }
    if let Some(f) = rest.first() {
        if is(f, "of") || is(f, "for") {
            subject = Some(&rest[1..]);
        } else {
            return Err(USAGE.into());
        }
    }
    let Some(subject) = subject.and_then(|s| who(s, env)) else {
        return Err(USAGE.into());
    };
    match verb {
        "set" => {
            let Some(v) = value_toks.and_then(|v| text(v, env)) else {
                return Err(USAGE.into());
            };
            // Both sides at once get the same text.
            let both = header && footer;
            Ok(Some(SbFx::TabList {
                who: subject,
                header: header.then(|| v.clone()),
                footer: (footer || both).then_some(v),
            }))
        }
        "reset" | "clear" | "remove" | "delete" if value_toks.is_none() => Ok(Some(SbFx::TabListReset {
            who: subject,
            header,
            footer,
        })),
        _ => Err(USAGE.into()),
    }
}

// ----- sidebar scoreboard -----

fn side_line(toks: &[Tok], verb: &str, env: &Env) -> Result<Option<SbFx>, String> {
    const USAGE: &str = "expected 'set title of <player>'s scoreboard to \"text\"', \
        'set line 1 of <player>'s scoreboard to \"text\"' or 'clear <player>'s scoreboard'";
    let s = toks
        .iter()
        .position(|t| is(t, "scoreboard") || is(t, "sidebar"))
        .unwrap_or(0);
    let left = &toks[1..s];
    let right = &toks[s + 1..];
    let left = match left.first() {
        Some(x) if is(x, "the") => &left[1..],
        _ => left,
    };
    // `prefix` holds the property when it comes before the scoreboard.
    let (prefix, player_toks, suffix): (&[Tok], &[Tok], &[Tok]) = if let Some((last, rest)) =
        left.split_last()
        && is(last, "'s")
    {
        match rest.iter().position(|t| is(t, "of")) {
            Some(o) => (&rest[..o], &rest[o + 1..], right),
            None => (&[], rest, right),
        }
    } else if right.first().is_some_and(|t| is(t, "of")) {
        let rest = &right[1..];
        match rest.iter().position(|t| is(t, "to")) {
            Some(i) => (left, &rest[..i], &rest[i..]),
            None => (left, rest, &[]),
        }
    } else {
        return Err(USAGE.into());
    };
    let prefix = match prefix.split_last() {
        Some((l, p)) if is(l, "of") => p,
        _ => prefix,
    };
    // The property is before the scoreboard (`title of`) or after it (`'s title`).
    let (prop_toks, value_toks): (&[Tok], Option<&[Tok]>) = match suffix.iter().position(|t| is(t, "to")) {
        Some(i) => (&suffix[..i], Some(&suffix[i + 1..])),
        None => (suffix, None),
    };
    let prop: &[Tok] = if prefix.is_empty() { prop_toks } else { prefix };
    let Some(p) = player(player_toks, env) else {
        return Err(USAGE.into());
    };
    let line_of = |prop: &[Tok]| -> Option<Expr> {
        match prop.split_first() {
            Some((l, n)) if is(l, "line") && !n.is_empty() => number(n, env),
            _ => None,
        }
    };
    let is_title = matches!(prop, [t] if is(t, "title") || is(t, "name"));
    let fx = match verb {
        "set" => {
            let Some(v) = value_toks.and_then(|v| text(v, env)) else {
                return Err(USAGE.into());
            };
            if is_title {
                SbFx::SideTitle { player: p, title: v }
            } else if let Some(line) = line_of(prop) {
                SbFx::SideLine {
                    player: p,
                    line,
                    text: Some(v),
                }
            } else {
                return Err(USAGE.into());
            }
        }
        "clear" | "reset" | "remove" | "delete" | "hide" if value_toks.is_none() => {
            if prop.is_empty() {
                SbFx::SideRemove(p)
            } else if let Some(line) = line_of(prop) {
                SbFx::SideLine {
                    player: p,
                    line,
                    text: None,
                }
            } else {
                return Err(USAGE.into());
            }
        }
        _ => return Err(USAGE.into()),
    };
    Ok(Some(fx))
}

// ----- boss bars -----

#[derive(Clone, Copy)]
enum BarCl {
    Title,
    Color,
    Style,
    Progress,
}

fn bar_mark(toks: &[Tok], i: usize) -> Option<(BarCl, usize)> {
    let w = lc(&toks[i])?;
    let direct = match w.as_str() {
        "titled" => Some(BarCl::Title),
        "colored" | "coloured" => Some(BarCl::Color),
        "styled" => Some(BarCl::Style),
        _ => None,
    };
    if let Some(k) = direct {
        return Some((k, i + 1));
    }
    let after_with = i > 0 && is_sep(&toks[i - 1]);
    let k = match w.as_str() {
        "title" | "name" => BarCl::Title,
        "color" | "colour" => BarCl::Color,
        "style" => BarCl::Style,
        "progress" => BarCl::Progress,
        _ => return None,
    };
    after_with.then_some((k, i + 1))
}

/// A word like `red` or `notched 10`, or a text to check when the script runs.
fn pick<T>(toks: &[Tok], env: &Env, parse: fn(&str) -> Option<T>, what: &str) -> Result<Pick<T>, String> {
    let mut words = String::new();
    let mut plain = !toks.is_empty();
    for t in toks {
        match t {
            Tok::Word(w) => {
                if !words.is_empty() {
                    words.push('_');
                }
                words.push_str(w);
            }
            Tok::Num(n) => {
                if !words.is_empty() {
                    words.push('_');
                }
                words.push_str(&format!("{n}"));
            }
            _ => plain = false,
        }
    }
    if plain {
        return match parse(&words) {
            Some(v) => Ok(Pick::Fixed(v)),
            None => Err(format!("unknown bossbar {what} '{}'", words.replace('_', " "))),
        };
    }
    text(toks, env)
        .map(Pick::Dyn)
        .ok_or_else(|| format!("expected a bossbar {what}"))
}

fn bar_line(toks: &[Tok], verb: &str, env: &Env) -> Result<Option<SbFx>, String> {
    const USAGE: &str = "expected something like 'create bossbar \"id\" with title \"Boss\" with color red', \
        'set progress of bossbar \"id\" to 50', 'show bossbar \"id\" to player' or 'remove bossbar \"id\"'";
    let b = toks.iter().position(|t| is(t, "bossbar")).unwrap_or(0);
    let id_of = |t: &[Tok]| text(t, env).ok_or_else(|| "expected a bossbar id like \"boss\"".to_string());
    let who_of = |t: &[Tok]| who(t, env).ok_or_else(|| "expected a player or 'all players'".to_string());
    let after = &toks[b + 1..];
    let fx = match verb {
        "create" | "make" | "add"
            if b > 0
                && toks[1..b]
                    .iter()
                    .all(|t| matches!(lc(t).as_deref(), Some("a" | "new" | "the"))) =>
        {
            let (head, clauses) = split_clauses(after, bar_mark);
            let mut head = head;
            loop {
                match head.first().and_then(lc).as_deref() {
                    Some("named" | "called" | "id" | "key" | "name") => head = &head[1..],
                    Some("with")
                        if head
                            .get(1)
                            .is_some_and(|t| is(t, "id") || is(t, "key") || is(t, "name")) =>
                    {
                        head = &head[2..];
                    }
                    _ => break,
                }
            }
            let mut spec = BarSpec {
                id: id_of(head)?,
                title: None,
                color: None,
                style: None,
                progress: None,
            };
            for (k, c) in clauses {
                match k {
                    BarCl::Title => spec.title = Some(text(c, env).ok_or("expected a title text")?),
                    BarCl::Progress => {
                        spec.progress = Some(number(c, env).ok_or("expected a progress number")?)
                    }
                    BarCl::Color => spec.color = Some(pick(c, env, BarColor::parse, "color")?),
                    BarCl::Style => spec.style = Some(pick(c, env, BarStyle::parse, "style")?),
                }
            }
            SbFx::BarCreate(Box::new(spec))
        }
        "set" => bar_set(toks, b, env)?,
        "show" | "display" if b == 1 => {
            let to = find(after, 0, "to").ok_or(USAGE)?;
            SbFx::BarShow(id_of(&after[..to])?, who_of(&after[to + 1..])?)
        }
        "hide" if b == 1 => {
            let from = find(after, 0, "from").ok_or(USAGE)?;
            SbFx::BarHide(id_of(&after[..from])?, who_of(&after[from + 1..])?)
        }
        "remove" | "delete" | "clear" if b == 1 => SbFx::BarRemove(id_of(after)?),
        "add" | "remove" if b > 1 => {
            // `add <who> to bossbar <id>`, `add <flag> flag to bossbar <id>`.
            let sep = toks[..b]
                .iter()
                .rposition(|t| is(t, "to") || is(t, "from"))
                .ok_or(USAGE)?;
            let add = verb == "add";
            if (add && !is(&toks[sep], "to")) || (!add && !is(&toks[sep], "from")) {
                return Err(USAGE.into());
            }
            let subj = &toks[1..sep];
            let id = id_of(after)?;
            match flag_of(subj) {
                Some(f) => SbFx::BarFlag(id, f, add),
                None if add => SbFx::BarShow(id, who_of(subj)?),
                None => SbFx::BarHide(id, who_of(subj)?),
            }
        }
        _ => return Err(USAGE.into()),
    };
    Ok(Some(fx))
}

/// `darken sky flag`, `fog`, `dragon bar`.
fn flag_of(toks: &[Tok]) -> Option<BarFlag> {
    let mut words = Vec::new();
    for t in toks {
        words.push(lc(t)?);
    }
    if words.last().is_some_and(|w| w == "flag" || w == "flags") {
        words.pop();
    }
    if words.first().is_some_and(|w| w == "the") {
        words.remove(0);
    }
    BarFlag::parse(&words.join("_"))
}

/// `set <prop> of [the] bossbar <id> to <v>` and `set bossbar <id>'s <prop> to <v>`.
fn bar_set(toks: &[Tok], b: usize, env: &Env) -> Result<SbFx, String> {
    const USAGE: &str = "expected 'set progress|title|color|style of bossbar \"id\" to <value>'";
    let to_i = find(toks, b + 1, "to").ok_or(USAGE)?;
    let value = &toks[to_i + 1..];
    let (prop, id_toks) = if b == 1 && toks.get(3).is_some_and(|t| is(t, "of")) && lc(&toks[2]).is_some() {
        // `set bossbar progress of <id> to 50`
        if to_i < 5 {
            return Err(USAGE.into());
        }
        (lc(&toks[2]).ok_or(USAGE)?, &toks[4..to_i])
    } else if b == 1 {
        // `set bossbar "id"'s progress to 50`
        let s = find(toks, 2, "'s").ok_or(USAGE)?;
        if s + 2 != to_i {
            return Err(USAGE.into());
        }
        (lc(&toks[s + 1]).ok_or(USAGE)?, &toks[2..s])
    } else {
        let the = b == 4 && is(&toks[3], "the");
        if !(is(&toks[2], "of") && (b == 3 || the)) {
            return Err(USAGE.into());
        }
        (lc(&toks[1]).ok_or(USAGE)?, &toks[b + 1..to_i])
    };
    let id = text(id_toks, env).ok_or("expected a bossbar id like \"boss\"")?;
    Ok(match prop.as_str() {
        "visibility" | "visible" | "visiblity" => SbFx::BarVisible(
            id,
            expr_of(value, env, &[Ty::Boolean]).ok_or("expected true or false")?,
        ),
        "title" | "name" => SbFx::BarTitle(id, text(value, env).ok_or("expected a title text")?),
        "progress" | "health" | "value" | "percentage" => SbFx::BarProgress(
            id,
            number(value, env).ok_or("expected a progress number from 0 to 100")?,
        ),
        "color" | "colour" => SbFx::BarColor(id, pick(value, env, BarColor::parse, "color")?),
        "style" | "division" | "divisions" => SbFx::BarStyle(id, pick(value, env, BarStyle::parse, "style")?),
        other => {
            return Err(format!(
                "a bossbar has no '{other}' to set; use progress, title, color or style"
            ));
        }
    })
}

// ---------- running ----------

fn clamp_ticks(ms: i64) -> u32 {
    (ms / 50).clamp(0, MAX_TICKS) as u32
}

impl Engine {
    fn sb_targets(&mut self, to: &Who, ex: &mut Exec, host: &mut dyn Host) -> Result<Vec<PlayerRef>, String> {
        match to {
            Who::One(e) => Ok(self.player(e, ex, host)?.into_iter().collect()),
            Who::All => Ok(host.online_players()),
        }
    }

    fn sb_ticks(
        &mut self,
        e: &Option<Expr>,
        default: u32,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<u32, String> {
        let Some(e) = e else { return Ok(default) };
        match self.eval(e, ex, host)? {
            Value::Timespan { millis } => Ok(clamp_ticks(millis)),
            other => Err(format!(
                "expected a time like 3 seconds, got {} ({other})",
                other.type_name()
            )),
        }
    }

    fn sb_pick<T: Copy>(
        &mut self,
        p: &Pick<T>,
        parse: fn(&str) -> Option<T>,
        what: &str,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<T, String> {
        match p {
            Pick::Fixed(v) => Ok(*v),
            Pick::Dyn(e) => {
                let s = self.text(e, ex, host)?;
                parse(&s).ok_or_else(|| format!("unknown bossbar {what} '{s}'"))
            }
        }
    }

    #[inline(never)]
    pub(crate) fn sb_effect(&mut self, fx: &SbFx, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        match fx {
            SbFx::Title {
                title,
                subtitle,
                to,
                fade_in,
                stay,
                fade_out,
            } => {
                let t = self.text(title, ex, host)?;
                let s = match subtitle {
                    Some(s) => Some(self.text(s, ex, host)?),
                    None => None,
                };
                let timed = fade_in.is_some() || stay.is_some() || fade_out.is_some();
                let times = (
                    self.sb_ticks(fade_in, DEFAULT_FADE_IN, ex, host)?,
                    self.sb_ticks(stay, DEFAULT_STAY, ex, host)?,
                    self.sb_ticks(fade_out, DEFAULT_FADE_OUT, ex, host)?,
                );
                for p in self.sb_targets(to, ex, host)? {
                    if timed {
                        host.send_title_timed(&p, &t, s.as_deref(), times.0, times.1, times.2);
                    } else {
                        host.send_title(&p, &t, s.as_deref());
                    }
                }
            }
            SbFx::ActionBar { text, to } => {
                let t = self.text(text, ex, host)?;
                for p in self.sb_targets(to, ex, host)? {
                    host.send_action_bar(&p, &t);
                }
            }
            SbFx::ClearTitle { to, reset } => {
                for p in self.sb_targets(to, ex, host)? {
                    host.clear_title(&p, *reset);
                }
            }
            SbFx::TabList { who, header, footer } => {
                let h = match header {
                    Some(e) => Some(self.text(e, ex, host)?),
                    None => None,
                };
                let f = match footer {
                    Some(e) => Some(self.text(e, ex, host)?),
                    None => None,
                };
                for p in self.sb_targets(who, ex, host)? {
                    host.set_tab_list(&p, h.as_deref(), f.as_deref());
                }
            }
            SbFx::TabListReset { who, header, footer } => {
                for p in self.sb_targets(who, ex, host)? {
                    host.set_tab_list(&p, header.then_some(""), footer.then_some(""));
                }
            }
            SbFx::SideTitle { player, title } => {
                let t = self.text(title, ex, host)?;
                if let Some(p) = self.player(player, ex, host)? {
                    host.sidebar_set_title(&p, &t);
                }
            }
            SbFx::SideLine { player, line, text } => {
                let n = self.num(line, ex, host)?;
                if !n.is_finite() || n < 1.0 || n > f64::from(MAX_LINES) {
                    return Err(format!("scoreboard line must be 1 to {MAX_LINES}, got {n}"));
                }
                let n = n as u32;
                let t = match text {
                    Some(t) => Some(self.text(t, ex, host)?),
                    None => None,
                };
                if let Some(p) = self.player(player, ex, host)? {
                    match t {
                        Some(t) => host.sidebar_set_line(&p, n, &t),
                        None => host.sidebar_clear_line(&p, n),
                    }
                }
            }
            SbFx::SideRemove(player) => {
                if let Some(p) = self.player(player, ex, host)? {
                    host.sidebar_remove(&p);
                }
            }
            SbFx::BarCreate(spec) => {
                let id = self.text(&spec.id, ex, host)?;
                let title = match &spec.title {
                    Some(t) => self.text(t, ex, host)?,
                    None => id.clone(),
                };
                let color = match &spec.color {
                    Some(c) => self.sb_pick(c, BarColor::parse, "color", ex, host)?,
                    None => BarColor::Pink,
                };
                let style = match &spec.style {
                    Some(c) => self.sb_pick(c, BarStyle::parse, "style", ex, host)?,
                    None => BarStyle::Solid,
                };
                let progress = match &spec.progress {
                    Some(e) => self.num(e, ex, host)?,
                    None => 100.0,
                };
                host.bossbar_create(&id, &title, color, style, fraction(progress));
            }
            SbFx::BarTitle(id, t) => {
                let id = self.text(id, ex, host)?;
                let t = self.text(t, ex, host)?;
                host.bossbar_set_title(&id, &t);
            }
            SbFx::BarProgress(id, v) => {
                let id = self.text(id, ex, host)?;
                let v = self.num(v, ex, host)?;
                host.bossbar_set_progress(&id, fraction(v));
            }
            SbFx::BarColor(id, c) => {
                let id = self.text(id, ex, host)?;
                let c = self.sb_pick(c, BarColor::parse, "color", ex, host)?;
                host.bossbar_set_color(&id, c);
            }
            SbFx::BarStyle(id, c) => {
                let id = self.text(id, ex, host)?;
                let c = self.sb_pick(c, BarStyle::parse, "style", ex, host)?;
                host.bossbar_set_style(&id, c);
            }
            SbFx::BarFlag(id, f, on) => {
                let id = self.text(id, ex, host)?;
                host.bossbar_set_flag(&id, *f, *on);
            }
            SbFx::BarShow(id, to) => {
                let id = self.text(id, ex, host)?;
                for p in self.sb_targets(to, ex, host)? {
                    host.bossbar_show(&id, &p);
                }
            }
            SbFx::BarVisible(id, on) => {
                let id = self.text(id, ex, host)?;
                let on = match self.eval(on, ex, host)? {
                    Value::Bool(b) => b,
                    Value::Text(t) => t.eq_ignore_ascii_case("true"),
                    _ => false,
                };
                for p in host.online_players() {
                    if on {
                        host.bossbar_show(&id, &p);
                    } else {
                        host.bossbar_hide(&id, &p);
                    }
                }
            }
            SbFx::BarHide(id, to) => {
                let id = self.text(id, ex, host)?;
                for p in self.sb_targets(to, ex, host)? {
                    host.bossbar_hide(&id, &p);
                }
            }
            SbFx::BarRemove(id) => {
                let id = self.text(id, ex, host)?;
                host.bossbar_remove(&id);
            }
        }
        Ok(())
    }
}

/// Percent (0 to 100) to a 0.0 to 1.0 fraction. Bad numbers become 0.
fn fraction(percent: f64) -> f64 {
    if percent.is_finite() {
        (percent / 100.0).clamp(0.0, 1.0)
    } else {
        0.0
    }
}
