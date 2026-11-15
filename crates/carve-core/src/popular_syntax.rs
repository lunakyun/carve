//! Core Skript syntax that popular public scripts use: command argument types
//! and defaults, offline players, and the small helpers behind
//! `message`, `player command`, `loop all offline players` and friends.

use crate::compile::{ArgTy, CmpOp, Cond};
use crate::expr::{Env, Expr, Part, Ty, parse_expr};
use crate::host::Host;
use crate::lexer::{Tok, lex};
use crate::value::{PlayerRef, Value};

impl ArgTy {
    /// `<offline player>`, `<integer>`, ... as written in a command header.
    pub fn from_name(name: &str) -> Option<ArgTy> {
        Some(
            match name.trim().to_ascii_lowercase().replace('_', " ").as_str() {
                "text" | "string" => ArgTy::Text,
                "number" | "decimal" | "double" | "float" => ArgTy::Number,
                "integer" | "int" | "long" => ArgTy::Integer,
                "player" => ArgTy::Player,
                "offline player" | "offlineplayer" => ArgTy::OfflinePlayer,
                "boolean" | "bool" => ArgTy::Boolean,
                "world" => ArgTy::World,
                "item" | "itemtype" | "item type" => ArgTy::Item,
                "timespan" | "time span" => ArgTy::Timespan,
                _ => return None,
            },
        )
    }

    /// The name shown in usage lines.
    pub fn name(self) -> &'static str {
        match self {
            ArgTy::Text => "text",
            ArgTy::Number => "number",
            ArgTy::Integer => "integer",
            ArgTy::Player => "player",
            ArgTy::OfflinePlayer => "offline player",
            ArgTy::Boolean => "boolean",
            ArgTy::World => "world",
            ArgTy::Item => "item",
            ArgTy::Timespan => "timespan",
        }
    }

    /// The script type of `arg-1` and friends.
    pub fn ty(self) -> Ty {
        match self {
            ArgTy::Text => Ty::Text,
            ArgTy::Number | ArgTy::Integer => Ty::Number,
            ArgTy::Player | ArgTy::OfflinePlayer => Ty::Player,
            ArgTy::Boolean => Ty::Boolean,
            ArgTy::World => Ty::World,
            ArgTy::Item => Ty::Item,
            ArgTy::Timespan => Ty::Timespan,
        }
    }
}

/// The `<type>` and `[<type>]` groups of a command header, in order. The
/// bool is true for a group in brackets (an optional argument).
pub fn split_arg_specs(s: &str) -> Result<Vec<(String, bool)>, String> {
    let c: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        match c[i] {
            ch if ch.is_whitespace() => i += 1,
            '[' => {
                let mut depth = 0usize;
                let start = i + 1;
                let mut end = None;
                for (j, ch) in c.iter().enumerate().skip(i) {
                    match ch {
                        '[' => depth += 1,
                        ']' => {
                            depth -= 1;
                            if depth == 0 {
                                end = Some(j);
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let Some(end) = end else {
                    return Err("missing ']' in the command's arguments".into());
                };
                let inner: String = c[start..end].iter().collect();
                let inner = inner.trim();
                let Some(spec) = inner.strip_prefix('<').and_then(|t| t.strip_suffix('>')) else {
                    return Err(format!(
                        "unknown argument '[{inner}]', use [<text>] for an optional argument"
                    ));
                };
                out.push((spec.to_string(), true));
                i = end + 1;
            }
            '<' => {
                let Some(rel) = c[i..].iter().position(|ch| *ch == '>') else {
                    return Err("missing '>' in the command's arguments".into());
                };
                let spec: String = c[i + 1..i + rel].iter().collect();
                out.push((spec, false));
                i += rel + 1;
            }
            _ => {
                let word: String = c[i..].iter().take_while(|ch| !ch.is_whitespace()).collect();
                return Err(format!(
                    "unknown argument '{word}', use <text>, <number>, <player> or [<text>] for optional"
                ));
            }
        }
    }
    Ok(out)
}

/// The default of `<number=5>`, `<player=%player%>`, `<text=hello>`.
pub fn compile_default(ty: ArgTy, raw: &str) -> Result<Expr, String> {
    let env = Env::default();
    let raw = raw.trim();
    if let Some(inner) = raw
        .strip_prefix('%')
        .and_then(|r| r.strip_suffix('%'))
        .filter(|r| !r.contains('%'))
    {
        let toks = lex(inner)?;
        return parse_expr(&toks, &env);
    }
    if ty == ArgTy::Text && !raw.starts_with('"') {
        return Ok(Expr::Text(vec![Part::Lit(raw.to_string())]));
    }
    let toks = lex(raw)?;
    let e = parse_expr(&toks, &env)?;
    if matches!(ty, ArgTy::Text) {
        return Ok(e);
    }
    Ok(e)
}

/// The offline-mode UUID of a name, the one a server without sign-in checks gives.
pub fn offline_uuid(name: &str) -> String {
    let hex = crate::hashing::digest_hex(
        crate::hashing::HashAlg::Md5,
        format!("OfflinePlayer:{name}").as_bytes(),
    );
    let b: Vec<char> = hex.chars().collect();
    let variant = match b.get(16).and_then(|c| c.to_digit(16)) {
        Some(d) => char::from_digit((d & 0x3) | 0x8, 16).unwrap_or('8'),
        None => '8',
    };
    let s: String = b
        .iter()
        .enumerate()
        .map(|(i, c)| match i {
            12 => '3',
            16 => variant,
            _ => *c,
        })
        .collect();
    format!(
        "{}-{}-{}-{}-{}",
        s.get(..8).unwrap_or(""),
        s.get(8..12).unwrap_or(""),
        s.get(12..16).unwrap_or(""),
        s.get(16..20).unwrap_or(""),
        s.get(20..).unwrap_or("")
    )
}

/// A player by name, online or not.
pub fn offline_player(name: &str, known: &[PlayerRef], host: &mut dyn Host) -> PlayerRef {
    host.find_player(name)
        .or_else(|| known.iter().find(|k| k.name.eq_ignore_ascii_case(name)).cloned())
        .or_else(|| host.offline_player(name))
        .unwrap_or_else(|| PlayerRef {
            uuid: offline_uuid(name),
            name: name.to_string(),
        })
}

/// `2 hours 30 minutes`, `1 minute and 5 seconds`.
pub fn parse_timespan(s: &str) -> Option<i64> {
    let words: Vec<&str> = s
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|w| !w.is_empty() && !w.eq_ignore_ascii_case("and"))
        .collect();
    if words.is_empty() || !words.len().is_multiple_of(2) {
        return None;
    }
    let mut total = 0i64;
    for pair in words.chunks(2) {
        let n: f64 = pair.first()?.parse().ok()?;
        let unit = crate::timespans::unit_ms(pair.get(1)?)?;
        total = total.checked_add(crate::timespans::span_ms(n, unit)?)?;
    }
    Some(total)
}

/// One typed command argument, read from the text the player typed.
pub fn parse_arg(
    ty: ArgTy,
    word: &str,
    usage: &str,
    known: &[PlayerRef],
    host: &mut dyn Host,
) -> Result<Value, String> {
    Ok(match ty {
        ArgTy::Text => Value::Text(word.to_string()),
        ArgTy::Number => Value::Num(
            word.parse::<f64>()
                .ok()
                .filter(|n| n.is_finite())
                .ok_or_else(|| format!("'{word}' is not a number. Usage: {usage}"))?,
        ),
        ArgTy::Integer => Value::Num(
            word.parse::<f64>()
                .ok()
                .filter(|n| n.is_finite() && n.fract() == 0.0)
                .ok_or_else(|| format!("'{word}' is not an integer. Usage: {usage}"))?,
        ),
        ArgTy::Player => Value::Player(
            host.find_player(word)
                .ok_or_else(|| format!("Player '{word}' is not online"))?,
        ),
        ArgTy::OfflinePlayer => Value::Player(offline_player(word, known, host)),
        ArgTy::Boolean => match word.to_ascii_lowercase().as_str() {
            "true" | "yes" | "on" => Value::Bool(true),
            "false" | "no" | "off" => Value::Bool(false),
            _ => {
                return Err(format!(
                    "'{word}' is not a boolean (true or false). Usage: {usage}"
                ));
            }
        },
        ArgTy::World => {
            let names = host.world_names();
            match names.iter().find(|n| n.eq_ignore_ascii_case(word)) {
                Some(n) => Value::World(n.clone()),
                None if names.is_empty() => Value::World(word.to_string()),
                None => return Err(format!("There is no world called '{word}'")),
            }
        }
        ArgTy::Item => {
            let id = word
                .trim()
                .trim_start_matches("minecraft:")
                .to_ascii_lowercase()
                .replace(' ', "_");
            if crate::items::is_known(&id) {
                Value::Item {
                    id,
                    count: 1,
                    meta: None,
                }
            } else {
                return Err(format!("'{word}' is not an item. Usage: {usage}"));
            }
        }
        ArgTy::Timespan => match parse_timespan(word) {
            Some(millis) => Value::Timespan { millis },
            None => {
                return Err(format!(
                    "'{word}' is not a timespan (like 5 minutes). Usage: {usage}"
                ));
            }
        },
    })
}

// ---------- conditions ----------

/// `an hour`, `a minute`: the article is a one.
fn span_tokens(toks: &[Tok]) -> Vec<Tok> {
    let mut v = toks.to_vec();
    if matches!(v.first(), Some(t) if t.is_lit("a") || t.is_lit("an")) && v.len() > 1 {
        v[0] = Tok::Num(1.0);
    }
    v
}

/// `{d} was less than 5 minutes ago`, `{x} is greater than 2 seconds` and the
/// other word forms of `<`, `>`, `<=` and `>=` for dates and timespans.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    // `arg-2 and arg-1 are set`, `{a} and {b} are not set`
    if let [head @ .., verb, set] = toks
        && verb.is_lit("are")
        && set.is_lit("set")
    {
        return all_set(head, true, env);
    }
    if let [head @ .., verb, not, set] = toks
        && verb.is_lit("are")
        && not.is_lit("not")
        && set.is_lit("set")
    {
        return all_set(head, false, env);
    }
    if let Some(c) = is_type(toks, env) {
        return Some(c);
    }
    // `player is holding anything`: the hand is not empty.
    if let [head @ .., is, holding, anything] = toks
        && is.is_lit("is")
        && holding.is_lit("holding")
        && anything.is_lit("anything")
        && !head.is_empty()
    {
        let mut t = vec![Tok::Word("tool".into()), Tok::Word("of".into())];
        t.extend(head.iter().cloned());
        let tool = parse_expr(&t, env).ok()?;
        let air = Expr::Item {
            id: "air".to_string(),
            count: 1,
        };
        return Some(Cond::Cmp(tool, CmpOp::Ne, air));
    }
    // `player's ip contains "a" or "b"`: any of them.
    if let Some(c) = toks.iter().position(|t| t.is_lit("contains"))
        && toks.iter().skip(c).any(|t| t.is_lit("or"))
    {
        let left = parse_expr(toks.get(..c)?, env).ok()?;
        if left.ty(env) != Ty::Text && left.ty(env) != Ty::Object {
            return None;
        }
        let mut any = Vec::new();
        for part in toks.get(c + 1..)?.split(|t| t.is_lit("or")) {
            any.push(Cond::Text {
                left: left.clone(),
                op: crate::compile::TextOp::Contains,
                right: parse_expr(part, env).ok()?,
                negate: false,
            });
        }
        return Some(Cond::Any(any));
    }
    let is_cmp_word = |t: &Tok| {
        [
            "less", "more", "greater", "smaller", "larger", "bigger", "higher", "fewer", "lower", "at", "was",
        ]
        .iter()
        .any(|w| t.is_lit(w))
    };
    if !toks.iter().any(is_cmp_word) {
        return None;
    }
    // `<date> was (less|more) than <span> ago`
    if let Some(was) = toks.iter().position(|t| t.is_lit("was"))
        && toks.last().is_some_and(|t| t.is_lit("ago"))
        && let Some(than) = toks.iter().position(|t| t.is_lit("than"))
        && than > was
    {
        let words: Vec<&str> = toks
            .get(was + 1..than)?
            .iter()
            .filter_map(|t| match t {
                Tok::Word(w) => Some(w.as_str()),
                _ => None,
            })
            .collect();
        let op = match words.as_slice() {
            ["less"] | ["fewer"] | ["smaller"] | ["shorter"] => CmpOp::Lt,
            ["more"] | ["greater"] | ["larger"] | ["bigger"] | ["longer"] => CmpOp::Gt,
            _ => return None,
        };
        let when = parse_expr(toks.get(..was)?, env).ok()?;
        let span = parse_expr(&span_tokens(toks.get(than + 1..toks.len() - 1)?), env).ok()?;
        if span.ty(env) != Ty::Timespan || !matches!(when.ty(env), Ty::Date | Ty::Object) {
            return None;
        }
        let now = Expr::Date(crate::expr::DateOp::Now, Vec::new());
        let ago = Expr::TBin(Box::new(now), crate::expr::BinOp::Sub, Box::new(when));
        return Some(Cond::Cmp(ago, op, span));
    }
    // `<a> is less than <b>` where the sides are timespans or dates.
    let verb = toks.iter().position(|t| t.is_lit("is") || t.is_lit("are"))?;
    let rest = toks.get(verb + 1..)?;
    let table: [(&[&str], CmpOp); 10] = [
        (&["greater", "than", "or", "equal", "to"], CmpOp::Ge),
        (&["less", "than", "or", "equal", "to"], CmpOp::Le),
        (&["at", "least"], CmpOp::Ge),
        (&["at", "most"], CmpOp::Le),
        (&["greater", "than"], CmpOp::Gt),
        (&["more", "than"], CmpOp::Gt),
        (&["higher", "than"], CmpOp::Gt),
        (&["less", "than"], CmpOp::Lt),
        (&["smaller", "than"], CmpOp::Lt),
        (&["lower", "than"], CmpOp::Lt),
    ];
    for (words, op) in table {
        if rest.len() > words.len() && words.iter().zip(rest).all(|(w, t)| t.is_lit(w)) {
            let a = parse_expr(toks.get(..verb)?, env).ok()?;
            let b = parse_expr(&span_tokens(rest.get(words.len()..)?), env).ok()?;
            let timey = |t: Ty| matches!(t, Ty::Timespan | Ty::Date);
            let (ta, tb) = (a.ty(env), b.ty(env));
            if (timey(ta) || timey(tb)) && (ta == tb || ta == Ty::Object || tb == Ty::Object) {
                return Some(Cond::Cmp(a, op, b));
            }
            return None;
        }
    }
    None
}

/// A type word of `x is a location`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeWord {
    Location,
    Number,
    Text,
    Boolean,
    Entity,
}

impl TypeWord {
    fn from_word(w: &str) -> Option<TypeWord> {
        Some(match w.to_ascii_lowercase().as_str() {
            "location" => TypeWord::Location,
            "number" | "integer" => TypeWord::Number,
            "text" | "string" => TypeWord::Text,
            "boolean" => TypeWord::Boolean,
            "entity" => TypeWord::Entity,
            _ => return None,
        })
    }

    pub fn matches(self, v: &Value) -> bool {
        match self {
            TypeWord::Location => matches!(v, Value::Location { .. }),
            TypeWord::Number => matches!(v, Value::Num(_)),
            TypeWord::Text => matches!(v, Value::Text(_)),
            TypeWord::Boolean => matches!(v, Value::Bool(_)),
            TypeWord::Entity => matches!(v, Value::Entity(_) | Value::Player(_)),
        }
    }
}

/// Conditions of this module that need their own check at run time.
#[derive(Debug, Clone, PartialEq)]
pub enum PopCond {
    IsType { e: Expr, kind: TypeWord, negate: bool },
}

/// `x is [not] [a|an] location`.
fn is_type(toks: &[Tok], env: &Env) -> Option<Cond> {
    let verb = toks.iter().position(|t| t.is_lit("is") || t.is_lit("isn't"))?;
    let mut i = verb + 1;
    let mut negate = toks.get(verb).is_some_and(|t| t.is_lit("isn't"));
    if toks.get(i).is_some_and(|t| t.is_lit("not")) {
        negate = true;
        i += 1;
    }
    if toks.get(i).is_some_and(|t| t.is_lit("a") || t.is_lit("an")) {
        i += 1;
    }
    let [Tok::Word(w)] = toks.get(i..)? else {
        return None;
    };
    let kind = TypeWord::from_word(w)?;
    let e = parse_expr(toks.get(..verb)?, env).ok()?;
    Some(Cond::Popular(Box::new(PopCond::IsType { e, kind, negate })))
}

fn all_set(head: &[Tok], want: bool, env: &Env) -> Option<Cond> {
    let parts = split_list(head);
    if parts.len() < 2 {
        return None;
    }
    let mut conds = Vec::new();
    for p in parts {
        conds.push(Cond::IsSet(parse_expr(p, env).ok()?, want));
    }
    Some(Cond::All(conds))
}

// ---------- expressions ----------

impl Env {
    /// Is this an event that happens somewhere (`on explode`), so `event-world`
    /// means something even with no player? `every` and `at` triggers and
    /// `on load` happen nowhere in particular.
    pub(crate) fn event_world_ok(&self) -> bool {
        let n = self.trigger_name.as_str();
        !(n.starts_with("'every") || n.starts_with("'at ") || n == "'on load'" || n.starts_with("command"))
    }
}

impl crate::expr::P<'_> {
    /// Expression heads of this module. `self.pos` is after the first word `lw`.
    pub(crate) fn popular_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let at = |p: &Self, n: usize, w: &str| {
            if p.pos + n >= p.toks.len() {
                p.hit_end.set(true);
            }
            matches!(p.toks.get(p.pos + n), Some(t) if t.is_lit(w))
        };
        match lw {
            // `name of current inventory of player`: the title of the open menu.
            "name" if at(self, 0, "of") && at(self, 1, "current") && at(self, 2, "inventory") => {
                self.pos += 3;
                if at(self, 0, "of") {
                    self.pos += 1;
                    self.primary()?;
                }
                Ok(Some(Expr::TextDirections(Box::new(
                    crate::expr_text_directions::TextExpr {
                        op: crate::expr_text_directions::TextOp::MenuTitle,
                        args: Vec::new(),
                    },
                ))))
            }
            // `event-block` in events that only know where it happened (`on redstone change`).
            "event-block"
                if !self.env.in_function
                    && !self.env.has_block
                    && crate::events_blocks_players::field_index(self.env.ev_fields, "event-block")
                        .is_none() =>
            {
                Ok(
                    crate::events_blocks_players::field_index(self.env.ev_fields, "event-location")
                        .map(|f| Expr::ExtVal(f.0)),
                )
            }
            // `the entity` in `on spawn` and `on rightclick on <entity>`: the entity itself.
            "entity" | "creature"
                if !self.env.in_function
                    && self.env.event_vals & crate::events_items::EventVal::Entity.bit() != 0
                    && !at(self, 0, "from")
                    && !at(self, 0, "type") =>
            {
                Ok(Some(Expr::TextDirections(Box::new(
                    crate::expr_text_directions::TextExpr {
                        op: crate::expr_text_directions::TextOp::EventEntity,
                        args: Vec::new(),
                    },
                ))))
            }
            // `arguments` in `on command`: what was typed after the command name.
            "arguments"
                if !self.env.in_function
                    && crate::events_blocks_players::field_index(
                        self.env.ev_fields,
                        "event-full-command",
                    )
                    .is_some() =>
            {
                let field = |name: &str| {
                    crate::events_blocks_players::field_index(self.env.ev_fields, name)
                        .map(|f| Expr::ExtVal(f.0))
                };
                match (field("event-full-command"), field("event-command")) {
                    (Some(full), Some(cmd)) => Ok(Some(Expr::TextDirections(Box::new(
                        crate::expr_text_directions::TextExpr {
                            op: crate::expr_text_directions::TextOp::CommandArgs,
                            args: vec![full, cmd],
                        },
                    )))),
                    _ => Ok(None),
                }
            }
            // `rounded player's x-coordinate`: the nearest whole number.
            "rounded" if self.pos < self.toks.len() => {
                let inner = self.primary()?;
                if !Ty::Number.accepts(inner.ty(self.env)) {
                    return Err("'rounded' needs a number".into());
                }
                Ok(Some(crate::functions_aliases::math_call(
                    crate::expr::MathFn::Round,
                    vec![inner],
                )))
            }
            _ => Ok(None),
        }
    }
}

/// `add <effect> to potion effects of <player>` is `apply <effect> to <player>`,
/// and `remove <effect> from potion effects of <player>` drops the `potion effects of`.
pub fn rewrite_effect(toks: &[Tok]) -> Option<Vec<Tok>> {
    let first = toks.first()?;
    if !(first.is_lit("add") || first.is_lit("remove")) {
        return None;
    }
    let p = toks
        .windows(3)
        .position(|w| w[0].is_lit("potion") && w[1].is_lit("effects") && w[2].is_lit("of"))?;
    let target = toks.get(p + 3..)?;
    let link = toks.get(p.checked_sub(1)?)?;
    if target.is_empty() {
        return None;
    }
    if first.is_lit("add") && link.is_lit("to") {
        let mut v = vec![Tok::Word("apply".into())];
        v.extend(toks.get(1..p)?.iter().cloned());
        v.extend(target.iter().cloned());
        return Some(v);
    }
    if first.is_lit("remove") && link.is_lit("from") {
        let mut v = toks.get(..p)?.to_vec();
        v.extend(target.iter().cloned());
        return Some(v);
    }
    None
}

/// Split a token list at the commas and `and`s that are outside brackets.
pub fn split_list(toks: &[Tok]) -> Vec<&[Tok]> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0usize);
    for (i, t) in toks.iter().enumerate() {
        if t.is_lit("(") {
            depth += 1;
        } else if t.is_lit(")") {
            depth -= 1;
        } else if depth == 0 && (t.is_lit(",") || t.is_lit("and")) {
            if let Some(part) = toks.get(start..i) {
                out.push(part);
            }
            start = i + 1;
        }
    }
    if let Some(part) = toks.get(start..) {
        out.push(part);
    }
    out
}

/// Cheap test before parsing a section header as an implicit `if`: a
/// condition has a verb in it.
pub fn looks_like_condition(toks: &[Tok]) -> bool {
    toks.iter().any(|t| {
        [
            "is", "are", "was", "has", "have", "contains", "contain", "doesn't", "isn't", "aren't", "not",
        ]
        .iter()
        .any(|w| t.is_lit(w))
    })
}
