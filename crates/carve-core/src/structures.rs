//! Structures: `variables:` defaults, `aliases:`, inline `<effect> if <cond>`,
//! `catch runtime errors:`, `parse if:` and `local function`.
//!
//! Most of this is sugar that compiles to the normal instructions. Only
//! `catch runtime errors:` has VM support (see `engine/structures_rt.rs`).

use std::sync::Arc;

use crate::blocks::Line;
use crate::compile::{Code, Cx, EventKind, Instr, Trigger, TriggerKind};
use crate::engine::ERROR_KEY;
use crate::error::ScriptError;
use crate::expr::{Env, Expr, Key, Part, Sig, VarRef, parse_expr};
use crate::lexer::{Tok, lex};
use crate::value::Value;

/// What a file's structure sections hold, apart from triggers and functions.
#[derive(Debug, Default)]
pub struct FileStructs {
    /// `variables:` defaults as `if {x} is not set:` lines, for everyone.
    var_global: Vec<Line>,
    /// The same for names that hold the player (`{coins::%player%}`).
    var_player: Vec<Line>,
}

fn is_structure(text: &str) -> bool {
    text.eq_ignore_ascii_case("variables:")
        || text.eq_ignore_ascii_case("aliases:")
        || text.get(..6).is_some_and(|h| h.eq_ignore_ascii_case("using "))
}

/// Take `variables:`, `aliases:` and `using ...` out of the top level, and
/// apply the aliases to the rest.
/// Broken entries are reported and left out; the rest is kept.
pub fn extract(file: &str, tree: Vec<Line>) -> (Vec<Line>, FileStructs, Vec<ScriptError>) {
    if !tree.iter().any(|t| is_structure(&t.text)) {
        return (tree, FileStructs::default(), Vec::new());
    }
    let mut errors = Vec::new();
    let mut aliases: Vec<(String, String)> = Vec::new();
    let mut var_blocks: Vec<Line> = Vec::new();
    let mut rest = Vec::new();
    for top in tree {
        if top.text.eq_ignore_ascii_case("aliases:") {
            for a in &top.children {
                match a.text.split_once('=') {
                    Some((k, v))
                        if !k.trim().is_empty()
                            && !v.trim().is_empty()
                            && a.children.is_empty()
                            && !k.contains(['"', '{', '%', ':']) =>
                    {
                        let k = k.trim().to_ascii_lowercase();
                        if aliases.iter().any(|(n, _)| *n == k) {
                            errors.push(ScriptError::new(
                                file,
                                a.num,
                                format!("alias '{k}' is defined twice"),
                            ));
                        } else {
                            aliases.push((k, v.trim().to_string()));
                        }
                    }
                    _ => errors.push(ScriptError::new(
                        file,
                        a.num,
                        "aliases look like 'my sword = diamond sword'",
                    )),
                }
            }
        } else if top.text.eq_ignore_ascii_case("variables:") {
            var_blocks.push(top);
        } else if top
            .text
            .get(..6)
            .is_some_and(|h| h.eq_ignore_ascii_case("using "))
            && !top.opens_block()
        {
            // `using <experimental feature>`: Carve has none, so nothing to switch on.
        } else {
            rest.push(top);
        }
    }
    if !aliases.is_empty() {
        // Longest names first, so `iron sword` wins over `iron`.
        aliases.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));
        for l in &mut rest {
            apply_aliases(l, &aliases);
        }
        for b in &mut var_blocks {
            for l in &mut b.children {
                apply_aliases(l, &aliases);
            }
        }
    }
    let mut fs = FileStructs::default();
    for block in var_blocks {
        for v in block.children {
            match default_line(&v) {
                Ok((per_player, line)) => {
                    if per_player {
                        fs.var_player.push(line);
                    } else {
                        fs.var_global.push(line);
                    }
                }
                Err(m) => errors.push(ScriptError::new(file, v.num, m)),
            }
        }
    }
    (rest, fs, errors)
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

fn apply_aliases(line: &mut Line, aliases: &[(String, String)]) {
    line.text = replace_aliases(&line.text, aliases);
    for c in &mut line.children {
        apply_aliases(c, aliases);
    }
}

/// Replace whole-word alias names outside text and `{variables}`.
fn replace_aliases(text: &str, aliases: &[(String, String)]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_str = false;
    let mut depth = 0usize;
    let mut prev: Option<char> = None;
    let mut i = 0;
    while let Some(rest) = text.get(i..) {
        let Some(ch) = rest.chars().next() else { break };
        match ch {
            '"' => in_str = !in_str,
            '{' if !in_str => depth += 1,
            '}' if !in_str && depth > 0 => depth -= 1,
            _ => {}
        }
        if !in_str && depth == 0 && ch != '"' && !prev.is_some_and(is_word_char) && is_word_char(ch) {
            let hit = aliases.iter().find(|(k, _)| {
                rest.get(..k.len()).is_some_and(|h| h.eq_ignore_ascii_case(k))
                    && !rest
                        .get(k.len()..)
                        .and_then(|r| r.chars().next())
                        .is_some_and(is_word_char)
            });
            if let Some((k, v)) = hit {
                out.push_str(v);
                i += k.len();
                prev = v.chars().last();
                continue;
            }
        }
        out.push(ch);
        prev = Some(ch);
        i += ch.len_utf8();
    }
    out
}

/// Does a variable name hold the player (`{coins::%player%}`)?
fn names_player(name: &str) -> bool {
    name.to_ascii_lowercase()
        .split('%')
        .skip(1)
        .step_by(2)
        .any(|part| part.contains("player"))
}

/// `{coins::%player%} = 0` -> (per player, `if {...} is not set:` with a `set` inside).
fn default_line(l: &Line) -> Result<(bool, Line), String> {
    let bad = || "a default looks like '{name} = value'".to_string();
    if !l.children.is_empty() {
        return Err(bad());
    }
    let t = l.text.trim();
    let inner = t.strip_prefix('{').ok_or_else(bad)?;
    let mut depth = 1usize;
    let mut close = None;
    for (i, c) in inner.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close.ok_or_else(|| "missing '}' in the variable name".to_string())?;
    let name = inner.get(..close).unwrap_or("").trim();
    let value = inner
        .get(close + 1..)
        .unwrap_or("")
        .trim()
        .strip_prefix('=')
        .ok_or_else(bad)?
        .trim();
    if name.is_empty() || value.is_empty() {
        return Err(bad());
    }
    if name.starts_with('_') {
        return Err("'variables:' sets global variables; {_name} is local to a run".into());
    }
    let line = Line {
        num: l.num,
        text: format!("if {{{name}}} is not set:"),
        children: vec![Line {
            num: l.num,
            text: format!("set {{{name}}} to {value}"),
            children: Vec::new(),
        }],
    };
    Ok((names_player(name), line))
}

/// Compile a file's `variables:` defaults into one trigger per group.
pub fn compile_defaults(
    file: &str,
    fs: &FileStructs,
    functions: &Arc<Vec<Sig>>,
    errors: &mut Vec<ScriptError>,
) -> Vec<(bool, Trigger)> {
    let mut out = Vec::new();
    for (per_player, lines) in [(false, &fs.var_global), (true, &fs.var_player)] {
        let Some(first) = lines.first() else { continue };
        let env = Env {
            trigger_name: "'variables:'".into(),
            no_player: !per_player,
            functions: functions.clone(),
            ..Env::default()
        };
        let mut cx = Cx::new(file, env);
        cx.block(lines);
        if cx.errors.is_empty() {
            let file: Arc<str> = Arc::from(file);
            out.push((
                per_player,
                Trigger {
                    file: file.clone(),
                    line: first.num,
                    kind: TriggerKind::Event(EventKind::Load),
                    code: Arc::new(Code {
                        file,
                        line: first.num,
                        instrs: cx.code,
                        lines: cx.lines,
                    }),
                },
            ));
        } else {
            errors.extend(cx.errors);
        }
    }
    out
}

// ---- local functions ----

/// `function f(...)` or `local function f(...)`: the header as
/// `parse_function_header` wants it, and whether it is local.
pub fn function_head(head: &str) -> Option<(&str, bool)> {
    if head.get(..9).is_some_and(|h| h.eq_ignore_ascii_case("function ")) {
        Some((head, false))
    } else if head
        .get(..15)
        .is_some_and(|h| h.eq_ignore_ascii_case("local function "))
    {
        Some((head.get(6..).unwrap_or(head), true))
    } else {
        None
    }
}

/// Is a function with this name already visible where the new one is declared?
/// Globals clash with globals; a local clashes with anything in its own file.
pub fn function_clash(sigs: &[Sig], scope: &[(bool, usize)], name: &str, local: bool, file: usize) -> bool {
    sigs.iter()
        .zip(scope)
        .any(|(s, &(l, f))| s.name.eq_ignore_ascii_case(name) && ((!local && !l) || f == file))
}

/// The function list each file sees, indexed like the files. Local functions
/// of other files, and globals hidden by a local of the same name, get a name
/// no script can write, so calls skip them. Ids stay the same.
pub fn function_views(sigs: &[Sig], scope: &[(bool, usize)], files: usize) -> Vec<Arc<Vec<Sig>>> {
    let shared = Arc::new(sigs.to_vec());
    if !scope.iter().any(|&(l, _)| l) {
        return vec![shared; files];
    }
    (0..files)
        .map(|f| {
            let view = sigs
                .iter()
                .zip(scope)
                .map(|(s, &(local, sf))| {
                    let hidden = if local {
                        sf != f
                    } else {
                        sigs.iter()
                            .zip(scope)
                            .any(|(o, &(ol, of))| ol && of == f && o.name.eq_ignore_ascii_case(&s.name))
                    };
                    let mut s = s.clone();
                    if hidden {
                        s.name = format!("\u{1}{}", s.name);
                    }
                    s
                })
                .collect();
            Arc::new(view)
        })
        .collect()
}

// ---- sections and inline conditions ----

/// Sections that are not `if` / `loop`: `catch runtime errors:` and
/// `parse if <cond>:`. Returns true when the line was one of them.
pub(crate) fn section(cx: &mut Cx, line: &Line, lower: &str) -> bool {
    let head = lower.trim_end_matches(':').trim();
    if matches!(
        head,
        "catch runtime errors" | "catch runtime error" | "catch errors" | "catch error"
    ) {
        let begin = cx.emit(Instr::CatchBegin(usize::MAX), line.num);
        cx.catch_depth += 1;
        cx.block(&line.children);
        cx.catch_depth -= 1;
        cx.emit(Instr::CatchEnd, line.num);
        let end = cx.code.len();
        if let Some(Instr::CatchBegin(t)) = cx.code.get_mut(begin) {
            *t = end;
        }
        return true;
    }
    if head.starts_with("parse if ") {
        let text = line.head().get("parse if ".len()..).unwrap_or("");
        match const_cond(text, &cx.env) {
            Ok(true) => cx.block(&line.children),
            Ok(false) => {}
            Err(e) => cx.err(
                line.num,
                format!("can't decide 'parse if {text}' while loading: {e}"),
            ),
        }
        return true;
    }
    crate::w4_sections::section(cx, line, lower)
}

/// `<effect> if <cond>` or `<effect> unless <cond>`, outside text and
/// `{variables}`. Gives the effect (without a leading `do`), the condition
/// and whether it is `unless`.
pub fn split_inline(text: &str) -> Option<(&str, &str, bool)> {
    let mut in_str = false;
    let mut depth = 0usize;
    for (i, ch) in text.char_indices() {
        match ch {
            '"' => in_str = !in_str,
            '{' if !in_str => depth += 1,
            '}' if !in_str && depth > 0 => depth -= 1,
            c if c.is_whitespace() && !in_str && depth == 0 => {
                let rest = text.get(i + c.len_utf8()..).unwrap_or("");
                for (kw, unless) in [("if", false), ("unless", true)] {
                    let hit = rest.get(..kw.len()).is_some_and(|h| h.eq_ignore_ascii_case(kw))
                        && rest
                            .get(kw.len()..)
                            .is_some_and(|r| r.starts_with(char::is_whitespace));
                    if !hit {
                        continue;
                    }
                    let eff = text.get(..i).unwrap_or("").trim();
                    let cond = rest.get(kw.len()..).unwrap_or("").trim();
                    // `a if <cond> else b` is a ternary value, not an inline `if`.
                    if eff.is_empty() || cond.is_empty() || has_else(cond) {
                        return None;
                    }
                    let eff = match eff.get(..3) {
                        Some(d) if d.eq_ignore_ascii_case("do ") => eff.get(3..).unwrap_or(eff).trim(),
                        _ => eff,
                    };
                    return Some((eff, cond, unless));
                }
            }
            _ => {}
        }
    }
    None
}

/// Is there an `else` / `otherwise` word outside text and `{variables}`?
fn has_else(text: &str) -> bool {
    let mut in_str = false;
    let mut depth = 0usize;
    let mut word = String::new();
    for ch in text.chars().chain(std::iter::once(' ')) {
        match ch {
            '"' => in_str = !in_str,
            '{' if !in_str => depth += 1,
            '}' if !in_str && depth > 0 => depth -= 1,
            _ => {}
        }
        if !in_str && depth == 0 && ch.is_alphabetic() {
            word.push(ch);
            continue;
        }
        if word.eq_ignore_ascii_case("else") || word.eq_ignore_ascii_case("otherwise") {
            return true;
        }
        word.clear();
    }
    false
}

/// Compile a line like `send "hi" to player if player is op`. Returns true
/// when the line was one.
pub(crate) fn inline_if(cx: &mut Cx, line: &Line) -> bool {
    let Some((eff, cond, unless)) = split_inline(&line.text) else {
        return false;
    };
    let num = line.num;
    let Some(c) = cx.cond(num, cond) else {
        return true;
    };
    // The condition's jump skips the effect. For `unless` the effect sits
    // behind a second jump, taken when the condition is true.
    let skip = cx.emit(Instr::JumpIfNot(c, usize::MAX), num);
    let over = unless.then(|| {
        let at = cx.code.len();
        patch(cx, skip, at + 1);
        cx.emit(Instr::Jump(usize::MAX), num)
    });
    if !cx.control(num, eff, &eff.to_ascii_lowercase()) {
        cx.effect(num, eff);
    }
    let end = cx.code.len();
    patch(cx, over.unwrap_or(skip), end);
    true
}

fn patch(cx: &mut Cx, at: usize, target: usize) {
    if let Some(Instr::Jump(t) | Instr::JumpIfNot(_, t)) = cx.code.get_mut(at) {
        *t = target;
    }
}

// ---- parse if ----

fn split_word<'a>(toks: &'a [Tok], word: &str) -> Vec<&'a [Tok]> {
    let mut parts = Vec::new();
    let (mut depth, mut from) = (0i32, 0);
    for (i, t) in toks.iter().enumerate() {
        if t.is_lit("(") {
            depth += 1;
        } else if t.is_lit(")") {
            depth -= 1;
        } else if depth == 0 && t.is_lit(word) {
            parts.push(toks.get(from..i).unwrap_or_default());
            from = i + 1;
        }
    }
    parts.push(toks.get(from..).unwrap_or_default());
    parts
}

/// A condition on fixed values, decided while the file loads.
pub(crate) fn const_cond(text: &str, env: &Env) -> Result<bool, String> {
    const_toks(&lex(text)?, env)
}

fn const_toks(toks: &[Tok], env: &Env) -> Result<bool, String> {
    if toks.is_empty() {
        return Err("missing condition".into());
    }
    for any in split_word(toks, "or") {
        let mut all = true;
        for term in split_word(any, "and") {
            all &= const_term(term, env)?;
        }
        if all {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Does the `(` at the start close at the very last token?
fn closes_at_end(toks: &[Tok]) -> bool {
    let mut depth = 0i32;
    for (i, t) in toks.iter().enumerate() {
        if t.is_lit("(") {
            depth += 1;
        } else if t.is_lit(")") {
            depth -= 1;
            if depth == 0 {
                return i + 1 == toks.len();
            }
        }
    }
    false
}

fn const_value(toks: &[Tok], env: &Env) -> Result<Value, String> {
    let fixed = "use fixed values: text, numbers, true or false (options like {@name} are filled in first)";
    match parse_expr(toks, env).map_err(|e| format!("{e}; {fixed}"))? {
        Expr::Num(n) => Ok(Value::Num(n)),
        Expr::Bool(b) => Ok(Value::Bool(b)),
        Expr::Text(parts) => {
            let mut s = String::new();
            for p in parts {
                match p {
                    Part::Lit(l) => s.push_str(&l),
                    Part::Expr(_) => return Err(fixed.into()),
                }
            }
            Ok(Value::Text(s))
        }
        _ => Err(fixed.into()),
    }
}

fn const_term(toks: &[Tok], env: &Env) -> Result<bool, String> {
    if toks.first().is_some_and(|t| t.is_lit("(")) && closes_at_end(toks) {
        return const_toks(toks.get(1..toks.len() - 1).unwrap_or_default(), env);
    }
    if toks.first().is_some_and(|t| t.is_lit("not")) {
        return Ok(!const_term(toks.get(1..).unwrap_or_default(), env)?);
    }
    let cmp = toks.iter().position(|t| {
        t.is_lit("=")
            || t.is_lit("!=")
            || t.is_lit(">")
            || t.is_lit("<")
            || t.is_lit(">=")
            || t.is_lit("<=")
            || t.is_lit("is")
    });
    let Some(i) = cmp else {
        return match const_value(toks, env)? {
            Value::Bool(b) => Ok(b),
            _ => Err("this is not a yes/no condition".into()),
        };
    };
    let op = toks.get(i);
    let negated = op.is_some_and(|t| t.is_lit("is")) && toks.get(i + 1).is_some_and(|t| t.is_lit("not"));
    let skip = if negated { 2 } else { 1 };
    let left = const_value(toks.get(..i).unwrap_or_default(), env)?;
    let right = const_value(toks.get(i + skip..).unwrap_or_default(), env)?;
    let ord = |a: &Value, b: &Value| match (a, b) {
        (Value::Num(x), Value::Num(y)) => x.partial_cmp(y),
        _ => None,
    };
    let sym = |s: &str| op.is_some_and(|t| t.is_lit(s));
    if sym(">") || sym("<") || sym(">=") || sym("<=") {
        let o = ord(&left, &right).ok_or("comparing with > or < needs two numbers")?;
        return Ok(match () {
            () if sym(">") => o.is_gt(),
            () if sym("<") => o.is_lt(),
            () if sym(">=") => o.is_ge(),
            () => o.is_le(),
        });
    }
    let eq = left.loose_eq(&right);
    Ok(if negated || sym("!=") { !eq } else { eq })
}

// ---- the caught error ----

const PHRASES: [&[&str]; 9] = [
    &["last", "caught", "runtime", "error"],
    &["last", "caught", "error"],
    &["last", "runtime", "error"],
    &["last", "error"],
    &["caught", "runtime", "error"],
    &["caught", "error"],
    &["runtime", "error"],
    &["last", "runtime", "errors"],
    &["caught", "runtime", "errors"],
];

fn phrase_words(first: &str, rest: &[Tok], max: usize) -> Option<Vec<String>> {
    let mut words = vec![first.to_ascii_lowercase()];
    for t in rest.iter().take(max) {
        match t {
            Tok::Word(w) => words.push(w.to_ascii_lowercase()),
            _ => break,
        }
    }
    Some(words)
}

/// `last caught runtime error` and its shorter forms. `first` is the word
/// just read, `rest` what follows it. Gives how many tokens of `rest` belong.
pub fn error_phrase(first: &str, rest: &[Tok]) -> Option<usize> {
    let words = phrase_words(first, rest, 3)?;
    (2..=words.len()).rev().find_map(|n| {
        let cand: Vec<&str> = words.iter().take(n).map(String::as_str).collect();
        PHRASES.iter().any(|p| **p == *cand).then_some(n - 1)
    })
}

/// Are the tokens up to the end the start of such a phrase? Then a longer
/// span can still match, which the pattern matcher needs to know.
pub fn error_phrase_start(first: &str, rest: &[Tok]) -> bool {
    if rest.len() > 2 || rest.iter().any(|t| !matches!(t, Tok::Word(_))) {
        return false;
    }
    let Some(words) = phrase_words(first, rest, 2) else {
        return false;
    };
    PHRASES
        .iter()
        .any(|p| p.len() > words.len() && p.iter().zip(&words).all(|(a, b)| a == b))
}

/// The expression for the caught error: a local the catch section fills in.
pub fn caught_error_expr() -> Expr {
    Expr::Var(VarRef {
        key: Key::Static(ERROR_KEY.to_string()),
        local: true,
        list: false,
    })
}
