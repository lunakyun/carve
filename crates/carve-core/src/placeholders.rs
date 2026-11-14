//! PlaceholderAPI-style placeholders.
//!
//! * `placeholder "player_name" for player` and
//!   `parse placeholders in "Hi %player_name%" for player` (written
//!   `%%player_name%%` inside a script string, because `%%` is a literal `%`).
//! * Built-in placeholders ([`builtin`]) and `carve_var_<name>`.
//! * Scripts add their own with `on placeholder request for prefix "x":`
//!   (see `w2_events_4::ExtEvent::PlaceholderRequest`): inside, `the
//!   identifier` is the text after `x_` and `set the result to ...` answers.
//! * Other plugins reach all of it through a small IPC protocol, see
//!   [`handle_ipc`].
//!
//! Every expression is one boxed [`Ph`], so the hot `Expr` enum stays small.

use crate::expr::{Env, Expr, Ty, parse_sum_at};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::{PlayerRef, Value, format_num};
use crate::vars::Variables;

/// Longest text that placeholders get parsed in, in bytes.
pub const MAX_PARSE_LEN: usize = 64 * 1024;
/// How deep a placeholder script may parse placeholders itself.
pub const MAX_DEPTH: u8 = 8;

#[derive(Debug, Clone, PartialEq)]
pub enum Ph {
    /// `placeholder "name" [for player]`
    Get(Expr, Option<Expr>),
    /// `parse placeholders in "text" [for player]`
    Parse(Expr, Option<Expr>),
}

impl Ph {
    pub fn ty(&self) -> Ty {
        Ty::Text
    }
}

fn is(toks: &[Tok], i: usize, s: &str) -> bool {
    toks.get(i).is_some_and(|t| t.is_lit(s))
}

/// The optional `for <player>` / `from <player>` / `of <player>` tail.
fn player_tail(toks: &[Tok], p: usize, env: &Env) -> Result<(Option<Expr>, usize), String> {
    if is(toks, p, "for") || is(toks, p, "from") || is(toks, p, "of") {
        let (e, end) = parse_sum_at(toks, p + 1, env)?;
        if !Ty::Player.accepts(e.ty(env)) {
            return Err(format!("'for' needs a player here, got {}", e.ty(env).name()));
        }
        return Ok((Some(e), end));
    }
    Ok((None, p))
}

/// Words that start one of these expressions. `pos` is the index after the
/// word `lw` (already lowercase). `None` when the words are not ours.
pub fn parse_word(toks: &[Tok], pos: usize, env: &Env, lw: &str) -> Result<Option<(Expr, usize)>, String> {
    use crate::w2_events_4::ExtEvent;
    match lw {
        "identifier" | "result" | "prefix" if env.ext4 == Some(ExtEvent::PlaceholderRequest) => {
            let name = if lw == "prefix" { "prefix" } else { lw };
            let i = ExtEvent::PlaceholderRequest.slot_index(name).unwrap_or(0);
            // `placeholder prefix` is the long form of `prefix`.
            Ok(Some((Expr::Ev4Val(i as u8), pos)))
        }
        "placeholder" if !is(toks, pos, "request") => {
            if pos >= toks.len() {
                return Err("expected 'placeholder \"name\" for <player>'".into());
            }
            let (name, p) = parse_sum_at(toks, pos, env)?;
            if !Ty::Text.accepts(name.ty(env)) {
                return Err(format!("placeholder name must be text, got {}", name.ty(env).name()));
            }
            let (pl, end) = player_tail(toks, p, env)?;
            Ok(Some((Expr::Ph(Box::new(Ph::Get(name, pl))), end)))
        }
        "parse" | "parsed" if is(toks, pos, "placeholders") || is(toks, pos, "placeholder") => {
            if !is(toks, pos + 1, "in") && !is(toks, pos + 1, "of") {
                return Err("expected 'parse placeholders in \"text\" for <player>'".into());
            }
            let (text, p) = parse_sum_at(toks, pos + 2, env)?;
            if !Ty::Text.accepts(text.ty(env)) {
                return Err(format!("can only parse placeholders in text, got {}", text.ty(env).name()));
            }
            let (pl, end) = player_tail(toks, p, env)?;
            Ok(Some((Expr::Ph(Box::new(Ph::Parse(text, pl))), end)))
        }
        _ => Ok(None),
    }
}

/// `placeholder request for prefix "x"` -> (`placeholder request`, Some("x")).
/// Other headers come back unchanged. `ev` is lowercase, without `on `.
pub fn split_header(ev: &str) -> (&str, Option<String>) {
    let Some(rest) = ev.strip_prefix("placeholder request") else {
        return (ev, None);
    };
    let base = ev.get(.."placeholder request".len()).unwrap_or(ev);
    let rest = rest.trim();
    let rest = rest
        .strip_prefix("for ")
        .or_else(|| rest.strip_prefix("with "))
        .unwrap_or(rest)
        .trim();
    let Some(p) = rest.strip_prefix("prefix") else {
        return (ev, None);
    };
    let p = p.trim().trim_matches('"').trim();
    if p.is_empty() {
        return (ev, None);
    }
    (base, Some(p.trim_end_matches('_').to_string()))
}

/// Name of a `set the <name> to` target in a placeholder request, as the
/// slot name it stands for.
pub fn slot_alias(name: &str) -> Option<&'static str> {
    match name {
        "result" => Some("result"),
        "identifier" => Some("identifier"),
        "prefix" => Some("prefix"),
        _ => None,
    }
}

// ---- values ----------------------------------------------------------------

fn num(n: f64) -> String {
    format_num(n)
}

fn hms(millis: i64) -> String {
    let s = millis.div_euclid(1000).rem_euclid(86_400);
    format!("{:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
}

/// The built-in placeholders. `name` is the text between the percent signs.
/// `None`: not a built-in, or it needs a player and there is none.
pub fn builtin(name: &str, player: Option<&PlayerRef>, vars: &Variables, host: &mut dyn Host) -> Option<String> {
    if let Some(var) = name.strip_prefix("carve_var_") {
        let v = match vars.get_ref(var) {
            Some(v) => v.clone(),
            None => vars.get_ref(&var.to_ascii_lowercase())?.clone(),
        };
        if matches!(v, Value::None) {
            return None;
        }
        let mut s = String::new();
        v.push_to(&mut s);
        return Some(s);
    }
    let lname = name.to_ascii_lowercase();
    match lname.as_str() {
        "server_online" | "server_online_players" => return Some(num(host.player_count())),
        "server_max_players" | "server_max" => return host.max_players().map(num),
        "server_time" => return Some(hms(host.wall_clock_millis())),
        "server_motd" => return host.motd(),
        "server_version" => return host.server_version(),
        _ => {}
    }
    let p = player?;
    match lname.as_str() {
        "player_name" => Some(p.name.clone()),
        "player_displayname" | "player_display_name" => {
            Some(host.display_name(p).unwrap_or_else(|| p.name.clone()))
        }
        "player_uuid" => Some(p.uuid.clone()),
        "player_health" => host.health(p).map(num),
        "player_level" => host.xp_get(p, crate::xp_level::XpField::Level).map(num),
        "player_gamemode" => host.gamemode(p),
        "player_world" => host.player_location(p).map(|l| l.3),
        "player_x" => host.player_location(p).map(|l| num(l.0.floor())),
        "player_y" => host.player_location(p).map(|l| num(l.1.floor())),
        "player_z" => host.player_location(p).map(|l| num(l.2.floor())),
        "player_ping" => host.ping(p).map(num),
        _ => None,
    }
}

/// Built-in placeholder names, for the docs and `/carve` help.
pub const BUILTIN_NAMES: &[&str] = &[
    "player_name",
    "player_displayname",
    "player_uuid",
    "player_health",
    "player_level",
    "player_gamemode",
    "player_world",
    "player_x",
    "player_y",
    "player_z",
    "player_ping",
    "server_online",
    "server_max_players",
    "server_time",
    "server_motd",
    "server_version",
    "carve_var_<name>",
];

/// Whether `s` can be a placeholder name between percent signs.
fn name_ok(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '.' | '-' | '/'))
}

/// Replace every `%name%` that `resolve` knows. Unknown ones stay as they are.
pub fn replace_all(text: &str, mut resolve: impl FnMut(&str) -> Option<String>) -> String {
    if !text.contains('%') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find('%') {
        out.push_str(rest.get(..i).unwrap_or(""));
        let after = rest.get(i + 1..).unwrap_or("");
        if let Some(j) = after.find('%')
            && let name = after.get(..j).unwrap_or("")
            && name_ok(name)
            && let Some(v) = resolve(name)
        {
            out.push_str(&v);
            rest = after.get(j + 1..).unwrap_or("");
        } else {
            out.push('%');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

// ---- IPC for other plugins -------------------------------------------------

/// Message other plugins send to Carve through Pumpkin's plugin IPC.
///
/// UTF-8 text, three lines:
///
/// ```text
/// carve:parse        (or carve:get)
/// <player uuid, or empty>
/// <text with %placeholders%>        (or the placeholder name for carve:get)
/// ```
///
/// The answer is the parsed text (`carve:parse`) or the placeholder value
/// (`carve:get`, an error when it is unknown).
pub fn handle_ipc(
    engine: &mut crate::engine::Engine,
    message: &[u8],
    host: &mut dyn Host,
) -> Result<Vec<u8>, String> {
    let msg = std::str::from_utf8(message).map_err(|_| "message is not UTF-8".to_string())?;
    let mut lines = msg.splitn(3, '\n');
    let verb = lines.next().unwrap_or("").trim();
    let uuid = lines.next().unwrap_or("").trim();
    let body = lines.next().unwrap_or("");
    if body.len() > MAX_PARSE_LEN {
        return Err("text is too long".into());
    }
    let player = if uuid.is_empty() {
        None
    } else {
        host.resolve_player(uuid).map(|name| PlayerRef {
            uuid: uuid.to_string(),
            name,
        })
    };
    match verb {
        "carve:parse" => Ok(engine.parse_placeholders(body, player.as_ref(), host).into_bytes()),
        "carve:get" => engine
            .placeholder(body.trim().trim_matches('%'), player.as_ref(), host)
            .map(String::into_bytes)
            .ok_or_else(|| format!("unknown placeholder '{}'", body.trim())),
        other => Err(format!(
            "unknown request '{other}', use 'carve:parse' or 'carve:get' (see docs/placeholders.md)"
        )),
    }
}
