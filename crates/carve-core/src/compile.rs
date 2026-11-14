//! Compiles a script file into triggers holding flat instruction lists.

use std::sync::{Arc, OnceLock};

use crate::blocks::{Line, parse_blocks};
use crate::error::ScriptError;
use crate::expr::{Env, Expr, LoopKind, Ty, VarRef, parse_expr, parse_template, parse_varref};
use crate::lexer::{Tok, lex};
use crate::pattern::{Capture, Memo, Pattern, closest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    Join,
    Quit,
    Chat,
    Break,
    Place,
}

impl EventKind {
    pub fn name(self) -> &'static str {
        match self {
            EventKind::Join => "join",
            EventKind::Quit => "quit",
            EventKind::Chat => "chat",
            EventKind::Break => "break",
            EventKind::Place => "place",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "join" | "player join" | "login" => EventKind::Join,
            "quit" | "leave" | "disconnect" | "player quit" => EventKind::Quit,
            "chat" | "player chat" => EventKind::Chat,
            "break" | "block break" | "mine" => EventKind::Break,
            "place" | "block place" => EventKind::Place,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgTy {
    Text,
    Number,
    Player,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArgDef {
    pub ty: ArgTy,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandDef {
    pub name: String,
    pub permission: Option<String>,
    pub description: Option<String>,
    pub args: Vec<ArgDef>,
}

impl CommandDef {
    pub fn usage(&self) -> String {
        let mut s = format!("/{}", self.name);
        for a in &self.args {
            let t = match a.ty {
                ArgTy::Text => "text",
                ArgTy::Number => "number",
                ArgTy::Player => "player",
            };
            if a.optional {
                s.push_str(&format!(" [<{t}>]"));
            } else {
                s.push_str(&format!(" <{t}>"));
            }
        }
        s
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TriggerKind {
    Event(EventKind),
    Command(CommandDef),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cond {
    Cmp(Expr, CmpOp, Expr),
    HasPerm { player: Expr, node: Expr, negate: bool },
    IsSet(Expr, bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Send {
        text: Expr,
        to: Option<Expr>,
    },
    Broadcast(Expr),
    Title {
        title: Expr,
        subtitle: Option<Expr>,
        to: Expr,
    },
    ActionBar {
        text: Expr,
        to: Expr,
    },
    Heal(Expr),
    SetHealth(Expr, Expr),
    Kill(Expr),
    Teleport(Expr, Expr, Expr, Expr),
    ConsoleCommand(Expr),
    Cancel,
    /// amount times ticks-per-unit
    Wait(Expr, u64),
    Set(VarRef, Expr),
    Add(VarRef, Expr),
    Remove(VarRef, Expr),
    Delete(VarRef),
    /// `add x to {list::*}`: append at the next free number index.
    ListAdd(VarRef, Expr),
    /// `remove x from {list::*}`: remove every item equal to x.
    ListRemove(VarRef, Expr),
}

/// What a `loop` goes over.
#[derive(Debug, Clone, PartialEq)]
pub enum LoopSrc {
    Times(Expr),
    List(VarRef),
    Players,
    Values(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    Eff(Effect),
    /// Jump to target when the condition is false.
    JumpIfNot(Cond, usize),
    Jump(usize),
    /// Start a loop: take a snapshot of its items. Falls through to `LoopNext`.
    LoopInit(LoopSrc),
    /// Move to the next item, or end the loop and jump to the target.
    LoopNext(usize),
    /// `exit loop`: end the innermost loop and jump to the target.
    LoopBreak(usize),
    /// `stop`: end the whole run.
    Stop,
}

#[derive(Debug, Clone)]
pub struct Trigger {
    pub file: Arc<str>,
    pub line: usize,
    pub kind: TriggerKind,
    pub code: Vec<Instr>,
    /// Source line of each instruction.
    pub lines: Vec<usize>,
}

// ---- syntax registry ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffKind {
    Title,
    ActionBar,
    Send,
    Broadcast,
    Heal,
    SetHealth,
    Kill,
    Teleport,
    ConsoleCommand,
    Cancel,
    WaitTicks,
    WaitSeconds,
    Set,
    Add,
    Remove,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CondKind {
    IsSet,
    NotSet,
    Ne,
    Eq,
    Ge,
    Le,
    Gt,
    Lt,
    HasPerm,
    NotHasPerm,
}

/// Effect patterns, tried in order (more specific first).
pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    (
        "send title %text% [with subtitle %text%] to %player%",
        EffKind::Title,
    ),
    ("send action bar %text% to %player%", EffKind::ActionBar),
    ("send %text% [to %player%]", EffKind::Send),
    ("broadcast %text%", EffKind::Broadcast),
    ("heal %player%", EffKind::Heal),
    ("set health of %player% to %number%", EffKind::SetHealth),
    ("kill %player%", EffKind::Kill),
    (
        "teleport %player% to %number%, %number%, %number%",
        EffKind::Teleport,
    ),
    ("execute console command %text%", EffKind::ConsoleCommand),
    ("cancel [the] event", EffKind::Cancel),
    ("wait %number% (tick|ticks)", EffKind::WaitTicks),
    ("wait %number% (second|seconds)", EffKind::WaitSeconds),
    ("set %variable% to %object%", EffKind::Set),
    ("add %object% to %variable%", EffKind::Add),
    ("remove %object% from %variable%", EffKind::Remove),
    ("delete %variable%", EffKind::Delete),
];

pub const CONDITION_PATTERNS: &[(&str, CondKind)] = &[
    ("%object% is set", CondKind::IsSet),
    ("%object% is not set", CondKind::NotSet),
    ("%object% (is not|!=) %object%", CondKind::Ne),
    ("%object% (is|=) %object%", CondKind::Eq),
    ("%number% >= %number%", CondKind::Ge),
    ("%number% <= %number%", CondKind::Le),
    ("%number% > %number%", CondKind::Gt),
    ("%number% < %number%", CondKind::Lt),
    ("%player% has permission %text%", CondKind::HasPerm),
    ("%player% does not have permission %text%", CondKind::NotHasPerm),
];

struct Registry {
    effects: Vec<Pattern<EffKind>>,
    conditions: Vec<Pattern<CondKind>>,
}

fn registry() -> &'static Registry {
    static R: OnceLock<Registry> = OnceLock::new();
    R.get_or_init(|| Registry {
        // The pattern strings are constants covered by tests; a bad one is skipped.
        effects: EFFECT_PATTERNS
            .iter()
            .filter_map(|(s, k)| Pattern::new(s, *k).ok())
            .collect(),
        conditions: CONDITION_PATTERNS
            .iter()
            .filter_map(|(s, k)| Pattern::new(s, *k).ok())
            .collect(),
    })
}

/// Checks every registered pattern compiles. Used by tests.
pub fn check_registry() -> Result<(), String> {
    for (s, k) in EFFECT_PATTERNS {
        Pattern::new(s, *k)?;
    }
    for (s, k) in CONDITION_PATTERNS {
        Pattern::new(s, *k)?;
    }
    Ok(())
}

/// Normalized text for "did you mean" distance: values become `x`.
fn norm_line(toks: &[Tok]) -> String {
    toks.iter()
        .map(|t| match t {
            Tok::Word(w) => w.to_ascii_lowercase(),
            Tok::Sym(s) => s.to_string(),
            _ => "x".into(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn norm_pattern(src: &str) -> String {
    let mut out = String::new();
    let mut in_slot = false;
    for c in src.chars() {
        match c {
            '%' => {
                if !in_slot {
                    out.push('x');
                }
                in_slot = !in_slot;
            }
            _ if in_slot => {}
            '[' | ']' | '(' | ')' => {}
            '|' => out.push(' '),
            _ => out.push(c),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn suggest(toks: &[Tok], srcs: &[&'static str]) -> Vec<String> {
    let line = norm_line(toks);
    let normed: Vec<(String, &str)> = srcs.iter().map(|s| (norm_pattern(s), *s)).collect();
    let best = closest(&line, normed.iter().map(|(n, _)| n.as_str()), 3);
    best.iter()
        .filter_map(|b| normed.iter().find(|(n, _)| n == b).map(|(_, s)| s.to_string()))
        .collect()
}

// ---- compiler ----

/// A loop being compiled: where `continue` jumps and which jumps `exit loop` patches.
struct LoopCx {
    /// Has a runtime frame (every loop except `while`).
    frame: bool,
    next: usize,
    breaks: Vec<usize>,
}

struct Cx<'a> {
    file: &'a str,
    env: Env,
    is_command: bool,
    code: Vec<Instr>,
    lines: Vec<usize>,
    errors: Vec<ScriptError>,
    loops: Vec<LoopCx>,
}

impl Cx<'_> {
    fn err(&mut self, line: usize, msg: impl Into<String>) {
        self.errors.push(ScriptError::new(self.file, line, msg));
    }

    fn emit(&mut self, i: Instr, line: usize) -> usize {
        self.code.push(i);
        self.lines.push(line);
        self.code.len() - 1
    }

    fn block(&mut self, lines: &[Line]) {
        let mut i = 0;
        while i < lines.len() {
            let line = &lines[i];
            let lower = line.text.to_ascii_lowercase();
            if lower.starts_with("if ") && line.opens_block() {
                let mut ends = Vec::new();
                let mut cur = line;
                let mut cond_text = line.head().get(3..).unwrap_or("");
                loop {
                    let jif = self
                        .cond(cur.num, cond_text)
                        .map(|c| self.emit(Instr::JumpIfNot(c, usize::MAX), cur.num));
                    self.block(&cur.children);
                    let next = lines.get(i + 1);
                    let next_lower = next.map(|l| l.text.to_ascii_lowercase()).unwrap_or_default();
                    let has_more = next.is_some_and(|l| l.opens_block())
                        && (next_lower.starts_with("else if ") || next_lower == "else:");
                    if has_more {
                        ends.push(self.emit(Instr::Jump(usize::MAX), cur.num));
                    }
                    let here = self.code.len();
                    if let Some(j) = jif
                        && let Some(Instr::JumpIfNot(_, t)) = self.code.get_mut(j)
                    {
                        *t = here;
                    }
                    if !has_more {
                        break;
                    }
                    i += 1;
                    let Some(n) = lines.get(i) else { break };
                    if next_lower == "else:" {
                        self.block(&n.children);
                        break;
                    }
                    cur = n;
                    cond_text = n.head().get(8..).unwrap_or("");
                }
                let end = self.code.len();
                for e in ends {
                    if let Some(Instr::Jump(t)) = self.code.get_mut(e) {
                        *t = end;
                    }
                }
                i += 1;
                continue;
            }
            if (lower.starts_with("loop ") || lower.starts_with("while ")) && line.opens_block() {
                self.loop_block(line);
                i += 1;
                continue;
            }
            if lower.starts_with("else") {
                self.err(line.num, "'else' without an 'if' before it");
                i += 1;
                continue;
            }
            if line.opens_block() {
                self.err(
                    line.num,
                    format!(
                        "unknown section '{}' (only 'if', 'else if' and 'else' open blocks here)",
                        line.head()
                    ),
                );
                i += 1;
                continue;
            }
            if self.control(line.num, &lower) {
                i += 1;
                continue;
            }
            self.effect(line.num, &line.text);
            i += 1;
        }
    }

    /// `stop`, `stop loop` / `exit loop`, `continue`. Returns true if handled.
    fn control(&mut self, num: usize, lower: &str) -> bool {
        let words: Vec<&str> = lower.split_whitespace().collect();
        match words.as_slice() {
            ["stop"] | ["stop", "trigger"] | ["exit"] => {
                self.emit(Instr::Stop, num);
            }
            ["stop", "loop"] | ["exit", "loop"] | ["exit", "this", "loop"] | ["break"] => {
                let Some(lp) = self.loops.last() else {
                    self.err(num, "'exit loop' only works inside a loop");
                    return true;
                };
                let instr = if lp.frame {
                    Instr::LoopBreak(usize::MAX)
                } else {
                    Instr::Jump(usize::MAX)
                };
                let at = self.emit(instr, num);
                if let Some(lp) = self.loops.last_mut() {
                    lp.breaks.push(at);
                }
            }
            ["continue"] | ["continue", "loop"] => {
                let Some(next) = self.loops.last().map(|l| l.next) else {
                    self.err(num, "'continue' only works inside a loop");
                    return true;
                };
                self.emit(Instr::Jump(next), num);
            }
            _ => return false,
        }
        true
    }

    fn loop_block(&mut self, line: &Line) {
        let head = line.head();
        let lower = head.to_ascii_lowercase();
        if let Some(cond_text) = head.get(6..).filter(|_| lower.starts_with("while ")) {
            let start = self.code.len();
            let jif = self
                .cond(line.num, cond_text)
                .map(|c| self.emit(Instr::JumpIfNot(c, usize::MAX), line.num));
            self.loops.push(LoopCx {
                frame: false,
                next: start,
                breaks: Vec::new(),
            });
            self.block(&line.children);
            self.emit(Instr::Jump(start), line.num);
            let end = self.code.len();
            if let Some(j) = jif
                && let Some(Instr::JumpIfNot(_, t)) = self.code.get_mut(j)
            {
                *t = end;
            }
            self.finish_loop(end);
            return;
        }
        let rest = head.get(5..).unwrap_or("").trim();
        let Some((src, kind)) = self.loop_source(line.num, rest) else {
            return;
        };
        self.emit(Instr::LoopInit(src), line.num);
        let next = self.emit(Instr::LoopNext(usize::MAX), line.num);
        self.loops.push(LoopCx {
            frame: true,
            next,
            breaks: Vec::new(),
        });
        self.env.loops.push(kind);
        self.block(&line.children);
        self.env.loops.pop();
        self.emit(Instr::Jump(next), line.num);
        let end = self.code.len();
        if let Some(Instr::LoopNext(t)) = self.code.get_mut(next) {
            *t = end;
        }
        self.finish_loop(end);
    }

    fn finish_loop(&mut self, end: usize) {
        if let Some(lp) = self.loops.pop() {
            for b in lp.breaks {
                if let Some(Instr::LoopBreak(t) | Instr::Jump(t)) = self.code.get_mut(b) {
                    *t = end;
                }
            }
        }
    }

    fn loop_source(&mut self, num: usize, rest: &str) -> Option<(LoopSrc, LoopKind)> {
        let lower = rest.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "all players" | "all online players" | "players" | "online players"
        ) {
            return Some((LoopSrc::Players, LoopKind::Players));
        }
        let toks = match lex(rest) {
            Ok(t) => t,
            Err(e) => {
                self.err(num, e);
                return None;
            }
        };
        if let Some((last, body)) = toks.split_last()
            && last.is_lit("times")
        {
            return match parse_expr(body, &self.env) {
                Ok(e) if Ty::Number.accepts(e.ty(&self.env)) => Some((LoopSrc::Times(e), LoopKind::Times)),
                Ok(e) => {
                    self.err(
                        num,
                        format!("'loop ... times' needs a number, got {}", e.ty(&self.env).name()),
                    );
                    None
                }
                Err(e) => {
                    self.err(num, format!("can't understand the loop count: {e}"));
                    None
                }
            };
        }
        if let [Tok::Var(raw)] = toks.as_slice() {
            return match parse_varref(raw, &self.env) {
                Ok(v) if v.is_list() => Some((LoopSrc::List(v), LoopKind::List)),
                Ok(_) => {
                    self.err(num, format!("'loop {rest}' needs a list variable like {{x::*}}"));
                    None
                }
                Err(e) => {
                    self.err(num, e);
                    None
                }
            };
        }
        // `loop 1, 2 and 3:` / `loop "a" and "b":`
        let mut items = Vec::new();
        for part in toks.split(|t| t.is_lit(",") || t.is_lit("and")) {
            match parse_expr(part, &self.env) {
                Ok(e) => items.push(e),
                Err(_) => {
                    self.err(
                        num,
                        format!("can't understand 'loop {rest}' (use 'loop N times', 'loop {{x::*}}', 'loop all players' or 'loop a, b and c')"),
                    );
                    return None;
                }
            }
        }
        Some((LoopSrc::Values(items), LoopKind::Values))
    }

    fn cond(&mut self, num: usize, text: &str) -> Option<Cond> {
        let toks = match lex(text) {
            Ok(t) => t,
            Err(e) => {
                self.err(num, e);
                return None;
            }
        };
        let mut type_err = None;
        let mut memo = Memo::default();
        for p in &registry().conditions {
            match p.matches_memo(&toks, &self.env, &mut memo) {
                Ok(caps) => return Some(build_cond(p.kind, caps)),
                Err(Some(e)) if type_err.is_none() => type_err = Some(e),
                Err(_) => {}
            }
        }
        // A broken template or variable explains the failure best.
        let type_err = inner_error(&toks, &self.env).or(type_err);
        let srcs: Vec<&'static str> = CONDITION_PATTERNS.iter().map(|(s, _)| *s).collect();
        let mut e = ScriptError::new(
            self.file,
            num,
            match type_err {
                Some(t) => format!("can't understand this condition: '{text}' ({t})"),
                None => format!("can't understand this condition: '{text}'"),
            },
        );
        e.suggestions = suggest(&toks, &srcs);
        self.errors.push(e);
        None
    }

    fn effect(&mut self, num: usize, text: &str) {
        let toks = match lex(text) {
            Ok(t) => t,
            Err(e) => {
                self.err(num, e);
                return;
            }
        };
        let mut type_err = None;
        let mut memo = Memo::default();
        for p in &registry().effects {
            match p.matches_memo(&toks, &self.env, &mut memo) {
                Ok(caps) => {
                    if p.kind == EffKind::Cancel && self.is_command {
                        self.err(num, "commands have no event to cancel");
                        return;
                    }
                    if matches!(p.kind, EffKind::Add | EffKind::Remove) {
                        let not_list = matches!(caps.get(1), Some(Some(Capture::Var(v))) if !v.is_list());
                        let ty = match caps.first() {
                            Some(Some(Capture::Expr(e))) => e.ty(&self.env),
                            _ => Ty::Object,
                        };
                        if not_list && !Ty::Number.accepts(ty) {
                            self.err(num, format!("can't add a {} to a single variable (only numbers; use a list like {{x::*}} for other values)", ty.name()));
                            return;
                        }
                    }
                    if let Some(eff) = build_effect(p.kind, caps) {
                        self.emit(Instr::Eff(eff), num);
                    }
                    return;
                }
                Err(Some(e)) if type_err.is_none() => type_err = Some(e),
                Err(_) => {}
            }
        }
        let type_err = inner_error(&toks, &self.env).or(type_err);
        let srcs: Vec<&'static str> = EFFECT_PATTERNS.iter().map(|(s, _)| *s).collect();
        let mut e = ScriptError::new(
            self.file,
            num,
            match type_err {
                Some(t) => format!("can't understand this effect: '{text}' ({t})"),
                None => format!("can't understand this effect: '{text}'"),
            },
        );
        e.suggestions = suggest(&toks, &srcs);
        self.errors.push(e);
    }
}

/// A broken text template or variable name inside the line, if any.
fn inner_error(toks: &[Tok], env: &Env) -> Option<String> {
    toks.iter().find_map(|t| match t {
        Tok::Str(s) | Tok::Var(s) => parse_template(s, env).err(),
        _ => None,
    })
}

fn take_expr(caps: &mut [Option<Capture>], i: usize) -> Option<Expr> {
    match caps.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    }
}

fn take_var(caps: &mut [Option<Capture>], i: usize) -> Option<VarRef> {
    match caps.get_mut(i).and_then(Option::take) {
        Some(Capture::Var(v)) => Some(v),
        _ => None,
    }
}

fn build_cond(kind: CondKind, mut c: Vec<Option<Capture>>) -> Cond {
    let a = take_expr(&mut c, 0).unwrap_or(Expr::Num(0.0));
    let b = take_expr(&mut c, 1).unwrap_or(Expr::Num(0.0));
    let op = match kind {
        CondKind::IsSet => return Cond::IsSet(a, true),
        CondKind::NotSet => return Cond::IsSet(a, false),
        CondKind::Ne => CmpOp::Ne,
        CondKind::Eq => CmpOp::Eq,
        CondKind::Ge => CmpOp::Ge,
        CondKind::Le => CmpOp::Le,
        CondKind::Gt => CmpOp::Gt,
        CondKind::Lt => CmpOp::Lt,
        CondKind::HasPerm => {
            return Cond::HasPerm {
                player: a,
                node: b,
                negate: false,
            };
        }
        CondKind::NotHasPerm => {
            return Cond::HasPerm {
                player: a,
                node: b,
                negate: true,
            };
        }
    };
    Cond::Cmp(a, op, b)
}

fn build_effect(kind: EffKind, mut c: Vec<Option<Capture>>) -> Option<Effect> {
    let c = &mut c;
    Some(match kind {
        EffKind::Title => Effect::Title {
            title: take_expr(c, 0)?,
            subtitle: take_expr(c, 1),
            to: take_expr(c, 2)?,
        },
        EffKind::ActionBar => Effect::ActionBar {
            text: take_expr(c, 0)?,
            to: take_expr(c, 1)?,
        },
        EffKind::Send => Effect::Send {
            text: take_expr(c, 0)?,
            to: take_expr(c, 1),
        },
        EffKind::Broadcast => Effect::Broadcast(take_expr(c, 0)?),
        EffKind::Heal => Effect::Heal(take_expr(c, 0)?),
        EffKind::SetHealth => Effect::SetHealth(take_expr(c, 0)?, take_expr(c, 1)?),
        EffKind::Kill => Effect::Kill(take_expr(c, 0)?),
        EffKind::Teleport => Effect::Teleport(
            take_expr(c, 0)?,
            take_expr(c, 1)?,
            take_expr(c, 2)?,
            take_expr(c, 3)?,
        ),
        EffKind::ConsoleCommand => Effect::ConsoleCommand(take_expr(c, 0)?),
        EffKind::Cancel => Effect::Cancel,
        EffKind::WaitTicks => Effect::Wait(take_expr(c, 0)?, 1),
        EffKind::WaitSeconds => Effect::Wait(take_expr(c, 0)?, 20),
        EffKind::Set => Effect::Set(take_var(c, 0)?, take_expr(c, 1)?),
        EffKind::Add => {
            let v = take_var(c, 1)?;
            let e = take_expr(c, 0)?;
            if v.is_list() {
                Effect::ListAdd(v, e)
            } else {
                Effect::Add(v, e)
            }
        }
        EffKind::Remove => {
            let v = take_var(c, 1)?;
            let e = take_expr(c, 0)?;
            if v.is_list() {
                Effect::ListRemove(v, e)
            } else {
                Effect::Remove(v, e)
            }
        }
        EffKind::Delete => Effect::Delete(take_var(c, 0)?),
    })
}

fn parse_command_header(file: &str, num: usize, head: &str) -> Result<CommandDef, ScriptError> {
    let rest = head.get("command".len()..).unwrap_or("").trim();
    let mut parts = rest.split_whitespace();
    let name = parts.next().unwrap_or("");
    let name = name.strip_prefix('/').unwrap_or(name).to_ascii_lowercase();
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ScriptError::new(
            file,
            num,
            "expected 'command /name:' with a simple name",
        ));
    }
    let mut args = Vec::new();
    for p in parts {
        let (inner, optional) = match p.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            Some(s) => (s, true),
            None => (p, false),
        };
        let ty = match inner {
            "<text>" | "<string>" => ArgTy::Text,
            "<number>" | "<integer>" => ArgTy::Number,
            "<player>" => ArgTy::Player,
            _ => {
                return Err(ScriptError::new(
                    file,
                    num,
                    format!(
                        "unknown argument '{p}', use <text>, <number>, <player> or [<text>] for optional"
                    ),
                ));
            }
        };
        if !optional && args.iter().any(|a: &ArgDef| a.optional) {
            return Err(ScriptError::new(
                file,
                num,
                "required arguments can't come after optional ones",
            ));
        }
        args.push(ArgDef { ty, optional });
    }
    Ok(CommandDef {
        name,
        permission: None,
        description: None,
        args,
    })
}

pub fn compile_file(file: &str, src: &str) -> Result<Vec<Trigger>, Vec<ScriptError>> {
    let tree = parse_blocks(file, src)?;
    let file_arc: Arc<str> = Arc::from(file);
    let mut triggers = Vec::new();
    let mut errors = Vec::new();
    for top in &tree {
        let lower = top.head().to_ascii_lowercase();
        if !top.opens_block() {
            errors.push(ScriptError::new(
                file,
                top.num,
                format!(
                    "expected a trigger like 'on join:' or 'command /name:', got '{}'",
                    top.text
                ),
            ));
            continue;
        }
        let (kind, body, env, is_command): (TriggerKind, &[Line], Env, bool) =
            if let Some(ev) = lower.strip_prefix("on ") {
                let Some(k) = EventKind::parse(ev.trim()) else {
                    errors.push(ScriptError::new(
                        file,
                        top.num,
                        format!(
                            "unknown event '{}', known: join, quit, chat, break, place",
                            ev.trim()
                        ),
                    ));
                    continue;
                };
                let env = Env {
                    has_message: k == EventKind::Chat,
                    arg_types: vec![],
                    trigger_name: format!("'on {}'", k.name()),
                    ..Env::default()
                };
                (TriggerKind::Event(k), &top.children, env, false)
            } else if lower.starts_with("command") {
                let mut def = match parse_command_header(file, top.num, top.head()) {
                    Ok(d) => d,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                };
                let mut body: Option<&[Line]> = None;
                for entry in &top.children {
                    let el = entry.text.to_ascii_lowercase();
                    if el == "trigger:" {
                        body = Some(&entry.children);
                    } else if el.starts_with("permission:") {
                        let v = entry.text.get("permission:".len()..).unwrap_or("").trim();
                        def.permission = Some(v.to_string()).filter(|s| !s.is_empty());
                    } else if el.starts_with("description:") {
                        def.description = Some(
                            entry
                                .text
                                .get("description:".len()..)
                                .unwrap_or("")
                                .trim()
                                .to_string(),
                        );
                    } else {
                        errors.push(ScriptError::new(
                            file,
                            entry.num,
                            format!(
                                "unknown command entry '{}', use permission:, description: or trigger:",
                                entry.text
                            ),
                        ));
                    }
                }
                let Some(body) = body else {
                    errors.push(ScriptError::new(
                        file,
                        top.num,
                        "command has no 'trigger:' section",
                    ));
                    continue;
                };
                let arg_types = def
                    .args
                    .iter()
                    .map(|a| match a.ty {
                        ArgTy::Text => Ty::Text,
                        ArgTy::Number => Ty::Number,
                        ArgTy::Player => Ty::Player,
                    })
                    .collect();
                let env = Env {
                    has_message: false,
                    arg_types,
                    trigger_name: format!("command /{}", def.name),
                    ..Env::default()
                };
                (TriggerKind::Command(def), body, env, true)
            } else {
                errors.push(ScriptError::new(
                    file,
                    top.num,
                    format!(
                        "unknown trigger '{}', expected 'on <event>:' or 'command /name:'",
                        top.head()
                    ),
                ));
                continue;
            };
        let mut cx = Cx {
            file,
            env,
            is_command,
            code: Vec::new(),
            lines: Vec::new(),
            errors: Vec::new(),
            loops: Vec::new(),
        };
        cx.block(body);
        if cx.errors.is_empty() {
            triggers.push(Trigger {
                file: file_arc.clone(),
                line: top.num,
                kind,
                code: cx.code,
                lines: cx.lines,
            });
        } else {
            errors.extend(cx.errors);
        }
    }
    if errors.is_empty() {
        Ok(triggers)
    } else {
        Err(errors)
    }
}
