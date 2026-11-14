//! Script table, event dispatch, the flat-instruction VM and wait/resume.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use crate::compile::{ArgTy, CmpOp, CommandDef, Cond, Effect, EventKind, Instr, Trigger, TriggerKind, compile_file};
use crate::error::ScriptError;
use crate::expr::{BinOp, Expr, Part, VarRef};
use crate::host::{Host, LogLevel};
use crate::value::{PlayerRef, Recipient, Value, format_num};
use crate::vars::{Variables, delete_in};

pub const DEFAULT_MAX_INSTRUCTIONS: usize = 100_000;
pub const SAVE_DEBOUNCE_MS: u64 = 5_000;

/// What an event offers to the scripts that handle it.
#[derive(Debug, Clone)]
pub struct EventCtx {
    pub player: Option<PlayerRef>,
    pub sender: Recipient,
    pub message: Option<String>,
    pub args: Vec<Value>,
}

impl EventCtx {
    pub fn player(p: PlayerRef) -> Self {
        EventCtx { sender: Recipient::Player(p.clone()), player: Some(p), message: None, args: Vec::new() }
    }

    pub fn chat(p: PlayerRef, message: impl Into<String>) -> Self {
        let mut c = Self::player(p);
        c.message = Some(message.into());
        c
    }
}

#[derive(Debug, Default)]
pub struct ScriptTable {
    events: HashMap<EventKind, Vec<Arc<Trigger>>>,
    commands: BTreeMap<String, Arc<Trigger>>,
    pub files: usize,
    pub triggers: usize,
}

impl ScriptTable {
    pub fn command_names(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }

    pub fn command(&self, name: &str) -> Option<&CommandDef> {
        match &self.commands.get(name)?.kind {
            TriggerKind::Command(d) => Some(d),
            TriggerKind::Event(_) => None,
        }
    }

    pub fn has_event(&self, k: EventKind) -> bool {
        self.events.get(&k).is_some_and(|v| !v.is_empty())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadSummary {
    pub files: usize,
    pub triggers: usize,
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandOutcome {
    NotFound,
    NoPermission,
    /// Bad arguments; the text is a message for the sender.
    Usage(String),
    Ran,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FireOutcome {
    pub cancelled: bool,
}

struct Exec {
    trig: Arc<Trigger>,
    pc: usize,
    locals: BTreeMap<String, Value>,
    ctx: EventCtx,
    waited: bool,
    cancelled: bool,
}

enum Step {
    Done,
    Wait(u64),
    Abort,
}

pub struct Engine {
    table: Arc<ScriptTable>,
    waits: HashMap<u64, Exec>,
    next_token: u64,
    pub vars: Variables,
    pub max_instructions: usize,
    dirty_since: Option<u64>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(Variables::default())
    }
}

fn build_table(files: &[(String, String)]) -> (ScriptTable, Vec<ScriptError>) {
    let mut table = ScriptTable::default();
    let mut errors = Vec::new();
    for (name, src) in files {
        match compile_file(name, src) {
            Ok(trigs) => {
                table.files += 1;
                for t in trigs {
                    table.triggers += 1;
                    let t = Arc::new(t);
                    match &t.kind {
                        TriggerKind::Event(k) => table.events.entry(*k).or_default().push(t.clone()),
                        TriggerKind::Command(d) => {
                            if table.commands.contains_key(&d.name) {
                                errors.push(ScriptError::new(&t.file, t.line, format!("command /{} is defined twice", d.name)));
                            } else {
                                table.commands.insert(d.name.clone(), t.clone());
                            }
                        }
                    }
                }
            }
            Err(e) => errors.extend(e),
        }
    }
    (table, errors)
}

impl Engine {
    pub fn new(vars: Variables) -> Self {
        Engine {
            table: Arc::new(ScriptTable::default()),
            waits: HashMap::new(),
            next_token: 1,
            vars,
            max_instructions: DEFAULT_MAX_INSTRUCTIONS,
            dirty_since: None,
        }
    }

    pub fn table(&self) -> &ScriptTable {
        &self.table
    }

    fn summary(&self) -> LoadSummary {
        LoadSummary { files: self.table.files, triggers: self.table.triggers, commands: self.table.command_names() }
    }

    /// First load: files that compile are used, the rest are reported.
    pub fn load(&mut self, files: &[(String, String)]) -> (LoadSummary, Vec<ScriptError>) {
        let (table, errors) = build_table(files);
        self.table = Arc::new(table);
        (self.summary(), errors)
    }

    /// Reload: all or nothing. On any error the old scripts stay active.
    /// Runs that are waiting keep the code they started with.
    pub fn reload(&mut self, files: &[(String, String)]) -> Result<LoadSummary, Vec<ScriptError>> {
        let (table, errors) = build_table(files);
        if !errors.is_empty() {
            return Err(errors);
        }
        self.table = Arc::new(table);
        Ok(self.summary())
    }

    pub fn pending_waits(&self) -> usize {
        self.waits.len()
    }

    /// Run every trigger for `kind`. Returns whether the event got cancelled
    /// (only `cancel event` before the first `wait` counts).
    pub fn fire(&mut self, kind: EventKind, ctx: EventCtx, host: &mut dyn Host) -> FireOutcome {
        let trigs: Vec<Arc<Trigger>> = self.table.events.get(&kind).cloned().unwrap_or_default();
        let mut cancelled = false;
        for trig in trigs {
            let ex = Exec { trig, pc: 0, locals: BTreeMap::new(), ctx: ctx.clone(), waited: false, cancelled: false };
            if let Some(c) = self.start(ex, host) {
                cancelled |= c;
            }
        }
        FireOutcome { cancelled }
    }

    /// Run a script command. `raw_args` is everything after the command name.
    pub fn run_command(&mut self, name: &str, sender: Recipient, raw_args: &str, host: &mut dyn Host) -> CommandOutcome {
        let name = name.trim_start_matches('/').to_ascii_lowercase();
        let Some(trig) = self.table.commands.get(&name).cloned() else {
            return CommandOutcome::NotFound;
        };
        let TriggerKind::Command(def) = &trig.kind else {
            return CommandOutcome::NotFound;
        };
        let player = match &sender {
            Recipient::Player(p) => Some(p.clone()),
            Recipient::Console => None,
        };
        if let (Some(perm), Some(p)) = (&def.permission, &player)
            && !host.has_permission(p, perm)
        {
            return CommandOutcome::NoPermission;
        }
        let args = match parse_args(def, raw_args, host) {
            Ok(a) => a,
            Err(msg) => return CommandOutcome::Usage(msg),
        };
        let ctx = EventCtx { player, sender, message: None, args };
        let ex = Exec { trig: trig.clone(), pc: 0, locals: BTreeMap::new(), ctx, waited: false, cancelled: false };
        self.start(ex, host);
        CommandOutcome::Ran
    }

    /// Continue a run that called `wait`.
    pub fn resume(&mut self, token: u64, host: &mut dyn Host) {
        if let Some(ex) = self.waits.remove(&token) {
            self.start(ex, host);
        }
    }

    /// Returns the cancel flag as it stood before any wait (None if the
    /// run was aborted or was a resumed run).
    fn start(&mut self, mut ex: Exec, host: &mut dyn Host) -> Option<bool> {
        let resumed = ex.waited;
        match self.run(&mut ex, host) {
            Step::Done => {}
            Step::Wait(ticks) => {
                let cancelled = ex.cancelled;
                ex.waited = true;
                let token = self.next_token;
                self.next_token += 1;
                self.waits.insert(token, ex);
                host.schedule_wait(ticks, token);
                return if resumed { None } else { Some(cancelled) };
            }
            Step::Abort => return None,
        }
        if resumed { None } else { Some(ex.cancelled) }
    }

    fn run(&mut self, ex: &mut Exec, host: &mut dyn Host) -> Step {
        let trig = ex.trig.clone();
        let mut budget = self.max_instructions;
        while let Some(instr) = trig.code.get(ex.pc) {
            let line = trig.lines.get(ex.pc).copied().unwrap_or(trig.line);
            if budget == 0 {
                host.log(
                    LogLevel::Error,
                    &format!("{}:{}: script stopped: ran more than {} instructions without a wait", trig.file, line, self.max_instructions),
                );
                return Step::Abort;
            }
            budget -= 1;
            ex.pc += 1;
            let r = match instr {
                Instr::Jump(t) => {
                    ex.pc = *t;
                    Ok(None)
                }
                Instr::JumpIfNot(c, t) => match self.cond(c, ex, host) {
                    Ok(true) => Ok(None),
                    Ok(false) => {
                        ex.pc = *t;
                        Ok(None)
                    }
                    Err(e) => Err(e),
                },
                Instr::Eff(e) => self.effect(e, ex, host, line),
            };
            match r {
                Ok(None) => {}
                Ok(Some(ticks)) => return Step::Wait(ticks),
                Err(e) => {
                    host.log(LogLevel::Error, &format!("{}:{}: {}", trig.file, line, e));
                    return Step::Abort;
                }
            }
        }
        Step::Done
    }

    fn cond(&mut self, c: &Cond, ex: &Exec, host: &mut dyn Host) -> Result<bool, String> {
        match c {
            Cond::Cmp(a, op, b) => {
                let a = self.eval(a, ex, host)?;
                let b = self.eval(b, ex, host)?;
                Ok(match op {
                    CmpOp::Eq => a.loose_eq(&b),
                    CmpOp::Ne => !a.loose_eq(&b),
                    _ => {
                        let (Some(x), Some(y)) = (to_num_opt(&a), to_num_opt(&b)) else {
                            return Ok(false);
                        };
                        match op {
                            CmpOp::Gt => x > y,
                            CmpOp::Lt => x < y,
                            CmpOp::Ge => x >= y,
                            _ => x <= y,
                        }
                    }
                })
            }
            Cond::IsSet(e, want) => {
                let v = self.eval(e, ex, host)?;
                Ok((v != Value::None) == *want)
            }
            Cond::HasPerm { player, node, negate } => {
                let p = self.eval(player, ex, host)?;
                let node = self.eval(node, ex, host)?.to_string();
                let Some(p) = self.online(&p, host) else {
                    return Ok(false);
                };
                Ok(host.has_permission(&p, &node) != *negate)
            }
        }
    }

    /// Resolve a value to an online player, or None.
    fn online(&self, v: &Value, host: &mut dyn Host) -> Option<PlayerRef> {
        match v {
            Value::Player(p) => host.resolve_player(&p.uuid).map(|name| PlayerRef { uuid: p.uuid.clone(), name }),
            Value::Text(name) => host.find_player(name),
            _ => None,
        }
    }

    fn text(&mut self, e: &Expr, ex: &Exec, host: &mut dyn Host) -> Result<String, String> {
        Ok(self.eval(e, ex, host)?.to_string())
    }

    fn num(&mut self, e: &Expr, ex: &Exec, host: &mut dyn Host) -> Result<f64, String> {
        let v = self.eval(e, ex, host)?;
        to_num_opt(&v).ok_or_else(|| format!("expected a number, got {} ({})", v.type_name(), v))
    }

    fn player(&mut self, e: &Expr, ex: &Exec, host: &mut dyn Host) -> Result<Option<PlayerRef>, String> {
        let v = self.eval(e, ex, host)?;
        Ok(self.online(&v, host))
    }

    fn key(&mut self, v: &VarRef, ex: &Exec, host: &mut dyn Host) -> Result<String, String> {
        Ok(self.template(&v.name, ex, host)?.to_ascii_lowercase())
    }

    fn template(&mut self, parts: &[Part], ex: &Exec, host: &mut dyn Host) -> Result<String, String> {
        let mut s = String::new();
        for p in parts {
            match p {
                Part::Lit(l) => s.push_str(l),
                Part::Expr(e) => s.push_str(&self.eval(e, ex, host)?.to_string()),
            }
        }
        Ok(s)
    }

    fn get_var(&mut self, v: &VarRef, ex: &Exec, host: &mut dyn Host) -> Result<Value, String> {
        let k = self.key(v, ex, host)?;
        Ok(if v.local { ex.locals.get(&k).cloned().unwrap_or(Value::None) } else { self.vars.get(&k) })
    }

    fn set_var(&mut self, v: &VarRef, val: Value, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        let k = self.key(v, ex, host)?;
        if v.local {
            if val == Value::None {
                ex.locals.remove(&k);
            } else {
                ex.locals.insert(k, val);
            }
        } else {
            self.vars.set(k, val);
            self.mark_dirty(host);
        }
        Ok(())
    }

    fn mark_dirty(&mut self, host: &mut dyn Host) {
        if self.dirty_since.is_none() {
            self.dirty_since = Some(host.now());
        }
    }

    fn eval(&mut self, e: &Expr, ex: &Exec, host: &mut dyn Host) -> Result<Value, String> {
        Ok(match e {
            Expr::Num(n) => Value::Num(*n),
            Expr::Text(parts) => Value::Text(self.template(parts, ex, host)?),
            Expr::Var(v) => self.get_var(v, ex, host)?,
            Expr::Player => match &ex.ctx.player {
                Some(p) => match host.resolve_player(&p.uuid) {
                    Some(name) => Value::Player(PlayerRef { uuid: p.uuid.clone(), name }),
                    None => Value::None,
                },
                None => Value::None,
            },
            Expr::Message => ex.ctx.message.clone().map(Value::Text).unwrap_or(Value::None),
            Expr::Arg(i) => ex.ctx.args.get(i.wrapping_sub(1)).cloned().unwrap_or(Value::None),
            Expr::NameOf(inner) => {
                let v = self.eval(inner, ex, host)?;
                match self.online(&v, host) {
                    Some(p) => Value::Text(p.name),
                    None => Value::None,
                }
            }
            Expr::Neg(inner) => Value::Num(-self.num(inner, ex, host)?),
            Expr::Bin(a, op, b) => {
                let x = self.num(a, ex, host)?;
                let y = self.num(b, ex, host)?;
                Value::Num(match op {
                    BinOp::Add => x + y,
                    BinOp::Sub => x - y,
                    BinOp::Mul => x * y,
                    BinOp::Div => {
                        if y == 0.0 {
                            return Err("division by zero".into());
                        }
                        x / y
                    }
                })
            }
        })
    }

    /// Ok(Some(ticks)) means: pause for that many ticks.
    fn effect(&mut self, e: &Effect, ex: &mut Exec, host: &mut dyn Host, line: usize) -> Result<Option<u64>, String> {
        match e {
            Effect::Send { text, to } => {
                let msg = self.text(text, ex, host)?;
                match to {
                    Some(to) => {
                        if let Some(p) = self.player(to, ex, host)? {
                            host.send_message(&Recipient::Player(p), &msg);
                        }
                    }
                    None => match ex.ctx.sender.clone() {
                        Recipient::Console => host.send_message(&Recipient::Console, &msg),
                        Recipient::Player(p) => {
                            if let Some(p) = self.online(&Value::Player(p), host) {
                                host.send_message(&Recipient::Player(p), &msg);
                            }
                        }
                    },
                }
            }
            Effect::Broadcast(t) => {
                let msg = self.text(t, ex, host)?;
                host.broadcast(&msg);
            }
            Effect::Title { title, subtitle, to } => {
                let t = self.text(title, ex, host)?;
                let s = match subtitle {
                    Some(s) => Some(self.text(s, ex, host)?),
                    None => None,
                };
                if let Some(p) = self.player(to, ex, host)? {
                    host.send_title(&p, &t, s.as_deref());
                }
            }
            Effect::ActionBar { text, to } => {
                let t = self.text(text, ex, host)?;
                if let Some(p) = self.player(to, ex, host)? {
                    host.send_action_bar(&p, &t);
                }
            }
            Effect::Heal(p) => {
                if let Some(p) = self.player(p, ex, host)? {
                    host.heal(&p);
                }
            }
            Effect::SetHealth(p, n) => {
                let n = self.num(n, ex, host)?;
                if let Some(p) = self.player(p, ex, host)? {
                    host.set_health(&p, n.max(0.0));
                }
            }
            Effect::Kill(p) => {
                if let Some(p) = self.player(p, ex, host)? {
                    host.kill(&p);
                }
            }
            Effect::Teleport(p, x, y, z) => {
                let (x, y, z) = (self.num(x, ex, host)?, self.num(y, ex, host)?, self.num(z, ex, host)?);
                if let Some(p) = self.player(p, ex, host)? {
                    host.teleport(&p, x, y, z);
                }
            }
            Effect::ConsoleCommand(c) => {
                let c = self.text(c, ex, host)?;
                host.execute_command(c.trim_start_matches('/'));
            }
            Effect::Cancel => {
                if ex.waited {
                    host.log(
                        LogLevel::Warn,
                        &format!("{}:{}: 'cancel event' after a wait does nothing (the event already happened)", ex.trig.file, line),
                    );
                } else {
                    ex.cancelled = true;
                }
            }
            Effect::Wait(amount, mult) => {
                let n = self.num(amount, ex, host)?;
                if !(n >= 0.0) || !n.is_finite() {
                    return Err(format!("can't wait {}", format_num(n)));
                }
                let ticks = (n * *mult as f64).round() as u64;
                if ticks > 0 {
                    return Ok(Some(ticks));
                }
            }
            Effect::Set(v, val) => {
                let val = self.eval(val, ex, host)?;
                self.set_var(v, val, ex, host)?;
            }
            Effect::Add(v, n) | Effect::Remove(v, n) => {
                let n = self.num(n, ex, host)?;
                let cur = self.get_var(v, ex, host)?;
                let base = match cur {
                    Value::None => 0.0,
                    other => to_num_opt(&other).ok_or_else(|| format!("can't add to a variable holding {} ({})", other.type_name(), other))?,
                };
                let res = if matches!(e, Effect::Add(..)) { base + n } else { base - n };
                self.set_var(v, Value::Num(res), ex, host)?;
            }
            Effect::Delete(v) => {
                let k = self.key(v, ex, host)?;
                if v.local {
                    delete_in(&mut ex.locals, &k);
                } else {
                    self.vars.delete(&k);
                    self.mark_dirty(host);
                }
            }
        }
        Ok(None)
    }

    /// JSON to write if variables changed at least `SAVE_DEBOUNCE_MS` ago.
    pub fn save_due(&mut self, now: u64) -> Option<String> {
        let since = self.dirty_since?;
        if now.saturating_sub(since) >= SAVE_DEBOUNCE_MS {
            self.dirty_since = None;
            Some(self.vars.to_json())
        } else {
            None
        }
    }

    /// JSON to write now (unload), if anything changed.
    pub fn save_now(&mut self) -> Option<String> {
        self.dirty_since.take().map(|_| self.vars.to_json())
    }
}

fn to_num_opt(v: &Value) -> Option<f64> {
    match v {
        Value::Num(n) => Some(*n),
        Value::Text(t) => t.trim().parse().ok(),
        _ => None,
    }
}

fn parse_args(def: &CommandDef, raw: &str, host: &mut dyn Host) -> Result<Vec<Value>, String> {
    let mut rest = raw.trim();
    let mut out = Vec::new();
    let n = def.args.len();
    for (i, a) in def.args.iter().enumerate() {
        if rest.is_empty() {
            if a.optional {
                out.push(Value::None);
                continue;
            }
            return Err(format!("Usage: {}", def.usage()));
        }
        let last = i + 1 == n;
        let word = if last && a.ty == ArgTy::Text {
            let w = rest;
            rest = "";
            w
        } else {
            let (w, r) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
            rest = r.trim_start();
            w
        };
        out.push(match a.ty {
            ArgTy::Text => Value::Text(word.to_string()),
            ArgTy::Number => Value::Num(word.parse().map_err(|_| format!("'{word}' is not a number. Usage: {}", def.usage()))?),
            ArgTy::Player => Value::Player(host.find_player(word).ok_or_else(|| format!("Player '{word}' is not online"))?),
        });
    }
    if !rest.is_empty() {
        return Err(format!("Too many arguments. Usage: {}", def.usage()));
    }
    Ok(out)
}
