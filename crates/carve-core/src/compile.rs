//! Compiles a script file into triggers holding flat instruction lists.

use std::sync::{Arc, OnceLock};

use crate::blocks::{Line, parse_blocks};
use crate::error::ScriptError;
use crate::expr::{Env, Expr, LoopKind, Sig, Ty, VarRef, parse_expr, parse_template, parse_varref};
use crate::lexer::{Tok, lex};
use crate::list_ops::ListSrc;
use crate::op_kick_ban::{AdminCond, AdminEff};
use crate::pattern::{Capture, Memo, Pattern, closest};
use crate::player_flags::{PLAYER_FLAG_PATTERNS, PlayerFlag};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    Join,
    Quit,
    Chat,
    Break,
    Place,
    Death,
    Damage,
    Respawn,
    /// Any click (left or right).
    Click,
    RightClick,
    LeftClick,
    /// After scripts load or reload.
    Load,
    SneakToggle,
    SprintToggle,
    FlightToggle,
}

impl EventKind {
    pub fn name(self) -> &'static str {
        match self {
            EventKind::Join => "join",
            EventKind::Quit => "quit",
            EventKind::Chat => "chat",
            EventKind::Break => "break",
            EventKind::Place => "place",
            EventKind::Death => "death",
            EventKind::Damage => "damage",
            EventKind::Respawn => "respawn",
            EventKind::Click => "click",
            EventKind::RightClick => "right click",
            EventKind::LeftClick => "left click",
            EventKind::Load => "load",
            EventKind::SneakToggle => "sneak toggle",
            EventKind::SprintToggle => "sprint toggle",
            EventKind::FlightToggle => "flight toggle",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "join" | "player join" | "login" => EventKind::Join,
            "quit" | "leave" | "disconnect" | "player quit" => EventKind::Quit,
            "chat" | "player chat" => EventKind::Chat,
            "break" | "block break" | "mine" => EventKind::Break,
            "place" | "block place" => EventKind::Place,
            "death" | "death of player" | "player death" => EventKind::Death,
            "damage" | "damage of player" | "player damage" => EventKind::Damage,
            "respawn" | "player respawn" => EventKind::Respawn,
            "click" | "rightclick" | "right click" => {
                if s == "click" {
                    EventKind::Click
                } else {
                    EventKind::RightClick
                }
            }
            "leftclick" | "left click" => EventKind::LeftClick,
            "load" | "script load" | "server start" | "skript load" => EventKind::Load,
            "sneak toggle" | "toggle sneak" | "player toggle sneak" => EventKind::SneakToggle,
            "sprint toggle" | "toggle sprint" | "player toggle sprint" => EventKind::SprintToggle,
            "flight toggle" | "toggle flight" | "player toggle flight" => EventKind::FlightToggle,
            _ => return None,
        })
    }

    /// Whether `cancel event` means anything here.
    pub fn cancellable(self) -> bool {
        !matches!(self, EventKind::Respawn | EventKind::Load)
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
    /// Other names for the command (`aliases: /h, /hi`).
    pub aliases: Vec<String>,
    /// Message for wrong arguments instead of the generated usage.
    pub usage_msg: Option<String>,
    /// `executable by: players` (true) or `console` (false).
    pub players_only: Option<bool>,
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
    /// `every 5 seconds:`, period in ticks.
    Every(u64),
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
    PlayerFlag {
        player: Expr,
        flag: PlayerFlag,
        negate: bool,
    },
    Cmp(Expr, CmpOp, Expr),
    HasPerm {
        player: Expr,
        node: Expr,
        negate: bool,
    },
    IsSet(Expr, bool),
    /// `chance of 25%`
    Chance(Expr),
    /// `player has 2 diamonds`, `player's inventory contains stone`.
    HasItem {
        player: Expr,
        item: Expr,
        negate: bool,
    },
    /// Every part is true (`a and b`).
    All(Vec<Cond>),
    /// At least one part is true (`a or b`).
    Any(Vec<Cond>),
    /// `x is a or b`: equal to any of the values (negated: equal to none).
    AnyOf {
        value: Expr,
        options: Vec<Expr>,
        negate: bool,
    },
    Text {
        left: Expr,
        op: TextOp,
        right: Expr,
        negate: bool,
    },
    Between {
        value: Expr,
        low: Expr,
        high: Expr,
        negate: bool,
    },
    /// `sender is a player` / `sender is console` (and the negations).
    SenderIs {
        value: Expr,
        console: bool,
        negate: bool,
    },
    /// `player is op`, `player is banned`.
    Admin(AdminCond, Expr, bool),
    /// `player has potion effect speed`.
    World(Box<crate::world_effects::WorldCond>),
    /// `block at loc is stone`.
    Block(Box<crate::block_ops::BlockCond>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOp {
    Contains,
    StartsWith,
    EndsWith,
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
    Hf(Box<crate::health_food::HfEffect>),
    SetHealth(Expr, Expr),
    SetGamemode(Expr, Expr),
    /// Experience change; the two expressions are in pattern order.
    Xp(crate::xp_level::XpOp, Expr, Expr),
    Kill(Expr),
    Teleport(Expr, Expr, Expr, Expr),
    TeleportTo(Expr, Expr),
    /// player, amount (None: the item's own amount), item
    Give(Expr, Option<Expr>, Expr),
    ConsoleCommand(Expr),
    /// who, command (`make player execute command "/x"`)
    ExecAs(Expr, Expr),
    Cancel,
    /// amount times ticks-per-unit
    Wait(Expr, u64),
    Set(VarRef, Expr),
    /// `set {x::*} to split <text> at <sep>`.
    SetSplit(VarRef, Expr, Expr),
    /// `replace all <from> in {var} with <to>`: var, from, to, all.
    Replace(VarRef, Expr, Expr, bool),
    Add(VarRef, Expr),
    Remove(VarRef, Expr),
    Delete(VarRef),
    /// `add x to {list::*}`: append at the next free number index.
    ListAdd(VarRef, Expr),
    /// `remove x from {list::*}`: remove every item equal to x.
    ListRemove(VarRef, Expr),
    PlaySound {
        sound: Expr,
        volume: Option<Expr>,
        pitch: Option<Expr>,
        to: Option<Expr>,
        at: Option<Expr>,
    },
    /// sound (None: all sounds), player (None: the event player)
    StopSound(Option<Expr>, Option<Expr>),
    /// `set {x::*} to a, b and c`.
    ListSet(VarRef, Vec<Expr>),
    /// `add a, b and c to {x::*}`.
    ListAddAll(VarRef, Vec<Expr>),
    /// `remove a, b from {x::*}`.
    ListRemoveAll(VarRef, Vec<Expr>),
    /// `set {x::*} to sorted {y::*}` and other list expressions.
    SetList(VarRef, ListSrc),
    /// `sort {x::*}` (true: descending).
    SortList(VarRef, bool),
    /// op, kick, ban, names: target and an optional text (reason or name).
    Admin(AdminEff, Expr, Option<Expr>),
    /// Lightning, explosions, particles, potion effects, fire.
    World(Box<crate::world_effects::WorldFx>),
    /// `set block at loc to stone`, `break block at loc`.
    Block(Box<crate::block_ops::BlockFx>),
}

/// What a `loop` goes over.
#[derive(Debug, Clone, PartialEq)]
pub enum LoopSrc {
    Times(Expr),
    /// `loop split <text> at <sep>`.
    Split(Expr, Expr),
    List(VarRef),
    /// `loop sorted {x::*}:` and other list expressions.
    Items(ListSrc),
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
    /// `stop`: end the trigger, or return from a function.
    Stop,
    /// A function call used as a statement (its result is dropped).
    Call(usize, Vec<Expr>),
    /// `return [value]` from a function.
    Return(Option<Expr>),
    /// A condition on its own line: stop here when it is false (Skript style).
    StopUnless(Cond),
}

/// A compiled body: a trigger's or a function's instructions.
#[derive(Debug, Clone)]
pub struct Code {
    pub file: Arc<str>,
    pub line: usize,
    pub instrs: Vec<Instr>,
    /// Source line of each instruction.
    pub lines: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Trigger {
    pub file: Arc<str>,
    pub line: usize,
    pub kind: TriggerKind,
    pub code: Arc<Code>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub sig: Sig,
    /// Default value of each parameter, if any.
    pub defaults: Vec<Option<Expr>>,
    pub code: Arc<Code>,
}

/// Everything compiled from a set of files.
#[derive(Debug, Default)]
pub struct Compiled {
    /// Triggers per file that compiled without errors.
    pub files: Vec<(String, Vec<Trigger>)>,
    /// By function id; `None` when the function's body had errors.
    pub functions: Vec<Option<Arc<Function>>>,
    pub errors: Vec<ScriptError>,
}

// ---- syntax registry ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffKind {
    Title,
    ActionBar,
    Send,
    Broadcast,
    Heal,
    Hf(crate::health_food::HfOp),
    SetHealth,
    SetGamemode,
    Xp(crate::xp_level::XpOp),
    Kill,
    Teleport,
    TeleportTo,
    Give,
    GiveCount,
    GiveTo,
    ConsoleCommand,
    ExecAs,
    ExecAsBy,
    Cancel,
    WaitTicks,
    WaitSeconds,
    Set,
    ReplaceAllIn,
    ReplaceAllWith,
    ReplaceFirstIn,
    ReplaceFirstWith,
    Add,
    Remove,
    Delete,
    PlaySound,
    StopSound,
    StopAllSounds,
    Admin(AdminEff),
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
    InGamemode,
    NotInGamemode,
    Contains,
    NotContains,
    StartsWith,
    NotStartsWith,
    EndsWith,
    NotEndsWith,
    Between,
    NotBetween,
    HasItem,
    NotHasItem,
    InvContains,
    IsPlayer,
    IsNotPlayer,
    IsConsole,
    IsNotConsole,
    /// `player is sneaking`; the bool is negation.
    Flag(PlayerFlag, bool),
    /// negated
    Admin(AdminCond, bool),
}

/// Text effects (`replace ...`). Kept apart from `EFFECT_PATTERNS` so the
/// text slice stays self-contained; tried after the main list.
pub const TEXT_EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    (
        "replace first %text% in %variable% with %text%",
        EffKind::ReplaceFirstIn,
    ),
    (
        "replace first %text% with %text% in %variable%",
        EffKind::ReplaceFirstWith,
    ),
    (
        "replace [all] %text% in %variable% with %text%",
        EffKind::ReplaceAllIn,
    ),
    (
        "replace [all] %text% with %text% in %variable%",
        EffKind::ReplaceAllWith,
    ),
];

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
    ("teleport %player% to %location%", EffKind::TeleportTo),
    ("give %number% [of] %item% to %player%", EffKind::GiveCount),
    ("give %item% to %player%", EffKind::GiveTo),
    ("give %player% %item%", EffKind::Give),
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
    ("%player%'s inventory contains %item%", CondKind::InvContains),
    ("inventory of %player% contains %item%", CondKind::InvContains),
    ("%player% has %item%", CondKind::HasItem),
    ("%player% (does not|doesn't) have %item%", CondKind::NotHasItem),
    ("%number% is between %number% and %number%", CondKind::Between),
    (
        "%number% is not between %number% and %number%",
        CondKind::NotBetween,
    ),
    ("%text% contains %text%", CondKind::Contains),
    ("%text% (does not|doesn't) contain %text%", CondKind::NotContains),
    ("%text% starts with %text%", CondKind::StartsWith),
    (
        "%text% (does not|doesn't) start with %text%",
        CondKind::NotStartsWith,
    ),
    ("%text% ends with %text%", CondKind::EndsWith),
    ("%text% (does not|doesn't) end with %text%", CondKind::NotEndsWith),
];

struct Registry {
    effects: Vec<Pattern<EffKind>>,
    conditions: Vec<Pattern<CondKind>>,
}

fn registry() -> &'static Registry {
    static R: OnceLock<Registry> = OnceLock::new();
    R.get_or_init(|| Registry {
        // The pattern strings are constants covered by tests; a bad one is skipped.
        effects: crate::op_kick_ban::EFFECT_PATTERNS
            .iter()
            .filter_map(|(s, k)| Pattern::new(s, *k).ok())
            .chain(crate::xp_level::XP_EFFECT_PATTERNS
            .iter()
            .filter_map(|(s, o)| Pattern::new(s, EffKind::Xp(*o)).ok()))
            .chain(
                crate::senders::SENDER_EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::gamemode::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::sounds::SOUND_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                TEXT_EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::health_food::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .collect(),
        conditions: crate::op_kick_ban::CONDITION_PATTERNS
            .iter()
            .chain(crate::senders::SENDER_CONDITION_PATTERNS)
            .chain(crate::gamemode::CONDITION_PATTERNS)
            .chain(PLAYER_FLAG_PATTERNS)
            .chain(CONDITION_PATTERNS)
            .filter_map(|(s, k)| Pattern::new(s, *k).ok())
            .collect(),
    })
}

/// Checks every registered pattern compiles. Used by tests.
pub fn check_registry() -> Result<(), String> {
    for (s, k) in crate::op_kick_ban::EFFECT_PATTERNS
        .iter()
        .chain(crate::senders::SENDER_EFFECT_PATTERNS)
        .chain(crate::gamemode::EFFECT_PATTERNS)
        .chain(EFFECT_PATTERNS)
        .chain(crate::sounds::SOUND_PATTERNS)
        .chain(TEXT_EFFECT_PATTERNS)
        .chain(crate::health_food::EFFECT_PATTERNS)
    {
        Pattern::new(s, *k)?;
    }
    for (s, o) in crate::xp_level::XP_EFFECT_PATTERNS {
        Pattern::new(s, EffKind::Xp(*o))?;
    }
    for (s, k) in crate::op_kick_ban::CONDITION_PATTERNS
        .iter()
        .chain(crate::senders::SENDER_CONDITION_PATTERNS)
        .chain(crate::gamemode::CONDITION_PATTERNS)
        .chain(PLAYER_FLAG_PATTERNS)
        .chain(CONDITION_PATTERNS)
    {
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
    /// Return type of the function being compiled (`Some(None)`: returns nothing).
    func_ret: Option<Option<Ty>>,
    /// Why `cancel event` is not allowed here, if it isn't.
    no_cancel: Option<String>,
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
        // Most lines are effects: check the first word before collecting all.
        let first = lower.split_whitespace().next().unwrap_or("");
        if !matches!(first, "stop" | "exit" | "break" | "continue" | "return") {
            return false;
        }
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
            ["return", ..] => {
                self.return_stmt(num, lower);
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

    fn return_stmt(&mut self, num: usize, lower: &str) {
        let Some(ret) = self.func_ret else {
            self.err(
                num,
                "'return' only works inside a function (use 'stop' to end a trigger)",
            );
            return;
        };
        let rest = lower.get(6..).unwrap_or("").trim();
        if rest.is_empty() {
            if ret.is_some() {
                self.err(num, "this function must return a value");
            }
            self.emit(Instr::Return(None), num);
            return;
        }
        let Some(want) = ret else {
            self.err(
                num,
                "this function has no return type; add ':: <type>' to its first line",
            );
            return;
        };
        let toks = match lex(rest) {
            Ok(t) => t,
            Err(e) => {
                self.err(num, e);
                return;
            }
        };
        if want == Ty::List {
            match crate::list_ops::parse_src_all(&toks, &self.env, true) {
                Ok(s) => {
                    self.emit(Instr::Return(Some(Expr::Items(Box::new(s)))), num);
                }
                Err(e) => self.err(num, format!("can't understand the returned list: {e}")),
            }
            return;
        }
        match parse_expr(&toks, &self.env) {
            Ok(e) if want.accepts(e.ty(&self.env)) => {
                self.emit(Instr::Return(Some(e)), num);
            }
            Ok(e) => self.err(
                num,
                format!(
                    "this function returns a {}, not a {}",
                    want.name(),
                    e.ty(&self.env).name()
                ),
            ),
            Err(e) => self.err(num, format!("can't understand the returned value: {e}")),
        }
    }

    /// A line that is just `name(args)`.
    fn call_stmt(&mut self, num: usize, toks: &[Tok]) -> bool {
        let is_call = match toks {
            [Tok::Word(w), Tok::Sym("("), .., Tok::Sym(")")] => {
                self.env.functions.iter().any(|f| f.name.eq_ignore_ascii_case(w))
            }
            _ => false,
        };
        if !is_call {
            return false;
        }
        match parse_expr(toks, &self.env) {
            Ok(Expr::Call(id, args)) => {
                self.emit(Instr::Call(id, args), num);
            }
            Ok(_) => return false,
            Err(e) => self.err(num, e),
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
        if toks.first().is_some_and(|t| t.is_lit("split")) {
            return match parse_expr(&toks, &self.env) {
                Ok(Expr::Split(a, b)) => Some((LoopSrc::Split(*a, *b), LoopKind::Values)),
                Ok(_) => {
                    self.err(num, "can't understand 'loop split ...'".to_string());
                    None
                }
                Err(e) => {
                    self.err(num, e);
                    None
                }
            };
        }
        if crate::list_ops::looks_like_list_op(&toks, &self.env) {
            return match crate::list_ops::parse_src_all(&toks, &self.env, true) {
                Ok(s) => Some((LoopSrc::Items(s), LoopKind::List)),
                Err(e) => {
                    self.err(num, format!("can't understand 'loop {rest}': {e}"));
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
        // `loop 1, 2 and 3:` / `loop "a" and "b":` / `loop indices of {x::*}:`
        let Some(items) = crate::list_literals::split_items(&toks, &self.env) else {
            self.err(
                num,
                format!("can't understand 'loop {rest}' (use 'loop N times', 'loop {{x::*}}', 'loop all players' or 'loop a, b and c')"),
            );
            return None;
        };
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
        if let Some(c) = self.parse_cond(&toks) {
            return Some(c);
        }
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
        let srcs: Vec<&'static str> = crate::op_kick_ban::CONDITION_PATTERNS
            .iter()
            .chain(PLAYER_FLAG_PATTERNS)
            .chain(CONDITION_PATTERNS)
            .map(|(s, _)| *s)
            .collect();
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

    /// `a or b`, `a and b`, `a, b and c`, `(a or b) and c`, `x is a or b`, or one condition.
    fn parse_cond(&self, toks: &[Tok]) -> Option<Cond> {
        let toks = strip_parens(toks);
        // `chance of 25%`
        if let [first, second, mid @ .., last] = toks
            && first.is_lit("chance")
            && second.is_lit("of")
            && last.is_lit("%")
        {
            return parse_expr(mid, &self.env)
                .ok()
                .filter(|e| Ty::Number.accepts(e.ty(&self.env)))
                .map(Cond::Chance);
        }
        // Most conditions are a single check: try that first when there is
        // nothing to split on.
        let joins = toks
            .iter()
            .any(|t| t.is_lit("or") || t.is_lit("and") || t.is_lit(","));
        if !joins {
            return self.single_cond(toks);
        }
        if let Some(parts) = self.split_conds(toks, &["or"]) {
            return Some(Cond::Any(parts));
        }
        if let Some(parts) = self.split_conds(toks, &["and", ","]) {
            return Some(Cond::All(parts));
        }
        if let Some(c) = self.single_cond(toks) {
            return Some(c);
        }
        self.alternatives(toks)
    }

    fn single_cond(&self, toks: &[Tok]) -> Option<Cond> {
        if let Some(c) = crate::world_effects::parse_cond(toks, &self.env) {
            return Some(c);
        }
        if let Some(c) = crate::block_ops::parse_cond(toks, &self.env) {
            return Some(c);
        }
        let mut memo = Memo::default();
        registry()
            .conditions
            .iter()
            .find_map(|p| {
                p.matches_memo(toks, &self.env, &mut memo)
                    .ok()
                    .map(|caps| build_cond(p.kind, caps))
            })
            .or_else(|| self.span_cmp(toks))
    }

    /// `time since {d} > 5 minutes`: ordering for spans and dates.
    fn span_cmp(&self, toks: &[Tok]) -> Option<Cond> {
        let (at, op) = toks.iter().enumerate().find_map(|(i, t)| match t {
            Tok::Sym(">=") => Some((i, CmpOp::Ge)),
            Tok::Sym("<=") => Some((i, CmpOp::Le)),
            Tok::Sym(">") => Some((i, CmpOp::Gt)),
            Tok::Sym("<") => Some((i, CmpOp::Lt)),
            _ => None,
        })?;
        let a = parse_expr(toks.get(..at)?, &self.env).ok()?;
        let b = parse_expr(toks.get(at + 1..)?, &self.env).ok()?;
        let timey = |t: Ty| matches!(t, Ty::Timespan | Ty::Date);
        let (ta, tb) = (a.ty(&self.env), b.ty(&self.env));
        let ok = (timey(ta) || timey(tb)) && (ta == tb || ta == Ty::Object || tb == Ty::Object);
        ok.then_some(Cond::Cmp(a, op, b))
    }

    /// `wait 5 minutes`, `wait {span}`: the pattern table only has number forms.
    fn wait_span(&self, toks: &[Tok]) -> Option<Effect> {
        if !toks.first()?.is_lit("wait") {
            return None;
        }
        let e = parse_expr(toks.get(1..)?, &self.env).ok()?;
        matches!(e.ty(&self.env), Ty::Timespan | Ty::Object).then_some(Effect::Wait(e, 0))
    }

    /// Split at top-level separators so that every part is a condition.
    /// Tries split points in order, so `x is between 1 and 5 and y is 2` works.
    fn split_conds(&self, toks: &[Tok], seps: &[&str]) -> Option<Vec<Cond>> {
        let cuts = top_level(toks, seps);
        if cuts.is_empty() {
            return None;
        }
        self.split_from(toks, &cuts, 0, seps)
    }

    fn split_from(&self, toks: &[Tok], cuts: &[usize], start: usize, seps: &[&str]) -> Option<Vec<Cond>> {
        for (i, &cut) in cuts.iter().enumerate() {
            if cut <= start {
                continue;
            }
            let Some(first) = toks.get(start..cut).and_then(|t| self.parse_part(t, seps)) else {
                continue;
            };
            let rest_cuts = cuts.get(i + 1..).unwrap_or_default();
            let tail_start = cut + 1;
            // The rest as one condition, or split further.
            if let Some(last) = toks.get(tail_start..).and_then(|t| self.parse_part(t, seps)) {
                return Some(vec![first, last]);
            }
            if let Some(mut more) = self.split_from(toks, rest_cuts, tail_start, seps) {
                more.insert(0, first);
                return Some(more);
            }
        }
        None
    }

    /// A part of an `or` may itself hold `and`; a part of an `and` is one condition.
    fn parse_part(&self, toks: &[Tok], seps: &[&str]) -> Option<Cond> {
        if toks.is_empty() {
            return None;
        }
        if seps.contains(&"or")
            && let Some(parts) = self.split_conds(strip_parens(toks), &["and", ","])
        {
            return Some(Cond::All(parts));
        }
        let inner = strip_parens(toks);
        if inner.len() != toks.len() {
            return self.parse_cond(inner);
        }
        self.single_cond(toks).or_else(|| self.alternatives(toks))
    }

    /// `x is "a" or "b"`, `x is not 1, 2 or 3`.
    fn alternatives(&self, toks: &[Tok]) -> Option<Cond> {
        let pos = toks
            .iter()
            .position(|t| t.is_lit("is") || t.is_lit("=") || t.is_lit("!="))?;
        let (negate, rest_at) = match toks.get(pos) {
            Some(t) if t.is_lit("!=") => (true, pos + 1),
            Some(t) if t.is_lit("=") => (false, pos + 1),
            _ if toks.get(pos + 1).is_some_and(|t| t.is_lit("not")) => (true, pos + 2),
            _ => (false, pos + 1),
        };
        let value = parse_expr(toks.get(..pos)?, &self.env).ok()?;
        let rest = toks.get(rest_at..)?;
        let mut options = Vec::new();
        for part in rest.split(|t| t.is_lit("or") || t.is_lit(",") || t.is_lit("and")) {
            options.push(parse_expr(part, &self.env).ok()?);
        }
        (options.len() >= 2).then_some(Cond::AnyOf {
            value,
            options,
            negate,
        })
    }

    fn effect(&mut self, num: usize, text: &str) {
        let toks = match lex(text) {
            Ok(t) => t,
            Err(e) => {
                self.err(num, e);
                return;
            }
        };
        if self.call_stmt(num, &toks) {
            return;
        }
        let ops_first = toks.first().is_some_and(|t| t.is_lit("sort"))
            || (toks.first().is_some_and(|t| t.is_lit("set"))
                && toks.get(3..).is_some_and(|r| crate::list_ops::looks_like_list_op(r, &self.env)));
        let parsed = if ops_first {
            crate::list_ops::parse_effect(&toks, &self.env)
                .or_else(|| crate::list_literals::parse_list_effect(&toks, &self.env))
        } else {
            crate::list_literals::parse_list_effect(&toks, &self.env)
                .or_else(|| crate::list_ops::parse_effect(&toks, &self.env))
        };
        match parsed {
            Some(Ok(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Some(Err(e)) => {
                self.err(num, e);
                return;
            }
            None => {}
        }
        match crate::block_ops::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::world_effects::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        let mut type_err = None;
        let mut memo = Memo::default();
        for p in &registry().effects {
            match p.matches_memo(&toks, &self.env, &mut memo) {
                Ok(caps) => {
                    if p.kind == EffKind::Cancel && self.is_command {
                        self.err(num, "commands have no event to cancel");
                        return;
                    }
                    if p.kind == EffKind::Cancel && self.func_ret.is_some() {
                        self.err(num, "functions have no event to cancel");
                        return;
                    }
                    if p.kind == EffKind::Cancel
                        && let Some(why) = self.no_cancel.clone()
                    {
                        self.err(num, why);
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
        if let Some(eff) = self.wait_span(&toks) {
            self.emit(Instr::Eff(eff), num);
            return;
        }
        // A condition on its own line stops the trigger when false.
        if let Some(c) = self.parse_cond(&toks) {
            self.emit(Instr::StopUnless(c), num);
            return;
        }
        let type_err = inner_error(&toks, &self.env).or(type_err);
        let srcs: Vec<&'static str> = EFFECT_PATTERNS
            .iter()
            .chain(TEXT_EFFECT_PATTERNS)
            .chain(crate::health_food::EFFECT_PATTERNS)
            .map(|(s, _)| *s)
            .collect();
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

/// `( ... )` around the whole token list, removed.
fn strip_parens(toks: &[Tok]) -> &[Tok] {
    let mut t = toks;
    while let (Some(first), Some(last)) = (t.first(), t.last()) {
        if !(first.is_lit("(") && last.is_lit(")")) {
            break;
        }
        // The first '(' must close at the very end.
        let mut depth = 0i32;
        let mut closes_at_end = true;
        for (i, tok) in t.iter().enumerate() {
            if tok.is_lit("(") {
                depth += 1;
            } else if tok.is_lit(")") {
                depth -= 1;
                if depth == 0 && i + 1 != t.len() {
                    closes_at_end = false;
                    break;
                }
            }
        }
        if !closes_at_end {
            break;
        }
        t = t.get(1..t.len() - 1).unwrap_or_default();
    }
    t
}

/// Positions of separator words outside brackets.
fn top_level(toks: &[Tok], seps: &[&str]) -> Vec<usize> {
    let mut depth = 0i32;
    let mut out = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        if t.is_lit("(") {
            depth += 1;
        } else if t.is_lit(")") {
            depth -= 1;
        } else if depth == 0 && seps.iter().any(|s| t.is_lit(s)) {
            out.push(i);
        }
    }
    out
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
        CondKind::IsPlayer | CondKind::IsNotPlayer | CondKind::IsConsole | CondKind::IsNotConsole => {
            return Cond::SenderIs {
                value: a,
                console: matches!(kind, CondKind::IsConsole | CondKind::IsNotConsole),
                negate: matches!(kind, CondKind::IsNotPlayer | CondKind::IsNotConsole),
            };
        }
        CondKind::Flag(flag, negate) => {
            return Cond::PlayerFlag {
                player: a,
                flag,
                negate,
            };
        }
        CondKind::Admin(k, negate) => return Cond::Admin(k, a, negate),
        CondKind::IsSet => return Cond::IsSet(a, true),
        CondKind::NotSet => return Cond::IsSet(a, false),
        CondKind::Ne => CmpOp::Ne,
        CondKind::Eq => CmpOp::Eq,
        CondKind::Ge => CmpOp::Ge,
        CondKind::Le => CmpOp::Le,
        CondKind::Gt => CmpOp::Gt,
        CondKind::Lt => CmpOp::Lt,
        CondKind::InGamemode | CondKind::NotInGamemode => {
            let op = if kind == CondKind::InGamemode { CmpOp::Eq } else { CmpOp::Ne };
            let mode = Expr::Prop(crate::expr::Prop::Gamemode, Box::new(a));
            return Cond::Cmp(mode, op, b);
        }
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
        CondKind::HasItem | CondKind::InvContains => {
            return Cond::HasItem {
                player: a,
                item: b,
                negate: false,
            };
        }
        CondKind::NotHasItem => {
            return Cond::HasItem {
                player: a,
                item: b,
                negate: true,
            };
        }
        CondKind::Between | CondKind::NotBetween => {
            let high = take_expr(&mut c, 2).unwrap_or(Expr::Num(0.0));
            return Cond::Between {
                value: a,
                low: b,
                high,
                negate: kind == CondKind::NotBetween,
            };
        }
        CondKind::Contains
        | CondKind::NotContains
        | CondKind::StartsWith
        | CondKind::NotStartsWith
        | CondKind::EndsWith
        | CondKind::NotEndsWith => {
            let op = match kind {
                CondKind::Contains | CondKind::NotContains => TextOp::Contains,
                CondKind::StartsWith | CondKind::NotStartsWith => TextOp::StartsWith,
                _ => TextOp::EndsWith,
            };
            let negate = matches!(
                kind,
                CondKind::NotContains | CondKind::NotStartsWith | CondKind::NotEndsWith
            );
            return Cond::Text {
                left: a,
                op,
                right: b,
                negate,
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
        EffKind::Hf(op) => return crate::health_food::build(op, c),
        EffKind::Heal => Effect::Heal(take_expr(c, 0)?),
        EffKind::SetHealth => Effect::SetHealth(take_expr(c, 0)?, take_expr(c, 1)?),
        EffKind::SetGamemode => Effect::SetGamemode(take_expr(c, 0)?, take_expr(c, 1)?),
        EffKind::Xp(op) => Effect::Xp(op, take_expr(c, 0)?, take_expr(c, 1)?),
        EffKind::Kill => Effect::Kill(take_expr(c, 0)?),
        EffKind::Teleport => Effect::Teleport(
            take_expr(c, 0)?,
            take_expr(c, 1)?,
            take_expr(c, 2)?,
            take_expr(c, 3)?,
        ),
        EffKind::TeleportTo => Effect::TeleportTo(take_expr(c, 0)?, take_expr(c, 1)?),
        EffKind::GiveCount => {
            let n = take_expr(c, 0)?;
            let item = take_expr(c, 1)?;
            Effect::Give(take_expr(c, 2)?, Some(n), item)
        }
        EffKind::GiveTo => {
            let item = take_expr(c, 0)?;
            Effect::Give(take_expr(c, 1)?, None, item)
        }
        EffKind::Give => {
            let p = take_expr(c, 0)?;
            Effect::Give(p, None, take_expr(c, 1)?)
        }
        EffKind::ConsoleCommand => Effect::ConsoleCommand(take_expr(c, 0)?),
        EffKind::PlaySound => Effect::PlaySound {
            sound: take_expr(c, 0)?,
            volume: take_expr(c, 1),
            pitch: take_expr(c, 2),
            to: take_expr(c, 3),
            at: take_expr(c, 4),
        },
        EffKind::StopSound => Effect::StopSound(Some(take_expr(c, 0)?), take_expr(c, 1)),
        EffKind::StopAllSounds => Effect::StopSound(None, take_expr(c, 0)),
        EffKind::ExecAs => {
            let who = take_expr(c, 0)?;
            Effect::ExecAs(who, take_expr(c, 1)?)
        }
        EffKind::ExecAsBy => {
            let cmd = take_expr(c, 0)?;
            Effect::ExecAs(take_expr(c, 1)?, cmd)
        }
        EffKind::Cancel => Effect::Cancel,
        EffKind::WaitTicks => Effect::Wait(take_expr(c, 0)?, 1),
        EffKind::WaitSeconds => Effect::Wait(take_expr(c, 0)?, 20),
        EffKind::Set => {
            let v = take_var(c, 0)?;
            match take_expr(c, 1)? {
                Expr::Split(a, b) if v.is_list() => Effect::SetSplit(v, *a, *b),
                e => Effect::Set(v, e),
            }
        }
        EffKind::ReplaceAllIn => Effect::Replace(take_var(c, 1)?, take_expr(c, 0)?, take_expr(c, 2)?, true),
        EffKind::ReplaceFirstIn => {
            Effect::Replace(take_var(c, 1)?, take_expr(c, 0)?, take_expr(c, 2)?, false)
        }
        EffKind::ReplaceAllWith => Effect::Replace(take_var(c, 2)?, take_expr(c, 0)?, take_expr(c, 1)?, true),
        EffKind::ReplaceFirstWith => {
            Effect::Replace(take_var(c, 2)?, take_expr(c, 0)?, take_expr(c, 1)?, false)
        }
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
        EffKind::Admin(k) => Effect::Admin(k, take_expr(c, 0)?, take_expr(c, 1)),
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
        aliases: Vec::new(),
        usage_msg: None,
        players_only: None,
    })
}

/// Compile one file on its own (no functions from other files).
pub fn compile_file(file: &str, src: &str) -> Result<Vec<Trigger>, Vec<ScriptError>> {
    let mut c = compile_all(&[(file.to_string(), src.to_string())]);
    if c.errors.is_empty() {
        Ok(c.files.pop().map(|(_, t)| t).unwrap_or_default())
    } else {
        Err(c.errors)
    }
}

/// Parsed file: its lines after options were filled in.
struct Parsed {
    name: String,
    tree: Vec<Line>,
}

/// Compile a set of files together, so functions can be called across files.
pub fn compile_all(files: &[(String, String)]) -> Compiled {
    let mut out = Compiled::default();
    // Pass 1: block structure, options and every function signature.
    let mut parsed: Vec<Parsed> = Vec::new();
    let mut bad_files: Vec<String> = Vec::new();
    let mut sigs: Vec<Sig> = Vec::new();
    let mut func_src: Vec<(usize, usize)> = Vec::new(); // (parsed index, top line index)
    let mut defaults_src: Vec<Vec<Option<String>>> = Vec::new();
    for (name, src) in files {
        let tree = match parse_blocks(name, src) {
            Ok(t) => t,
            Err(e) => {
                out.errors.extend(e);
                bad_files.push(name.clone());
                continue;
            }
        };
        // Most files have no options; skip the extra pass for them.
        let has_options = src.contains("{@") || tree.iter().any(|t| t.text.eq_ignore_ascii_case("options:"));
        let tree = match if has_options {
            apply_options(name, tree)
        } else {
            Ok(tree)
        } {
            Ok(t) => t,
            Err(e) => {
                out.errors.extend(e);
                bad_files.push(name.clone());
                continue;
            }
        };
        for (ti, top) in tree.iter().enumerate() {
            if top.head().to_ascii_lowercase().starts_with("function ") {
                match parse_function_header(name, top.num, top.head()) {
                    Ok((sig, defs)) => {
                        if let Some(prev) = sigs.iter().position(|s| s.name.eq_ignore_ascii_case(&sig.name)) {
                            let _ = prev;
                            out.errors.push(ScriptError::new(
                                name,
                                top.num,
                                format!("function {}() is defined twice", sig.name),
                            ));
                            continue;
                        }
                        sigs.push(sig);
                        defaults_src.push(defs);
                        func_src.push((parsed.len(), ti));
                    }
                    Err(e) => out.errors.push(e),
                }
            }
        }
        parsed.push(Parsed {
            name: name.clone(),
            tree,
        });
    }
    let sigs = Arc::new(sigs);

    // Pass 2: function bodies.
    for (id, &(pi, ti)) in func_src.iter().enumerate() {
        let (Some(p), Some(sig)) = (parsed.get(pi), sigs.get(id)) else {
            out.functions.push(None);
            continue;
        };
        let Some(top) = p.tree.get(ti) else {
            out.functions.push(None);
            continue;
        };
        let env = Env {
            trigger_name: format!("function {}()", sig.name),
            functions: sigs.clone(),
            in_function: true,
            ..Env::default()
        };
        // Defaults are plain values: compile them without parameters in scope.
        let mut defaults = Vec::new();
        let mut errs = Vec::new();
        for d in defaults_src.get(id).map(Vec::as_slice).unwrap_or_default() {
            defaults.push(match d {
                Some(text) => match lex(text).and_then(|t| parse_expr(&t, &env)) {
                    Ok(e) => Some(e),
                    Err(e) => {
                        errs.push(ScriptError::new(
                            &p.name,
                            top.num,
                            format!("bad default value '{text}': {e}"),
                        ));
                        None
                    }
                },
                None => None,
            });
        }
        let mut cx = Cx {
            file: &p.name,
            env,
            is_command: false,
            func_ret: Some(sig.ret),
            no_cancel: None,
            code: Vec::new(),
            lines: Vec::new(),
            errors: errs,
            loops: Vec::new(),
        };
        cx.block(&top.children);
        if cx.errors.is_empty() {
            out.functions.push(Some(Arc::new(Function {
                sig: sig.clone(),
                defaults,
                code: Arc::new(Code {
                    file: Arc::from(p.name.as_str()),
                    line: top.num,
                    instrs: cx.code,
                    lines: cx.lines,
                }),
            })));
        } else {
            out.errors.extend(cx.errors);
            bad_files.push(p.name.clone());
            out.functions.push(None);
        }
    }

    // Pass 3: triggers.
    for p in &parsed {
        let (trigs, errs) = compile_triggers(&p.name, &p.tree, &sigs);
        if errs.is_empty() && !bad_files.contains(&p.name) {
            out.files.push((p.name.clone(), trigs));
        } else {
            out.errors.extend(errs);
        }
    }
    out
}

/// Replace `{@name}` with the values from the file's `options:` blocks.
fn apply_options(file: &str, tree: Vec<Line>) -> Result<Vec<Line>, Vec<ScriptError>> {
    let mut opts: Vec<(String, String)> = Vec::new();
    let mut rest = Vec::new();
    let mut errors = Vec::new();
    for top in tree {
        if top.text.eq_ignore_ascii_case("options:") {
            for o in &top.children {
                match o.text.split_once(':') {
                    Some((k, v)) if !k.trim().is_empty() => {
                        opts.push((k.trim().to_ascii_lowercase(), v.trim().to_string()))
                    }
                    _ => errors.push(ScriptError::new(file, o.num, "options look like 'name: value'")),
                }
            }
        } else {
            rest.push(top);
        }
    }
    fn fill(line: &mut Line, opts: &[(String, String)], file: &str, errors: &mut Vec<ScriptError>) {
        while let Some(start) = line.text.find("{@") {
            let Some(len) = line.text.get(start..).and_then(|t| t.find('}')) else {
                errors.push(ScriptError::new(
                    file,
                    line.num,
                    "unclosed option: missing '}' after '{@'",
                ));
                break;
            };
            let name = line
                .text
                .get(start + 2..start + len)
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            match opts.iter().find(|(k, _)| *k == name) {
                Some((_, v)) => line.text.replace_range(start..start + len + 1, v),
                None => {
                    errors.push(ScriptError::new(
                        file,
                        line.num,
                        format!("unknown option {{@{name}}}"),
                    ));
                    break;
                }
            }
        }
        for c in &mut line.children {
            fill(c, opts, file, errors);
        }
    }
    for l in &mut rest {
        fill(l, &opts, file, &mut errors);
    }
    if errors.is_empty() { Ok(rest) } else { Err(errors) }
}

pub fn type_from_name(name: &str) -> Option<Ty> {
    Some(match name.trim().to_ascii_lowercase().as_str() {
        "text" | "string" => Ty::Text,
        "number" | "integer" | "int" => Ty::Number,
        "player" | "offline player" | "offlineplayer" => Ty::Player,
        "boolean" | "bool" => Ty::Boolean,
        "item" | "itemtype" | "item type" | "block" => Ty::Item,
        "location" => Ty::Location,
        "world" => Ty::World,
        "timespan" | "time span" => Ty::Timespan,
        "date" => Ty::Date,
        "object" => Ty::Object,
        "numbers" | "integers" | "texts" | "strings" | "players" | "booleans" | "items" | "itemtypes"
        | "locations" | "worlds" | "objects" => Ty::List,
        _ => return None,
    })
}

/// Split at commas that are outside quotes and brackets.
fn split_commas(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut in_str, mut last) = (0i32, false, 0usize);
    for (i, c) in s.char_indices() {
        match c {
            '"' => in_str = !in_str,
            '(' | '{' if !in_str => depth += 1,
            ')' | '}' if !in_str => depth -= 1,
            ',' if !in_str && depth == 0 => {
                out.push(s.get(last..i).unwrap_or(""));
                last = i + 1;
            }
            _ => {}
        }
    }
    out.push(s.get(last..).unwrap_or(""));
    out
}

/// `function name(a: text, b: number = 5) :: text` (the trailing `:` already removed).
fn parse_function_header(
    file: &str,
    num: usize,
    head: &str,
) -> Result<(Sig, Vec<Option<String>>), ScriptError> {
    let err = |m: String| ScriptError::new(file, num, m);
    let rest = head.get("function".len()..).unwrap_or("").trim();
    let (name, after) = rest
        .split_once('(')
        .ok_or_else(|| err("expected 'function name(...)'".into()))?;
    let name = name.trim();
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(err(format!(
            "bad function name '{name}': use letters, digits and _"
        )));
    }
    let close = after
        .rfind(')')
        .ok_or_else(|| err("missing ')' in the function header".into()))?;
    let params_src = after.get(..close).unwrap_or("");
    let tail = after.get(close + 1..).unwrap_or("").trim();
    let ret = if tail.is_empty() {
        None
    } else {
        let t = tail.strip_prefix("::").ok_or_else(|| {
            err(format!(
                "unexpected '{tail}' after the parameters (use ':: type' for the return type)"
            ))
        })?;
        Some(type_from_name(t).ok_or_else(|| err(format!("unknown return type '{}'", t.trim())))?)
    };
    let mut params = Vec::new();
    let mut defaults = Vec::new();
    if !params_src.trim().is_empty() {
        for p in split_commas(params_src) {
            let (decl, default) = match p.split_once('=') {
                Some((d, v)) => (d, Some(v.trim().to_string())),
                None => (p, None),
            };
            let (pname, pty) = decl.split_once(':').ok_or_else(|| {
                err(format!(
                    "parameter '{}' needs a type, like 'name: text'",
                    decl.trim()
                ))
            })?;
            let pname = pname.trim().to_ascii_lowercase();
            let ty = type_from_name(pty).ok_or_else(|| {
                err(format!("unknown type '{}' for parameter {pname} (use text, number, player, boolean, item, location, world or object; for lists use numbers, texts, players, objects and so on)", pty.trim()))
            })?;
            if default.is_none() && defaults.iter().any(Option::is_some) {
                return Err(err(format!(
                    "parameter {pname} needs a default, because the one before it has one"
                )));
            }
            params.push((format!("_{pname}"), ty, default.is_some()));
            defaults.push(default);
        }
    }
    Ok((
        Sig {
            name: name.to_string(),
            params,
            ret,
        },
        defaults,
    ))
}

/// `right click with tnt on barrel` -> ("right click", [("with", "tnt"), ("on", "barrel")]).
fn split_click_filters(ev: &str) -> (&str, Vec<(&'static str, String)>) {
    let mut base = ev;
    let mut filters = Vec::new();
    // Find the first ` with ` / ` on ` and split from there.
    let first = [" with ", " on "].iter().filter_map(|k| ev.find(k)).min();
    if let Some(i) = first {
        base = ev.get(..i).unwrap_or(ev).trim();
        let mut rest = ev.get(i..).unwrap_or("").trim();
        while !rest.is_empty() {
            let (key, after) = if let Some(r) = rest.strip_prefix("with ") {
                ("with", r)
            } else if let Some(r) = rest.strip_prefix("on ") {
                ("on", r)
            } else {
                break;
            };
            let next = [" with ", " on "]
                .iter()
                .filter_map(|k| after.find(k))
                .min()
                .unwrap_or(after.len());
            filters.push((key, after.get(..next).unwrap_or(after).trim().to_string()));
            rest = after.get(next..).unwrap_or("").trim();
        }
    }
    (base, filters)
}

/// `5 seconds`, `1 minute`, `20 ticks`, `second` -> ticks.
fn parse_every(s: &str) -> Option<u64> {
    let mut parts = s.split_whitespace();
    let (n, unit) = match (parts.next(), parts.next(), parts.next()) {
        (Some(n), Some(u), None) => (n.parse::<f64>().ok()?, u),
        (Some(u), None, None) => (1.0, u),
        _ => return None,
    };
    let per = match unit {
        "tick" | "ticks" => 1.0,
        "second" | "seconds" => 20.0,
        "minute" | "minutes" => 1200.0,
        "hour" | "hours" => 72000.0,
        _ => return None,
    };
    let ticks = (n * per).round();
    (ticks >= 1.0 && ticks.is_finite()).then_some(ticks as u64)
}

fn compile_triggers(file: &str, tree: &[Line], sigs: &Arc<Vec<Sig>>) -> (Vec<Trigger>, Vec<ScriptError>) {
    let file_arc: Arc<str> = Arc::from(file);
    let mut triggers = Vec::new();
    let mut errors = Vec::new();
    let mut click_filters: Vec<(&'static str, String)> = Vec::new();
    for top in tree {
        click_filters.clear();
        let lower = top.head().to_ascii_lowercase();
        if lower.starts_with("function ") {
            continue;
        }
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
        let (kind, body, env, is_command): (TriggerKind, &[Line], Env, bool) = if let Some(ev) =
            lower.strip_prefix("on ")
        {
            let (ev, filters) = split_click_filters(ev.trim());
            let Some(k) = EventKind::parse(ev) else {
                errors.push(ScriptError::new(
                        file,
                        top.num,
                        format!(
                            "unknown event '{ev}', known: join, quit, chat, break, place, death, damage, respawn, click, right click, left click, sneak toggle, sprint toggle, flight toggle, load (and 'every N seconds:')"
                        ),
                    ));
                continue;
            };
            if !filters.is_empty()
                && !matches!(k, EventKind::Click | EventKind::RightClick | EventKind::LeftClick)
            {
                errors.push(ScriptError::new(
                    file,
                    top.num,
                    "'on ... with/on <item>' only works for click events",
                ));
                continue;
            }
            let env = Env {
                has_message: k == EventKind::Chat,
                arg_types: vec![],
                trigger_name: format!("'on {}'", k.name()),
                no_player: k == EventKind::Load,
                has_damage: k == EventKind::Damage,
                has_attacker: matches!(k, EventKind::Damage | EventKind::Death),
                has_block: matches!(k, EventKind::Click | EventKind::RightClick | EventKind::LeftClick),
                ..Env::default()
            };
            click_filters.extend(filters);
            (TriggerKind::Event(k), &top.children, env, false)
        } else if let Some(rest) = lower.strip_prefix("every ") {
            let Some(ticks) = parse_every(rest) else {
                errors.push(ScriptError::new(
                        file,
                        top.num,
                        format!("can't understand 'every {rest}', use 'every 5 seconds:', 'every 20 ticks:' or 'every 2 minutes:'"),
                    ));
                continue;
            };
            let env = Env {
                trigger_name: format!("'every {rest}'"),
                no_player: true,
                ..Env::default()
            };
            (TriggerKind::Every(ticks), &top.children, env, false)
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
                } else if el.starts_with("aliases:") {
                    let v = entry.text.get("aliases:".len()..).unwrap_or("");
                    def.aliases = v
                        .split(',')
                        .map(|a| a.trim().trim_start_matches('/').to_ascii_lowercase())
                        .filter(|a| !a.is_empty())
                        .collect();
                } else if el.starts_with("usage:") {
                    def.usage_msg = Some(entry.text.get("usage:".len()..).unwrap_or("").trim().to_string());
                } else if el.starts_with("executable by:") {
                    let v = el.get("executable by:".len()..).unwrap_or("").trim().to_string();
                    def.players_only = match v.as_str() {
                        "players" | "player" => Some(true),
                        "console" => Some(false),
                        "players and console" | "console and players" => None,
                        _ => {
                            errors.push(ScriptError::new(
                                file,
                                entry.num,
                                "use 'executable by: players' or 'console'",
                            ));
                            None
                        }
                    };
                } else if el.starts_with("cooldown") {
                    errors.push(ScriptError::new(
                        file,
                        entry.num,
                        "command cooldowns are not supported yet; store the time in a variable instead",
                    ));
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
                            "unknown command entry '{}', use permission:, description:, aliases:, usage:, executable by: or trigger:",
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
        let no_cancel = match &kind {
            TriggerKind::Event(k) if !k.cancellable() => {
                Some(format!("'on {}' can't be cancelled", k.name()))
            }
            TriggerKind::Every(_) => Some("'every' has no event to cancel".to_string()),
            _ => None,
        };
        let mut cx = Cx {
            file,
            env,
            is_command,
            func_ret: None,
            no_cancel,
            code: Vec::new(),
            lines: Vec::new(),
            errors: Vec::new(),
            loops: Vec::new(),
        };
        cx.env.functions = sigs.clone();
        // `on right click with diamond on stone:` checks first.
        let mut guards = Vec::new();
        for (what, text) in std::mem::take(&mut click_filters) {
            let item = match lex(&text).and_then(|t| parse_expr(&t, &cx.env)) {
                Ok(e) if e.ty(&cx.env) == Ty::Item => e,
                _ => {
                    cx.err(top.num, format!("'{text}' is not an item or block name"));
                    continue;
                }
            };
            let left = if what == "with" {
                Expr::Prop(crate::expr::Prop::Tool, Box::new(Expr::Player))
            } else {
                Expr::ClickedBlock
            };
            let at = cx.emit(
                Instr::JumpIfNot(Cond::Cmp(left, CmpOp::Eq, item), usize::MAX),
                top.num,
            );
            guards.push(at);
        }
        cx.block(body);
        let end = cx.code.len();
        for g in guards {
            if let Some(Instr::JumpIfNot(_, t)) = cx.code.get_mut(g) {
                *t = end;
            }
        }
        if cx.errors.is_empty() {
            triggers.push(Trigger {
                file: file_arc.clone(),
                line: top.num,
                kind,
                code: Arc::new(Code {
                    file: file_arc.clone(),
                    line: top.num,
                    instrs: cx.code,
                    lines: cx.lines,
                }),
            });
        } else {
            errors.extend(cx.errors);
        }
    }
    (triggers, errors)
}
