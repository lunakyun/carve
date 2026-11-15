//! Script table, event dispatch, the flat-instruction VM and wait/resume.

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::Arc;

use crate::compile::{
    ArgTy, CmpOp, Code, CommandDef, Cond, Effect, EventKind, Function, Instr, LoopSrc, TextOp, Trigger,
    TriggerKind, compile_all,
};
use crate::error::ScriptError;
use crate::expr::{BinOp, Expr, Key, MathFn, Part, Prop, VarRef};
use crate::host::{Host, LogLevel};
use crate::list_literals::ElemSel;
use crate::value::{PlayerRef, Recipient, Value, format_num, push_num};
use crate::vars::{Locals, Variables};

pub const DEFAULT_MAX_INSTRUCTIONS: usize = 100_000;
/// How deep functions may call each other (recursion included).
pub const MAX_CALL_DEPTH: usize = 64;
/// Error text for "already reported, just stop".
const SILENT: &str = "(reported)";
pub const SAVE_DEBOUNCE_MS: u64 = 5_000;

/// What an event offers to the scripts that handle it.
#[derive(Debug, Clone)]
pub struct EventCtx {
    pub player: Option<PlayerRef>,
    pub sender: Recipient,
    pub message: Option<String>,
    pub args: Vec<Value>,
    /// `damage` in `on damage`.
    pub damage: Option<f64>,
    /// `attacker` in `on damage` / `on death`, when it is a player.
    pub attacker: Option<PlayerRef>,
    /// `clicked block` in click events (block id without `minecraft:`).
    pub block: Option<String>,
    /// Values of the item and level events (see `events_items`).
    pub extra: Option<Box<crate::events_items::EventExtra>>,
    /// The clicked slot in `on inventory click`.
    pub inv: Option<Box<crate::inventories::InvClick>>,
    /// `event-...` values, in the order of `ExtEvent::fields` (see `w2_events_1`).
    pub ev: Vec<Value>,
    /// Values of the extra events in `w2_events_2`.
    pub ext: Option<Box<crate::w2_events_2::ExtVals>>,
    /// `death message` in `on death`.
    pub death_message: Option<String>,
    /// `damage cause` in `on damage` (lower case, like `fall`).
    pub damage_cause: Option<String>,
    /// `chat recipients` in `on chat`.
    pub recipients: Option<Vec<PlayerRef>>,
    /// Victim and attacker of a hit that are not players (see `combat`).
    pub combat: Option<Box<crate::combat::Combat>>,
}

impl EventCtx {
    pub fn player(p: PlayerRef) -> Self {
        EventCtx {
            sender: Recipient::Player(p.clone()),
            player: Some(p),
            message: None,
            args: Vec::new(),
            damage: None,
            attacker: None,
            block: None,
            extra: None,
            inv: None,
            ev: Vec::new(),
            ext: None,
            death_message: None,
            damage_cause: None,
            recipients: None,
            combat: None,
        }
    }

    /// No player: `on load`, `every`, or a console run.
    pub fn console() -> Self {
        EventCtx {
            player: None,
            sender: Recipient::Console,
            message: None,
            args: Vec::new(),
            damage: None,
            attacker: None,
            block: None,
            extra: None,
            inv: None,
            ev: Vec::new(),
            ext: None,
            death_message: None,
            damage_cause: None,
            recipients: None,
            combat: None,
        }
    }

    pub fn chat(p: PlayerRef, message: impl Into<String>) -> Self {
        let mut c = Self::player(p);
        c.message = Some(message.into());
        c
    }
}

/// What one loaded script file contains, for `/carve list` and `/carve info`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FileInfo {
    pub name: String,
    /// Trigger headers, like `on join`, `command /home`, `every 100 ticks`.
    pub triggers: Vec<String>,
    pub commands: Vec<String>,
    pub functions: Vec<String>,
}

/// Run counts and times of one trigger, for `/carve timings`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Timing {
    pub calls: u64,
    pub total: std::time::Duration,
    pub max: std::time::Duration,
}

/// `on join`, `command /home`, `every 100 ticks`.
pub fn trigger_label(t: &Trigger) -> String {
    match &t.kind {
        TriggerKind::Event(k) => format!("on {}", k.name()),
        TriggerKind::Command(d) => format!("command /{}", d.name),
        TriggerKind::Every(n) => format!("every {n} ticks"),
    }
}

/// Hasher for the event map. `EventKind` hashes as one or two small
/// integers, and SipHash cost more than the rest of an event with no triggers.
#[derive(Default, Clone, Copy)]
pub(crate) struct KindHasher(u64);

impl KindHasher {
    #[inline]
    fn add(&mut self, x: u64) {
        self.0 = (self.0.rotate_left(5) ^ x).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
}

impl Hasher for KindHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.add(u64::from(b));
        }
    }
    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add(u64::from(i));
    }
    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add(u64::from(i));
    }
    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add(u64::from(i));
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add(i);
    }
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add(i as u64);
    }
    #[inline]
    fn write_isize(&mut self, i: isize) {
        self.add(i as u64);
    }
}

#[derive(Debug, Default)]
pub struct ScriptTable {
    events: HashMap<EventKind, Vec<Arc<Trigger>>, BuildHasherDefault<KindHasher>>,
    commands: BTreeMap<String, Arc<Trigger>>,
    functions: Vec<Option<Arc<Function>>>,
    every: Vec<(u64, Arc<Trigger>)>,
    /// `variables:` defaults: (applies per player, code).
    pub(crate) var_defaults: Vec<(bool, Arc<Trigger>)>,
    pub file_info: Vec<FileInfo>,
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
            TriggerKind::Event(_) | TriggerKind::Every(_) => None,
        }
    }

    pub fn has_event(&self, k: EventKind) -> bool {
        self.events.get(&k).is_some_and(|v| !v.is_empty())
    }

    pub fn function_count(&self) -> usize {
        self.functions.iter().filter(|f| f.is_some()).count()
    }

    /// Any `at <time>:` trigger.
    pub fn has_at(&self) -> bool {
        self.events
            .keys()
            .any(|k| matches!(k, EventKind::Ext(crate::w2_events_1::ExtEvent::At(_))))
    }

    pub fn has_every(&self) -> bool {
        !self.every.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadSummary {
    pub files: usize,
    pub triggers: usize,
    /// Functions that loaded.
    pub functions: usize,
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
    /// Values changed with `set event-<name> to ...` (slot, new value).
    pub set: Vec<(u8, Value)>,
}

/// One running loop. Items are a snapshot taken when the loop starts.
struct LoopFrame {
    items: LoopItems,
    /// Items handed out so far; the current item is `pos - 1`.
    pos: usize,
}

enum LoopItems {
    Times(usize),
    Values(Vec<Value>),
    List(Vec<(String, Value)>),
}

impl LoopFrame {
    fn len(&self) -> usize {
        match &self.items {
            LoopItems::Times(n) => *n,
            LoopItems::Values(v) => v.len(),
            LoopItems::List(l) => l.len(),
        }
    }

    fn value(&self) -> Value {
        let i = self.pos.saturating_sub(1);
        match &self.items {
            LoopItems::Times(_) => Value::Num(self.pos as f64),
            LoopItems::Values(v) => v.get(i).cloned().unwrap_or(Value::None),
            LoopItems::List(l) => l.get(i).map(|(_, v)| v.clone()).unwrap_or(Value::None),
        }
    }

    /// A `loop N times`, whose value is its counter.
    fn is_times(&self) -> bool {
        matches!(self.items, LoopItems::Times(_))
    }

    fn index(&self) -> Value {
        match &self.items {
            LoopItems::List(l) => l
                .get(self.pos.saturating_sub(1))
                .map(|(k, _)| Value::Text(k.clone()))
                .unwrap_or(Value::None),
            _ => Value::Num(self.pos as f64),
        }
    }
}

/// `a, b and c`, like Skript prints a list.
mod ext_eval;
mod gaps_eval;
mod list_eval;
mod structures_rt;
pub(crate) use structures_rt::ERROR_KEY;
mod w2;
mod w2_eval;
mod w2_run;
mod w2e3;
mod w4_eval;
mod w5_eval;
mod w5c2_eval;
mod w6_eval;
mod x2_eval;

fn join_list(items: &[(String, Value)], out: &mut String) {
    if items.is_empty() {
        out.push_str("<none>");
    }
    for (i, (_, v)) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(if i + 1 == items.len() { " and " } else { ", " });
        }
        v.push_to(out);
    }
}

/// Compare two numbers the way `loose_eq` and `cmp_num` do.
#[inline]
fn cmp_nums(x: f64, op: CmpOp, y: f64) -> bool {
    let eq = (x - y).abs() < 1e-9;
    match op {
        CmpOp::Eq => eq,
        CmpOp::Ne => !eq,
        CmpOp::Gt => x > y,
        CmpOp::Lt => x < y,
        CmpOp::Ge => x >= y,
        CmpOp::Le => x <= y,
    }
}

/// A caller's state, saved while a function runs.
struct Frame {
    code: Arc<Code>,
    pc: usize,
    locals: Locals,
    loops: Vec<LoopFrame>,
}

pub(crate) struct Exec {
    /// The trigger this run started from (for timings).
    root: Arc<Code>,
    /// The code running now (a trigger or a function body).
    code: Arc<Code>,
    pc: usize,
    pub(crate) locals: Locals,
    loops: Vec<LoopFrame>,
    /// Callers of the running function, innermost last.
    callers: Vec<Frame>,
    pub(crate) ctx: EventCtx,
    pub(crate) waited: bool,
    cancelled: bool,
    /// Open `catch runtime errors:` sections, innermost last.
    catches: Vec<structures_rt::Catch>,
    /// uuid -> current name (None = offline), for the current run slice only.
    /// Cleared at every wait, because a player can leave while a run waits.
    pub(crate) seen: Vec<(String, Option<String>)>,
    /// `last spawned entity`.
    pub(crate) last_entity: Option<Box<crate::value::EntityRef>>,
    /// Locals of the last function that returned, emptied, so the next call
    /// can reuse their memory.
    spare: Option<Locals>,
}

impl Exec {
    fn new(trig: &Trigger, ctx: EventCtx) -> Self {
        // The event's own player counts as online until the first wait, even if
        // the server already dropped them (`on quit` runs after the player left).
        let seen = ctx
            .player
            .iter()
            .chain(ctx.attacker.iter())
            .map(|p| (p.uuid.clone(), Some(p.name.clone())))
            .collect();
        Exec {
            root: trig.code.clone(),
            code: trig.code.clone(),
            pc: 0,
            locals: Locals::default(),
            loops: Vec::new(),
            callers: Vec::new(),
            ctx,
            waited: false,
            cancelled: false,
            catches: Vec::new(),
            seen,
            last_entity: None,
            spare: None,
        }
    }

    /// Enter a function body whose parameters are already in `locals`.
    fn push_call(&mut self, f: &Function, locals: Locals) {
        let caller = Frame {
            code: std::mem::replace(&mut self.code, f.code.clone()),
            pc: std::mem::replace(&mut self.pc, 0),
            locals: std::mem::replace(&mut self.locals, locals),
            loops: std::mem::take(&mut self.loops),
        };
        self.callers.push(caller);
    }

    /// Leave the running function and continue in its caller.
    fn pop_call(&mut self) {
        if let Some(f) = self.callers.pop() {
            self.code = f.code;
            self.pc = f.pc;
            let mut done = std::mem::replace(&mut self.locals, f.locals);
            done.clear();
            self.spare = Some(done);
            self.loops = f.loops;
            let depth = self.callers.len();
            if !self.catches.is_empty() {
                self.catches.retain(|c| c.depth <= depth);
            }
        }
    }
}

/// The player's current name, asking the host at most once per run slice.
fn resolve<'c>(
    seen: &'c mut Vec<(String, Option<String>)>,
    uuid: &str,
    host: &mut dyn Host,
) -> Option<&'c str> {
    let i = match seen.iter().position(|(u, _)| u == uuid) {
        Some(i) => i,
        None => {
            let name = host.resolve_player(uuid);
            seen.push((uuid.to_string(), name));
            seen.len() - 1
        }
    };
    seen.get(i).and_then(|(_, n)| n.as_deref())
}

/// Resolve a value to an online player, or None.
pub(crate) fn online(
    v: &Value,
    seen: &mut Vec<(String, Option<String>)>,
    host: &mut dyn Host,
) -> Option<PlayerRef> {
    match v {
        Value::Player(p) => resolve(seen, &p.uuid, host).map(|name| PlayerRef {
            uuid: p.uuid.clone(),
            name: name.to_string(),
        }),
        Value::Text(name) => host.find_player(name),
        _ => None,
    }
}

enum Step {
    Done,
    Wait(u64),
    Abort,
    /// A function called as a value returned.
    Returned(Value),
}

pub struct Engine {
    table: Arc<ScriptTable>,
    waits: HashMap<u64, Exec>,
    next_token: u64,
    pub vars: Variables,
    pub max_instructions: usize,
    /// `variables:` defaults still to apply (after a load or reload).
    pending_defaults: bool,
    /// Message of the last error that an outer `catch runtime errors` will take.
    caught_stash: String,
    dirty_since: Option<u64>,
    /// Instructions left in the current run slice (shared by function calls).
    budget: usize,
    /// xorshift state for `random`; seeded from the host clock on first use.
    rng: u64,
    /// Sources of the files that are loaded now, by file name.
    sources: BTreeMap<String, String>,
    /// Errors of the last load or reload.
    pub errors: Vec<ScriptError>,
    /// Per-trigger run times while `/carve timings on`.
    pub timings: Option<BTreeMap<String, Timing>>,
    /// SkBee-style bounds (see `skbee_bound`).
    pub bounds: crate::skbee_bound::Bounds,
    /// Sidebar boards by player uuid (not saved).
    pub(crate) boards: crate::skbee_scoreboard_teams::Boards,
    /// Event values changed by the run in progress (`w2_events_4`).
    pub(crate) ext_out: Vec<(u8, Value)>,
    w5: crate::w2_expressions_5::State,
    pub(crate) w5c1: Box<crate::w5_chunk1::State>,
    /// Nesting of placeholder scripts that parse placeholders themselves.
    ph_depth: u8,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(Variables::default())
    }
}

fn build_table(files: &[(String, String)]) -> (ScriptTable, Vec<ScriptError>) {
    let mut table = ScriptTable::default();
    let compiled = compile_all(files);
    let mut errors = compiled.errors;
    table.functions = compiled.functions;
    table.var_defaults = compiled
        .var_defaults
        .into_iter()
        .map(|(p, t)| (p, Arc::new(t)))
        .collect();
    for (name, trigs) in compiled.files {
        table.files += 1;
        let mut info = FileInfo {
            name: name.clone(),
            ..FileInfo::default()
        };
        for f in table.functions.iter().flatten() {
            if *f.code.file == *name {
                let params: Vec<String> = f
                    .sig
                    .params
                    .iter()
                    .map(|(k, t, _)| format!("{}: {}", k.trim_start_matches('_'), t.name()))
                    .collect();
                let ret = f.sig.ret.map(|r| format!(" :: {}", r.name())).unwrap_or_default();
                info.functions
                    .push(format!("{}({}){ret}", f.sig.name, params.join(", ")));
            }
        }
        for t in &trigs {
            info.triggers.push(trigger_label(t));
            if let TriggerKind::Command(d) = &t.kind {
                info.commands.push(format!("/{}", d.name));
            }
        }
        table.file_info.push(info);
        for t in trigs {
            table.triggers += 1;
            let t = Arc::new(t);
            match &t.kind {
                TriggerKind::Event(k) => table.events.entry(*k).or_default().push(t.clone()),
                TriggerKind::Every(ticks) => table.every.push((*ticks, t.clone())),
                TriggerKind::Command(d) => {
                    if table.commands.contains_key(&d.name) {
                        errors.push(ScriptError::new(
                            &t.file,
                            t.line,
                            format!("command /{} is defined twice", d.name),
                        ));
                    } else {
                        table.commands.insert(d.name.clone(), t.clone());
                        for a in &d.aliases {
                            if table.commands.contains_key(a) {
                                errors.push(ScriptError::new(
                                    &t.file,
                                    t.line,
                                    format!("alias /{a} is already a command"),
                                ));
                            } else {
                                table.commands.insert(a.clone(), t.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    crate::error::fill_code(&mut errors, files);
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
            pending_defaults: false,
            caught_stash: String::new(),
            dirty_since: None,
            budget: 0,
            rng: 0,
            sources: BTreeMap::new(),
            errors: Vec::new(),
            timings: None,
            bounds: crate::skbee_bound::Bounds::default(),
            boards: Default::default(),
            ext_out: Vec::new(),
            w5: Default::default(),
            w5c1: Default::default(),
            ph_depth: 0,
        }
    }

    /// Names of the files that are loaded now.
    pub fn loaded_files(&self) -> Vec<String> {
        self.sources.keys().cloned().collect()
    }

    /// Reload one file (or unload it with `None`), keeping every other file.
    /// Like Skript, the new version replaces the old one even with errors: what
    /// parsed is loaded, the errors (of every file) are returned.
    pub fn reload_file(&mut self, name: &str, src: Option<String>) -> (LoadSummary, Vec<ScriptError>) {
        let mut candidate = self.sources.clone();
        match src {
            Some(s) => candidate.insert(name.to_string(), s),
            None => candidate.remove(name),
        };
        let files: Vec<(String, String)> = candidate.into_iter().collect();
        self.reload(&files)
    }

    /// Mark the variables changed (saved after the debounce), for admin edits.
    pub fn vars_changed(&mut self, host: &mut dyn Host) {
        self.mark_dirty(host);
    }

    /// Run one line of Carve right now, as the given sender. Errors in the
    /// line are returned with line 1 of file `run`.
    pub fn run_line(
        &mut self,
        line: &str,
        sender: Recipient,
        host: &mut dyn Host,
    ) -> Result<(), Vec<ScriptError>> {
        let src = format!(
            "command /carve-run-line:\n    trigger:\n        {}\n",
            line.trim()
        );
        let mut files: Vec<(String, String)> =
            self.sources.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        files.push(("run".to_string(), src));
        let compiled = crate::compile::compile_all(&files);
        let mine: Vec<ScriptError> = compiled
            .errors
            .into_iter()
            .filter(|e| e.file == "run")
            .map(|mut e| {
                e.line = 1;
                e
            })
            .collect();
        if !mine.is_empty() {
            return Err(mine);
        }
        let Some(trig) = compiled
            .files
            .into_iter()
            .find(|(n, _)| n == "run")
            .and_then(|(_, t)| t.into_iter().next())
        else {
            return Err(vec![ScriptError::new("run", 1, "nothing to run")]);
        };
        let mut ctx = EventCtx::console();
        if let Recipient::Player(p) = &sender {
            ctx.player = Some(p.clone());
        }
        ctx.sender = sender;
        let ex = Exec::new(&trig, ctx);
        self.start(ex, host);
        Ok(())
    }

    pub fn table(&self) -> &ScriptTable {
        &self.table
    }

    fn summary(&self) -> LoadSummary {
        LoadSummary {
            files: self.table.files,
            triggers: self.table.triggers,
            functions: self.table.function_count(),
            commands: self.table.command_names(),
        }
    }

    /// Load scripts, like Skript: a broken line, section or trigger is
    /// reported and left out, everything that parsed is used.
    pub fn load(&mut self, files: &[(String, String)]) -> (LoadSummary, Vec<ScriptError>) {
        let (table, errors) = build_table(files);
        self.sources = files.iter().cloned().collect();
        self.pending_defaults = !table.var_defaults.is_empty();
        self.table = Arc::new(table);
        self.errors = errors.clone();
        (self.summary(), errors)
    }

    /// Reload: replaces the old scripts with what parsed (see [`load`](Self::load)).
    /// Runs that are waiting keep the code they started with.
    pub fn reload(&mut self, files: &[(String, String)]) -> (LoadSummary, Vec<ScriptError>) {
        self.load(files)
    }

    pub fn pending_waits(&self) -> usize {
        self.waits.len()
    }

    /// Run every trigger for `kind`. Returns whether the event got cancelled
    /// (only `cancel event` before the first `wait` counts).
    pub fn fire(&mut self, kind: EventKind, ctx: EventCtx, host: &mut dyn Host) -> FireOutcome {
        if self.pending_defaults || (kind == EventKind::Join && !self.table.var_defaults.is_empty()) {
            self.defaults_for(kind, &ctx, host);
        }
        let table = self.table.clone();
        let mut cancelled = false;
        self.ext_out.clear();
        let trigs = table.events.get(&kind).map(Vec::as_slice).unwrap_or_default();
        let mut ctx = Some(ctx);
        for (i, trig) in trigs.iter().enumerate() {
            // The last trigger gets the ctx itself, the others a copy.
            let c = if i + 1 == trigs.len() {
                ctx.take()
            } else {
                ctx.clone()
            };
            let Some(c) = c else { break };
            let ex = Exec::new(trig, c);
            if let Some(c) = self.start(ex, host) {
                cancelled |= c;
            }
            if !self.ext_out.is_empty()
                && let Some(c) = ctx.as_mut()
            {
                crate::combat::carry(c, &self.ext_out);
            }
        }
        FireOutcome {
            cancelled,
            set: std::mem::take(&mut self.ext_out),
        }
    }

    /// Run the `at <time>:` triggers whose time the world clock passed when it
    /// went from `prev` to `cur` (ticks of the day). Call once per tick when
    /// `table().has_at()`.
    pub fn fire_at_time(&mut self, prev: i64, cur: i64, host: &mut dyn Host) {
        use crate::w2_events_1::{ExtEvent, crossed};
        let table = self.table.clone();
        for k in table.events.keys() {
            if let EventKind::Ext(ExtEvent::At(t)) = k
                && crossed(prev, cur, *t)
            {
                self.fire(*k, EventCtx::console(), host);
            }
        }
    }

    /// Run a script command. `raw_args` is everything after the command name.
    pub fn run_command(
        &mut self,
        name: &str,
        sender: Recipient,
        raw_args: &str,
        host: &mut dyn Host,
    ) -> CommandOutcome {
        if self.pending_defaults {
            self.apply_defaults(host);
        }
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
        match (def.players_only, &player) {
            (Some(true), None) => return CommandOutcome::Usage("Only players can use this command.".into()),
            (Some(false), Some(_)) => {
                return CommandOutcome::Usage("Only the console can use this command.".into());
            }
            _ => {}
        }
        let args = match parse_args(def, raw_args, host) {
            Ok(a) => a,
            Err(msg) => {
                return CommandOutcome::Usage(match &def.usage_msg {
                    Some(u) if !msg.starts_with("Player '") => u.clone(),
                    _ => msg,
                });
            }
        };
        if let Some(cd) = &def.cooldown
            && let Some(msg) = self.w5c1_cooldown_gate(&name, cd, player.as_ref(), host)
        {
            return CommandOutcome::Usage(msg);
        }
        let mut ctx = EventCtx::console();
        ctx.player = player;
        ctx.sender = sender;
        ctx.args = args;
        let ex = Exec::new(&trig, ctx);
        self.start(ex, host);
        if def.cooldown.is_some() {
            self.w5c1_command_done();
        }
        if let Some(msg) = self.w5c1.failed.take() {
            return CommandOutcome::Usage(msg);
        }
        CommandOutcome::Ran
    }

    /// Call once per server tick with a counter that goes up by one. Runs the
    /// `every ...:` triggers that are due.
    pub fn tick(&mut self, tick: u64, host: &mut dyn Host) {
        if tick.is_multiple_of(crate::skbee_bound::POLL_TICKS) && self.bounds_wanted() {
            self.bounds_poll(host);
        }
        if self.pending_defaults {
            self.apply_defaults(host);
        }
        if self.table.every.is_empty() {
            return;
        }
        let table = self.table.clone();
        for (period, trig) in &table.every {
            if *period > 0 && tick.is_multiple_of(*period) {
                let ex = Exec::new(trig, EventCtx::console());
                self.start(ex, host);
            }
        }
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
        self.budget = self.max_instructions;
        let started = self.timings.as_ref().map(|_| std::time::Instant::now());
        let step = self.run(&mut ex, host, 0);
        if let (Some(t0), Some(map)) = (started, self.timings.as_mut()) {
            let took = t0.elapsed();
            let label = format!("{}:{}", ex.root.file, ex.root.line);
            let e = map.entry(label).or_default();
            e.calls += 1;
            e.total += took;
            e.max = e.max.max(took);
        }
        match step {
            Step::Done | Step::Returned(_) => {}
            Step::Wait(ticks) => {
                let cancelled = ex.cancelled;
                ex.waited = true;
                ex.seen.clear();
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

    /// Run until the run ends, waits, fails, or (for `base > 0`) the function
    /// entered at depth `base` returns.
    fn run(&mut self, ex: &mut Exec, host: &mut dyn Host, base: usize) -> Step {
        loop {
            let code = ex.code.clone();
            let depth = ex.callers.len();
            while ex.callers.len() == depth {
                let Some(instr) = code.instrs.get(ex.pc) else {
                    // End of the body: an implicit `return` with no value.
                    if let Some(step) = self.leave(ex, base, Value::None) {
                        return step;
                    }
                    break;
                };
                let line = code.lines.get(ex.pc).copied().unwrap_or(code.line);
                if self.budget == 0 {
                    host.log(
                        LogLevel::Error,
                        &format!(
                            "{}:{}: script stopped: ran more than {} instructions without a wait",
                            code.file, line, self.max_instructions
                        ),
                    );
                    return Step::Abort;
                }
                self.budget -= 1;
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
                    Instr::LoopInit(src) => self.loop_init(src, ex, host).map(|()| None),
                    Instr::LoopNext(end) => {
                        match ex.loops.last_mut() {
                            Some(f) if f.pos < f.len() => f.pos += 1,
                            _ => {
                                ex.loops.pop();
                                ex.pc = *end;
                            }
                        }
                        Ok(None)
                    }
                    Instr::LoopBreak(end) => {
                        ex.loops.pop();
                        ex.pc = *end;
                        Ok(None)
                    }
                    Instr::Stop => {
                        if ex.callers.is_empty() {
                            return Step::Done;
                        }
                        if let Some(step) = self.leave(ex, base, Value::None) {
                            return step;
                        }
                        Ok(None)
                    }
                    Instr::Return(e) => {
                        let v = match e {
                            Some(e) => self.eval(e, ex, host),
                            None => Ok(Value::None),
                        };
                        match v {
                            Ok(v) => {
                                if let Some(step) = self.leave(ex, base, v) {
                                    return step;
                                }
                                Ok(None)
                            }
                            Err(e) => Err(e),
                        }
                    }
                    Instr::Call(id, args) => self.enter(*id, args, ex, host).map(|()| None),
                    Instr::CatchBegin(end) => {
                        structures_rt::catch_begin(ex, *end);
                        Ok(None)
                    }
                    Instr::CatchEnd => {
                        ex.catches.pop();
                        Ok(None)
                    }
                    Instr::StopUnless(c) => match self.cond(c, ex, host) {
                        Ok(true) => Ok(None),
                        Ok(false) => {
                            if ex.callers.is_empty() {
                                return Step::Done;
                            }
                            if let Some(step) = self.leave(ex, base, Value::None) {
                                return step;
                            }
                            Ok(None)
                        }
                        Err(e) => Err(e),
                    },
                };
                match r {
                    Ok(None) => {}
                    Ok(Some(ticks)) => {
                        if base > 0 {
                            host.log(
                                LogLevel::Error,
                                &format!(
                                    "{}:{}: a function used as a value can't wait (call it on its own line instead)",
                                    code.file, line
                                ),
                            );
                            return Step::Abort;
                        }
                        return Step::Wait(ticks);
                    }
                    Err(e) => {
                        if !ex.catches.is_empty() && self.catch_error(ex, &e, base) {
                            continue;
                        }
                        if e != SILENT && ex.catches.is_empty() {
                            host.log(LogLevel::Error, &format!("{}:{}: {}", code.file, line, e));
                        }
                        return Step::Abort;
                    }
                }
            }
        }
    }

    /// Return from the running body. `Some(step)` when the run (or the
    /// value call at depth `base`) is over.
    fn leave(&mut self, ex: &mut Exec, base: usize, v: Value) -> Option<Step> {
        if ex.callers.is_empty() {
            return Some(Step::Done);
        }
        let ends_value_call = base > 0 && ex.callers.len() == base;
        ex.pop_call();
        ends_value_call.then_some(Step::Returned(v))
    }

    /// Evaluate arguments (filling defaults) and enter function `id`.
    fn enter(&mut self, id: usize, args: &[Expr], ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        let f = match self.table.functions.get(id) {
            Some(Some(f)) => f.clone(),
            _ => return Err("this function failed to load; see the load errors".into()),
        };
        if ex.callers.len() >= MAX_CALL_DEPTH {
            return Err(format!(
                "functions called each other more than {MAX_CALL_DEPTH} levels deep (endless recursion?)"
            ));
        }
        // Arguments are evaluated in the caller, straight into the new locals.
        let mut locals = ex.spare.take().unwrap_or_default();
        for (i, (d, (key, _, _))) in f.defaults.iter().zip(&f.sig.params).enumerate() {
            let v = match (args.get(i), d) {
                (Some(a), _) => self.eval(a, ex, host)?,
                (None, Some(d)) => self.eval(d, ex, host)?,
                (None, None) => Value::None,
            };
            if let Value::List(items) = v {
                for (i, x) in items.into_iter().enumerate() {
                    locals.set(&format!("{key}::{}", i + 1), x);
                }
            } else {
                locals.set(key, v);
            }
        }
        ex.push_call(&f, locals);
        Ok(())
    }

    /// A function call used as a value: run it to its `return` right now.
    fn call_value(
        &mut self,
        id: usize,
        args: &[Expr],
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        self.enter(id, args, ex, host)?;
        let base = ex.callers.len();
        match self.run(ex, host, base) {
            Step::Returned(v) => Ok(v),
            // The error was already logged with the function's line.
            _ => {
                while ex.callers.len() >= base {
                    ex.pop_call();
                }
                Err(SILENT.into())
            }
        }
    }

    /// A random number in [0, 1).
    fn next_random(&mut self, host: &mut dyn Host) -> f64 {
        if self.rng == 0 {
            self.rng = host.now().wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        }
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        (x >> 11) as f64 / (1u64 << 53) as f64
    }

    /// A location value, or the location of a player value.
    pub(crate) fn as_location(
        &mut self,
        v: &Value,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Option<(f64, f64, f64, Option<String>)> {
        match v {
            Value::Location { x, y, z, world } => Some((*x, *y, *z, world.clone())),
            Value::Player(_) | Value::Text(_) => {
                let p = online(v, &mut ex.seen, host)?;
                host.player_location(&p).map(|(x, y, z, w)| (x, y, z, Some(w)))
            }
            Value::Entity(e) => host.entity_location(e).map(|(x, y, z, w)| (x, y, z, Some(w))),
            _ => None,
        }
    }

    fn property(&mut self, prop: Prop, v: &Value, ex: &mut Exec, host: &mut dyn Host) -> Value {
        if let Value::Chunk { world, .. } = v
            && prop == Prop::World
        {
            return world
                .clone()
                .or_else(|| host.default_world())
                .map_or(Value::None, Value::World);
        }
        if let Value::Entity(e) = v
            && let Some(r) = self.entity_prop(prop, e, host)
        {
            return r;
        }
        if let Value::Quaternion { .. } = v
            && let Some(r) = crate::w6_chunk6::quat_comp(prop, v)
        {
            return r;
        }
        if let Value::Vector { .. } = v
            && let Some(r) = crate::w2_expressions_7::component(prop, v)
        {
            return r;
        }
        match prop {
            Prop::X | Prop::Y | Prop::Z | Prop::Location | Prop::World => {
                let Some((x, y, z, world)) = self.as_location(v, ex, host) else {
                    return Value::None;
                };
                match prop {
                    Prop::X => Value::Num(x),
                    Prop::Y => Value::Num(y),
                    Prop::Z => Value::Num(z),
                    Prop::World => world.map(Value::World).unwrap_or(Value::None),
                    _ => Value::Location { x, y, z, world },
                }
            }
            Prop::Uuid => match v {
                Value::Player(p) => Value::Text(p.uuid.clone()),
                Value::Text(_) => online(v, &mut ex.seen, host)
                    .map(|p| Value::Text(p.uuid))
                    .unwrap_or(Value::None),
                _ => Value::None,
            },
            Prop::Name => online(v, &mut ex.seen, host)
                .map(|p| Value::Text(p.name))
                .unwrap_or(Value::None),
            Prop::Tool => {
                let p = online(v, &mut ex.seen, host);
                match p.as_ref().and_then(|p| host.held_item(p)) {
                    Some((id, count)) => {
                        let meta = p.and_then(|p| host.held_item_meta(&p)).map(Box::new);
                        Value::Item { id, count, meta }
                    }
                    None => Value::Item {
                        id: "air".into(),
                        count: 1,
                        meta: None,
                    },
                }
            }
            Prop::Gamemode => online(v, &mut ex.seen, host)
                .and_then(|p| host.gamemode(&p))
                .map(Value::Text)
                .unwrap_or(Value::None),
            Prop::DisplayName | Prop::TabName | Prop::Ping | Prop::Ip => online(v, &mut ex.seen, host)
                .and_then(|p| crate::op_kick_ban::read_prop(host, prop, &p))
                .unwrap_or(Value::None),
            Prop::Health | Prop::MaxHealth | Prop::Food | Prop::Saturation => online(v, &mut ex.seen, host)
                .and_then(|p| host.vital(&p, crate::health_food::Vital::of_prop(prop)?))
                .map(Value::Num)
                .unwrap_or(Value::None),
            Prop::Level | Prop::Xp | Prop::LevelProgress => {
                let field = match prop {
                    Prop::Level => crate::xp_level::XpField::Level,
                    Prop::Xp => crate::xp_level::XpField::Points,
                    _ => crate::xp_level::XpField::Progress,
                };
                online(v, &mut ex.seen, host)
                    .and_then(|p| host.xp_get(&p, field))
                    .map(Value::Num)
                    .unwrap_or(Value::None)
            }
        }
    }

    fn loop_init(&mut self, src: &LoopSrc, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        let items = match src {
            LoopSrc::Times(n) => {
                let n = self.num(n, ex, host)?;
                LoopItems::Times(if n.is_finite() && n > 0.0 {
                    n.floor() as usize
                } else {
                    0
                })
            }
            LoopSrc::Values(es) => LoopItems::Values(self.many(es, ex, host)?),
            LoopSrc::Players => {
                let players = host.online_players();
                // They are online right now: no need to ask again in this slice.
                for p in &players {
                    if !ex.seen.iter().any(|(u, _)| *u == p.uuid) {
                        ex.seen.push((p.uuid.clone(), Some(p.name.clone())));
                    }
                }
                LoopItems::Values(players.into_iter().map(Value::Player).collect())
            }
            LoopSrc::Split(t, s) => {
                let t = self.text(t, ex, host)?;
                let s = self.text(s, ex, host)?;
                LoopItems::Values(crate::text_join_split::split_text(&t, &s))
            }
            LoopSrc::Items(s) => {
                let v = self.eval_list(s, ex, host)?;
                LoopItems::List(
                    v.into_iter()
                        .enumerate()
                        .map(|(i, x)| ((i + 1).to_string(), x))
                        .collect(),
                )
            }
            LoopSrc::List(v) => {
                let key = self.key(v, ex, host)?;
                let prefix = key.strip_suffix('*').unwrap_or(&key);
                LoopItems::List(if v.local {
                    ex.locals.list(prefix)
                } else {
                    self.vars.list(prefix)
                })
            }
        };
        ex.loops.push(LoopFrame { items, pos: 0 });
        Ok(())
    }

    /// Items of a list variable.
    pub(crate) fn list_items(
        &mut self,
        v: &VarRef,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Vec<(String, Value)>, String> {
        let key = self.key(v, ex, host)?;
        let prefix = key.strip_suffix('*').unwrap_or(&key);
        Ok(if v.local {
            ex.locals.list(prefix)
        } else {
            self.vars.list(prefix)
        })
    }

    #[inline(always)]
    fn cond(&mut self, c: &Cond, ex: &mut Exec, host: &mut dyn Host) -> Result<bool, String> {
        if let Cond::Cmp(a, op, b) = c {
            // Fast path: `{var} = 5`, `{var} > 2`... read the variable in place.
            if let (Expr::Var(v), Expr::Num(n)) = (a, b)
                && !v.list
            {
                let k = self.key(v, ex, host)?;
                let cur = if v.local {
                    ex.locals.get(&k)
                } else {
                    self.vars.get_ref(&k)
                };
                if let Some(Value::Num(x)) = cur {
                    return Ok(cmp_nums(*x, *op, *n));
                }
            }
            // `loop-number is -3`: compare the loop counter without building a value.
            if let (Expr::LoopValue(d) | Expr::LoopIter(d), Expr::Num(n)) = (a, b)
                && let Some(f) = ex.loops.get(*d)
                && (matches!(a, Expr::LoopIter(_)) || f.is_times())
            {
                return Ok(cmp_nums(f.pos as f64, *op, *n));
            }
            let a = self.eval(a, ex, host)?;
            let b = self.eval(b, ex, host)?;
            return Ok(match op {
                CmpOp::Eq => a.loose_eq(&b),
                CmpOp::Ne => !a.loose_eq(&b),
                _ => {
                    let (Some(x), Some(y)) = (crate::timespans::cmp_num(&a), crate::timespans::cmp_num(&b))
                    else {
                        return Ok(false);
                    };
                    match op {
                        CmpOp::Gt => x > y,
                        CmpOp::Lt => x < y,
                        CmpOp::Ge => x >= y,
                        _ => x <= y,
                    }
                }
            });
        }
        self.cond_rare(c, ex, host)
    }

    #[inline(never)]
    fn cond_rare(&mut self, c: &Cond, ex: &mut Exec, host: &mut dyn Host) -> Result<bool, String> {
        match c {
            Cond::Cmp(a, op, b) => {
                let a = self.eval(a, ex, host)?;
                let b = self.eval(b, ex, host)?;
                Ok(match op {
                    CmpOp::Eq => a.loose_eq(&b),
                    CmpOp::Ne => !a.loose_eq(&b),
                    _ => {
                        let (Some(x), Some(y)) =
                            (crate::timespans::cmp_num(&a), crate::timespans::cmp_num(&b))
                        else {
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
            Cond::Chance(p) => {
                let p = self.num(p, ex, host)?;
                Ok(self.next_random(host) * 100.0 < p)
            }
            Cond::HasItem { player, item, negate } => {
                let p = self.eval(player, ex, host)?;
                let item = self.eval(item, ex, host)?;
                let Some(p) = online(&p, &mut ex.seen, host) else {
                    return Ok(false);
                };
                let Value::Item { id, count, .. } = item else {
                    return Err(format!("expected an item, got {} ({item})", item.type_name()));
                };
                Ok((host.count_item(&p, &id) >= count.max(1)) != *negate)
            }
            Cond::ItemEnch { item, ench, negate } => {
                let item = self.eval(item, ex, host)?;
                let spec = match ench {
                    Some(e) => Some(self.text(e, ex, host)?),
                    None => None,
                };
                Ok(crate::item_meta::has_enchant(&item, spec.as_deref()) != *negate)
            }
            Cond::All(parts) => {
                for p in parts {
                    if !self.cond(p, ex, host)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Cond::Any(parts) => {
                for p in parts {
                    if self.cond(p, ex, host)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Cond::AnyOf {
                value,
                options,
                negate,
            } => {
                let v = self.eval(value, ex, host)?;
                let mut hit = false;
                for o in options {
                    if v.loose_eq(&self.eval(o, ex, host)?) {
                        hit = true;
                        break;
                    }
                }
                Ok(hit != *negate)
            }
            Cond::Between {
                value,
                low,
                high,
                negate,
            } => {
                let v = self.eval(value, ex, host)?;
                let (lo, hi) = (self.num(low, ex, host)?, self.num(high, ex, host)?);
                let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
                Ok(to_num_opt(&v).is_some_and(|n| n >= lo && n <= hi) != *negate)
            }
            Cond::Text {
                left,
                op,
                right,
                negate,
            } => {
                let needle = self.text(right, ex, host)?.to_lowercase();
                // `{list::*} contains x` checks the items.
                if let (Expr::Var(v), TextOp::Contains) = (left, op)
                    && v.list
                {
                    let items = self.list_items(v, ex, host)?;
                    let hit = items
                        .iter()
                        .any(|(_, item)| item.to_string().to_lowercase() == needle);
                    return Ok(hit != *negate);
                }
                let hay = self.text(left, ex, host)?.to_lowercase();
                let hit = match op {
                    TextOp::Contains => hay.contains(&needle),
                    TextOp::StartsWith => hay.starts_with(&needle),
                    TextOp::EndsWith => hay.ends_with(&needle),
                };
                Ok(hit != *negate)
            }
            Cond::Rare(r) => {
                let a = self.eval(&r.a, ex, host)?;
                let b = match &r.b {
                    Some(b) => Some(self.eval(b, ex, host)?),
                    None => None,
                };
                let hit = if r.kind == crate::w2_conditions_2::W2Kind::ScriptCommand {
                    match &a {
                        Value::Text(t) => self.table.command(t.trim().trim_start_matches('/')).is_some(),
                        _ => false,
                    }
                } else {
                    crate::w2_conditions_2::check(r.kind, &a, b.as_ref(), host)
                };
                Ok(hit != r.negate)
            }
            Cond::IsSet(e, want) => {
                let v = self.eval(e, ex, host)?;
                Ok((v != Value::None) == *want)
            }
            Cond::SenderIs {
                value,
                console,
                negate,
            } => {
                let v = self.eval(value, ex, host)?;
                let is = if *console {
                    matches!(v, Value::Console)
                } else {
                    matches!(v, Value::Player(_))
                };
                Ok(is != *negate)
            }
            Cond::Rare3(r) => self.cond_w2(r, ex, host),
            Cond::Rare4(r) => {
                let a = self.eval(&r.a, ex, host)?;
                let b = match &r.b {
                    Some(b) => Some(self.eval(b, ex, host)?),
                    None => None,
                };
                Ok(
                    crate::w4_conditions_1::check(r.kind, &a, b.as_ref(), ex.cancelled, host)
                        .is_some_and(|hit| hit != r.negate),
                )
            }
            Cond::Gap(g) => self.gap_cond(g, ex, host),
            Cond::W5c3(c) => self.cond_w5c3(c, ex, host),
            Cond::W6c3(c) => self.cond_w6c3(c, ex, host),
            Cond::Rtw(r) => {
                let a = self.eval(&r.a, ex, host)?;
                let b = match &r.b {
                    Some(b) => Some(self.eval(b, ex, host)?),
                    None => None,
                };
                Ok(crate::w4_skbee_raytrace_world::check(r, &a, b.as_ref(), host))
            }
            Cond::PlayerFlag { player, flag, negate } => {
                let p = self.eval(player, ex, host)?;
                let Some(p) = online(&p, &mut ex.seen, host) else {
                    return Ok(false);
                };
                Ok(host.player_flag(&p, *flag) != *negate)
            }
            Cond::Admin(kind, target, negate) => {
                let v = self.eval(target, ex, host)?;
                let (name, p) = self.admin_target(&v, ex, host);
                Ok(crate::op_kick_ban::check(host, *kind, &name, p.as_ref()) != *negate)
            }
            Cond::World(w) => self.world_cond(w, ex, host),
            Cond::Block(b) => self.block_cond(b, ex, host),
            Cond::Bound(b) => self.bound_cond(b, ex, host),
            Cond::Wt(w) => self.wt_cond(w, ex, host),
            Cond::Sb(c, negate) => Ok(self.sb_cond(c, ex, host)? != *negate),
            Cond::Nbt(n) => self.nbt_cond(n, ex, host),
            Cond::Ent(c) => self.ent_cond(c, ex, host),
            Cond::W5c2(c) => self.w5c2_cond(c, ex, host),
            Cond::W5c1(c) => self.w5c1_cond(c, ex, host),
            Cond::Skb(c) => self.skb_cond(c, ex, host),
            Cond::Server(c) => self.server_cond(c, ex, host),
            Cond::HasPerm { player, node, negate } => {
                let p = self.eval(player, ex, host)?;
                let node = self.text(node, ex, host)?;
                let Some(p) = online(&p, &mut ex.seen, host) else {
                    return Ok(false);
                };
                Ok(host.has_permission(&p, &node) != *negate)
            }
        }
    }

    pub(crate) fn text(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<String, String> {
        let mut s = String::new();
        self.write(e, ex, host, &mut s)?;
        Ok(s)
    }

    pub(crate) fn num(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<f64, String> {
        match e {
            Expr::Num(n) => return Ok(*n),
            Expr::Var(var) => {
                // Read in place: no clone of the stored value.
                let k = self.key(var, ex, host)?;
                let val = if var.local {
                    ex.locals.get(&k)
                } else {
                    self.vars.get_ref(&k)
                };
                return match val {
                    Some(Value::Num(n)) => Ok(*n),
                    other => num_of(other),
                };
            }
            _ => {}
        }
        let v = self.eval(e, ex, host)?;
        match v {
            Value::Num(n) => Ok(n),
            other => num_of(Some(&other)),
        }
    }

    pub(crate) fn player(
        &mut self,
        e: &Expr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Option<PlayerRef>, String> {
        if let Expr::Player = e {
            return Ok(ex.ctx.player.as_ref().and_then(|p| {
                resolve(&mut ex.seen, &p.uuid, host).map(|name| PlayerRef {
                    uuid: p.uuid.clone(),
                    name: name.to_string(),
                })
            }));
        }
        let v = self.eval(e, ex, host)?;
        Ok(online(&v, &mut ex.seen, host))
    }

    /// A name for an admin target (a player or a text), and the player when
    /// they are online.
    fn admin_target(&mut self, v: &Value, ex: &mut Exec, host: &mut dyn Host) -> (String, Option<PlayerRef>) {
        let p = online(v, &mut ex.seen, host);
        let name = match (&p, v) {
            (Some(p), _) => p.name.clone(),
            (None, Value::Player(p)) => p.name.clone(),
            (None, other) => other.to_string(),
        };
        (name, p)
    }

    pub(crate) fn key<'k>(
        &mut self,
        v: &'k VarRef,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Cow<'k, str>, String> {
        match &v.key {
            Key::Static(k) => Ok(Cow::Borrowed(k)),
            Key::Dynamic(parts) => {
                let mut s = String::new();
                self.write_parts(parts, ex, host, &mut s)?;
                s.make_ascii_lowercase();
                Ok(Cow::Owned(s))
            }
        }
    }

    fn write_parts(
        &mut self,
        parts: &[Part],
        ex: &mut Exec,
        host: &mut dyn Host,
        out: &mut String,
    ) -> Result<(), String> {
        for p in parts {
            match p {
                Part::Lit(l) => out.push_str(l),
                Part::Expr(e) => self.write(e, ex, host, out)?,
            }
        }
        Ok(())
    }

    /// Append the printed value of `e` to `out`. Same result as
    /// `eval(e).to_string()`, without the temporary values.
    fn write(
        &mut self,
        e: &Expr,
        ex: &mut Exec,
        host: &mut dyn Host,
        out: &mut String,
    ) -> Result<(), String> {
        match e {
            Expr::Num(n) => push_num(out, *n),
            Expr::Text(parts) => self.write_parts(parts, ex, host, out)?,
            Expr::Var(v) if v.list => {
                let items = self.list_items(v, ex, host)?;
                join_list(&items, out);
            }
            Expr::Var(v) => {
                let k = self.key(v, ex, host)?;
                let val = if v.local {
                    ex.locals.get(&k)
                } else {
                    self.vars.get_ref(&k)
                };
                match val {
                    Some(val) => val.push_to(out),
                    None => out.push_str("<none>"),
                }
            }
            Expr::Player => {
                let name = match &ex.ctx.player {
                    Some(p) => resolve(&mut ex.seen, &p.uuid, host),
                    None => None,
                };
                out.push_str(name.unwrap_or("<none>"));
            }
            Expr::Message => out.push_str(ex.ctx.message.as_deref().unwrap_or("<none>")),
            Expr::Arg(i) => match ex.ctx.args.get(i.wrapping_sub(1)) {
                Some(v) => v.push_to(out),
                None => out.push_str("<none>"),
            },
            _ => self.eval(e, ex, host)?.push_to(out),
        }
        Ok(())
    }

    pub(crate) fn get_var(
        &mut self,
        v: &VarRef,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        if v.list {
            return self.list_text(v, ex, host);
        }
        let k = self.key(v, ex, host)?;
        let val = if v.local {
            ex.locals.get(&k)
        } else {
            self.vars.get_ref(&k)
        };
        Ok(val.cloned().unwrap_or(Value::None))
    }

    /// `{x::*}` used as a value: `a, b and c`, or `<none>` when empty.
    #[inline(never)]
    fn list_text(&mut self, v: &VarRef, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        let items = self.list_items(v, ex, host)?;
        if items.is_empty() {
            return Ok(Value::None);
        }
        let mut s = String::new();
        join_list(&items, &mut s);
        Ok(Value::Text(s))
    }

    /// Evaluate list items: whole lists and `indices of` spread into their
    /// values, everything else gives one value.
    #[inline(never)]
    fn many(&mut self, es: &[Expr], ex: &mut Exec, host: &mut dyn Host) -> Result<Vec<Value>, String> {
        let mut out = Vec::with_capacity(es.len());
        for e in es {
            match e {
                Expr::Var(v) if v.list => {
                    out.extend(self.list_items(v, ex, host)?.into_iter().map(|(_, x)| x));
                }
                Expr::Indices(v) => {
                    out.extend(
                        self.list_items(v, ex, host)?
                            .into_iter()
                            .map(|(k, _)| Value::Text(k)),
                    );
                }
                Expr::ItemInfo(..) => match self.eval(e, ex, host)? {
                    Value::List(l) => out.extend(l),
                    v => out.push(v),
                },
                Expr::Ext(..) => match self.eval(e, ex, host)? {
                    Value::List(v) => out.extend(v),
                    v => out.push(v),
                },
                _ => out.push(self.eval(e, ex, host)?),
            }
        }
        Ok(out)
    }

    /// `set {x::*} to a, b`, `add a, b to {x::*}`, `remove a, b from {x::*}`.
    #[inline(never)]
    fn list_bulk(&mut self, e: &Effect, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        let (Effect::ListSet(v, items) | Effect::ListAddAll(v, items) | Effect::ListRemoveAll(v, items)) = e
        else {
            return Ok(());
        };
        // Evaluate everything first, so `set {x::*} to {x::*}, 1` sees the old list.
        let vals = self.many(items, ex, host)?;
        let key = self.key(v, ex, host)?.into_owned();
        let prefix = key.strip_suffix('*').unwrap_or(&key);
        let local = v.local;
        match e {
            Effect::ListSet(..) => {
                if local {
                    ex.locals.delete(&key);
                } else {
                    self.vars.delete(&key);
                }
            }
            Effect::ListRemoveAll(..) => {
                for x in &vals {
                    if local {
                        ex.locals.list_remove(prefix, x);
                    } else {
                        self.vars.list_remove(prefix, x);
                    }
                }
            }
            _ => {}
        }
        if !matches!(e, Effect::ListRemoveAll(..)) {
            for x in vals.into_iter().filter(|x| *x != Value::None) {
                if local {
                    ex.locals.list_push(prefix, x);
                } else {
                    self.vars.list_push(prefix, x);
                }
            }
        }
        if !local {
            self.mark_dirty(host);
        }
        Ok(())
    }

    /// `indices of {x::*}`, `first element of {x::*}`, `element N of {x::*}`.
    #[inline(never)]
    fn eval_list_fn(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        match e {
            Expr::Indices(v) => {
                let items = self.list_items(v, ex, host)?;
                if items.is_empty() {
                    return Ok(Value::None);
                }
                let keys: Vec<(String, Value)> = items
                    .into_iter()
                    .map(|(k, _)| (String::new(), Value::Text(k)))
                    .collect();
                let mut s = String::new();
                join_list(&keys, &mut s);
                Ok(Value::Text(s))
            }
            Expr::Elem(b) => {
                let (sel, v) = &**b;
                let n = match sel {
                    ElemSel::Nth(n) => Some(self.num(n, ex, host)?),
                    _ => None,
                };
                let items = self.list_items(v, ex, host)?;
                let i = match (sel, n) {
                    (ElemSel::First, _) => 0,
                    (ElemSel::Last, _) => items.len().wrapping_sub(1),
                    (_, Some(n)) if n >= 1.0 && n.fract() == 0.0 => n as usize - 1,
                    _ => return Ok(Value::None),
                };
                Ok(items.into_iter().nth(i).map(|(_, x)| x).unwrap_or(Value::None))
            }
            _ => Ok(Value::None),
        }
    }

    pub(crate) fn store(&mut self, local: bool, key: &str, val: Value, ex: &mut Exec, host: &mut dyn Host) {
        if local {
            ex.locals.set(key, val);
        } else {
            self.vars.set_str(key, val);
            self.mark_dirty(host);
        }
    }

    pub(crate) fn mark_dirty(&mut self, host: &mut dyn Host) {
        if self.dirty_since.is_none() {
            self.dirty_since = Some(host.now());
        }
    }

    /// Common expressions here; the rest in `eval_rare`. Keeping this
    /// function small keeps its stack frame small, which matters because it
    /// runs for nearly every value.
    pub(crate) fn eval(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        match e {
            Expr::Num(n) => Ok(Value::Num(*n)),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Var(v) => self.get_var(v, ex, host),
            Expr::Bin(a, op, b) => {
                let x = match self.num(a, ex, host) {
                    Ok(x) => x,
                    Err(err) => return self.bin_time(a, *op, b, err, ex, host),
                };
                let y = match self.num(b, ex, host) {
                    Ok(y) => y,
                    Err(err) => return self.bin_time(a, *op, b, err, ex, host),
                };
                Ok(Value::Num(match op {
                    BinOp::Add => x + y,
                    BinOp::Sub => x - y,
                    BinOp::Mul => x * y,
                    BinOp::Div => {
                        if y == 0.0 {
                            return Err("division by zero".into());
                        }
                        x / y
                    }
                }))
            }
            Expr::Text(parts) => {
                let mut s = String::new();
                self.write_parts(parts, ex, host, &mut s)?;
                Ok(Value::Text(s))
            }
            _ => self.eval_rare(e, ex, host),
        }
    }

    /// `{a} + {b}` where a variable turned out to hold a timespan or date.
    /// Any other failure keeps its original error.
    #[inline(never)]
    fn bin_time(
        &mut self,
        a: &Expr,
        op: BinOp,
        b: &Expr,
        err: String,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        let x = self.eval(a, ex, host).map_err(|_| err.clone())?;
        let y = self.eval(b, ex, host).map_err(|_| err.clone())?;
        if matches!(x, Value::Vector { .. }) || matches!(y, Value::Vector { .. }) {
            return crate::w2_expressions_7::arith(&x, op, &y);
        }
        let timey = |v: &Value| matches!(v, Value::Timespan { .. } | Value::Date(_));
        if !timey(&x) && !timey(&y) {
            return Err(err);
        }
        crate::timespans::arith(&x, op, &y)
    }

    #[inline(never)]
    fn eval_date(
        &mut self,
        op: crate::expr::DateOp,
        args: &[Expr],
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        use crate::expr::DateOp;
        if op == DateOp::Now {
            return Ok(Value::Date(host.wall_clock_millis()));
        }
        let first = match args.first() {
            Some(a) => self.eval(a, ex, host)?,
            None => Value::None,
        };
        if op == DateOp::FromUnix {
            return Ok(match to_num_opt(&first) {
                Some(n) if n.is_finite() && !matches!(first, Value::Date(_)) => {
                    Value::Date((n * 1000.0).round() as i64)
                }
                _ => Value::None,
            });
        }
        let Value::Date(d) = first else {
            if let (Value::Timespan { millis }, DateOp::Format, 1) = (&first, op, args.len()) {
                return Ok(Value::Text(crate::w2_expressions_3::clock_text(*millis)));
            }
            return Ok(Value::None);
        };
        Ok(match op {
            DateOp::ToUnix => Value::Num(d as f64 / 1000.0),
            _ => {
                let fmt = match args.get(1) {
                    Some(f) => self.text(f, ex, host)?,
                    None => crate::time_dates::DEFAULT_FORMAT.to_string(),
                };
                Value::Text(crate::time_dates::format_date(d, &fmt))
            }
        })
    }

    /// Evaluate the arguments of a `w2_expressions_4` expression or effect.
    /// A player that is offline becomes `<none>`.
    #[inline(never)]
    fn w4_args(&mut self, args: &[Expr], ex: &mut Exec, host: &mut dyn Host) -> Result<Vec<Value>, String> {
        let mut vals = Vec::with_capacity(args.len());
        for a in args {
            let v = self.eval(a, ex, host)?;
            vals.push(match v {
                Value::Player(_) => online(&v, &mut ex.seen, host).map_or(Value::None, Value::Player),
                v => v,
            });
        }
        Ok(vals)
    }

    #[inline(never)]
    fn eval_rare(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        Ok(match e {
            Expr::Num(n) => Value::Num(*n),
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Text(parts) => {
                let mut s = String::new();
                self.write_parts(parts, ex, host, &mut s)?;
                Value::Text(s)
            }
            Expr::Var(v) => self.get_var(v, ex, host)?,
            Expr::Player => match self.player(e, ex, host)? {
                Some(p) => Value::Player(p),
                None => Value::None,
            },
            Expr::Sender => match &ex.ctx.sender {
                Recipient::Console => Value::Console,
                Recipient::Player(p) => online(&Value::Player(p.clone()), &mut ex.seen, host)
                    .map(Value::Player)
                    .unwrap_or(Value::None),
            },
            Expr::Console => Value::Console,
            Expr::Message => ex.ctx.message.clone().map(Value::Text).unwrap_or(Value::None),
            Expr::Damage => ex.ctx.damage.map(Value::Num).unwrap_or(Value::None),
            Expr::Ev2Val(s) => crate::w2_events_2::read(&ex.ctx, *s),
            Expr::Attacker => match ex.ctx.attacker.clone() {
                Some(a) => online(&Value::Player(a), &mut ex.seen, host)
                    .map(Value::Player)
                    .unwrap_or(Value::None),
                None => crate::combat::attacker(&mut ex.ctx, host),
            },
            Expr::Inv(i) => self.inv_expr(i, ex, host)?,
            Expr::Gap(g) => self.gap_expr(g, ex, host)?,
            Expr::C4(n) => self.c4_expr(n, ex, host)?,
            Expr::ExtVal(i) => ex.ctx.ev.get(usize::from(*i)).cloned().unwrap_or(Value::None),
            Expr::ClickedBlock => match &ex.ctx.block {
                Some(b) => Value::Item {
                    id: b.clone(),
                    count: 1,
                    meta: None,
                },
                None => Value::None,
            },
            Expr::EventVal(v) => crate::events_items::eval(*v, &ex.ctx),
            Expr::Ev4Val(i) => ex.ctx.args.get(usize::from(*i)).cloned().unwrap_or(Value::None),
            Expr::Arg(i) => ex.ctx.args.get(i.wrapping_sub(1)).cloned().unwrap_or(Value::None),
            Expr::NameOf(inner) => {
                let v = self.eval(inner, ex, host)?;
                if matches!(v, Value::Item { .. }) {
                    return Ok(crate::item_meta::info(crate::item_meta::ItemInfo::Name, &v));
                }
                if let Value::Inventory(i) = &v {
                    return Ok(Value::Text(i.title.clone()));
                }
                if let Value::Entity(e) = &v {
                    // Like Skript: the custom name, else the type.
                    return Ok(Value::Text(
                        host.entity_name(e).unwrap_or_else(|| e.kind.replace('_', " ")),
                    ));
                }
                match online(&v, &mut ex.seen, host) {
                    Some(p) => Value::Text(p.name),
                    None => Value::None,
                }
            }
            Expr::Neg(inner) => Value::Num(-self.num(inner, ex, host)?),
            Expr::Call(id, args) => self.call_value(*id, args, ex, host)?,
            Expr::Item { id, count } => Value::Item {
                id: id.clone(),
                count: *count,
                meta: None,
            },
            Expr::ItemMeta(m) => {
                let base = self.eval(&m.base, ex, host)?;
                let name = match &m.name {
                    Some(n) => Some(self.text(n, ex, host)?),
                    None => None,
                };
                let mut lore = Vec::with_capacity(m.lore.len());
                for l in &m.lore {
                    lore.push(self.text(l, ex, host)?);
                }
                let mut specs = Vec::with_capacity(m.enchants.len());
                for s in &m.enchants {
                    specs.push(self.text(s, ex, host)?);
                }
                crate::item_meta::with_meta(base, name, lore, &specs)
            }
            Expr::ItemInfo(info, inner) => {
                let v = self.eval(inner, ex, host)?;
                crate::item_meta::info(*info, &v)
            }
            Expr::Enchant(b) => Value::Text(crate::item_meta::spec_text(&b.0, b.1)),
            Expr::WorldNamed(n) => Value::World(self.text(n, ex, host)?),
            Expr::Blk(b) => self.block_eval(b, ex, host)?,
            Expr::Bound(b) => self.bound_eval(b, ex, host)?,
            Expr::Server(q, w) => self.server_expr(*q, w, ex, host)?,
            Expr::Loc(x, y, z, w) => {
                let (x, y, z) = (
                    self.num(x, ex, host)?,
                    self.num(y, ex, host)?,
                    self.num(z, ex, host)?,
                );
                let world = match w {
                    Some(w) => match self.eval(w, ex, host)? {
                        Value::None => None,
                        v => Some(v.to_string()),
                    },
                    None => None,
                };
                Value::Location { x, y, z, world }
            }
            Expr::Distance(a, b) => {
                let a = self.eval(a, ex, host)?;
                let b = self.eval(b, ex, host)?;
                match (self.as_location(&a, ex, host), self.as_location(&b, ex, host)) {
                    (Some((ax, ay, az, aw)), Some((bx, by, bz, bw))) => {
                        if aw.is_some() && bw.is_some() && aw != bw {
                            return Err("distance between locations in different worlds".into());
                        }
                        Value::Num(((ax - bx).powi(2) + (ay - by).powi(2) + (az - bz).powi(2)).sqrt())
                    }
                    _ => Value::None,
                }
            }
            Expr::Nbt(n) => self.nbt_expr(n, ex, host)?,
            Expr::Date(op, args) => self.eval_date(*op, args, ex, host)?,
            Expr::Wt(w) => self.wt_eval(w, ex, host)?,
            Expr::Skb(s) => self.skb_eval(s, ex, host)?,
            Expr::X2(w) => self.eval_x2(w, ex, host)?,
            Expr::Color(on, e) => {
                let t = self.text(e, ex, host)?;
                Value::Text(if *on {
                    crate::colors::colorize(&t)
                } else {
                    crate::colors::strip(&t)
                })
            }
            Expr::ParseNum(e) => {
                let t = self.text(e, ex, host)?;
                match t.trim().parse::<f64>().ok().filter(|n| n.is_finite()) {
                    Some(n) => Value::Num(n),
                    None => {
                        self.w5.last_parse_error = Some(format!("'{t}' is not a number"));
                        Value::None
                    }
                }
            }
            Expr::Join(v, sep) => {
                let items = self.list_items(v, ex, host)?;
                let sep = self.text(sep, ex, host)?;
                Value::Text(crate::text_join_split::join_values(&items, &sep))
            }
            Expr::Split(..) => {
                return Err("split makes a list: use 'set {x::*} to split ...' or 'loop split ...'".into());
            }
            Expr::Length(e) => {
                if matches!(**e, Expr::Var(_) | Expr::W2(_) | Expr::W7(_) | Expr::Prop(..))
                    && let Value::Vector { x, y, z } = self.eval(e, ex, host)?
                {
                    return Ok(Value::Num(crate::w2_expressions_7::length((x, y, z))));
                }
                let t = self.text(e, ex, host)?;
                Value::Num(crate::text_join_split::char_len(&t) as f64)
            }
            Expr::Replaced(t, from, to, all) => {
                let t = self.text(t, ex, host)?;
                let from = self.text(from, ex, host)?;
                let to = self.text(to, ex, host)?;
                Value::Text(crate::text_join_split::replace_text(&t, &from, &to, *all))
            }
            Expr::Comp(f, args) => {
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval(a, ex, host)?);
                }
                crate::skbee_text::apply(*f, &vals)
            }
            Expr::W4(w) => {
                let vals = self.w4_args(&w.args, ex, host)?;
                crate::w2_expressions_4::apply(w.op, &vals, host)
            }
            Expr::TextFn(f, args) => {
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval(a, ex, host)?);
                }
                crate::text_case_substring::apply(*f, &vals)
            }
            Expr::Util(f, args) => {
                let mut vals = Vec::with_capacity(args.len().max(4));
                for a in args {
                    vals.push(self.eval(a, ex, host)?);
                }
                if *f == crate::skbee_utilities::UtilFn::RandomUuid {
                    for _ in 0..4 {
                        vals.push(Value::Num(self.next_random(host)));
                    }
                } else if f.needs_player() {
                    vals[0] = online(&vals[0], &mut ex.seen, host).map_or(Value::None, Value::Player);
                }
                crate::skbee_utilities::apply(*f, &vals, host)
            }
            Expr::Span(ms) => Value::Timespan { millis: *ms },
            Expr::Since(d, since) => {
                let Value::Date(millis) = self.eval(d, ex, host)? else {
                    return Ok(Value::None);
                };
                let now = host.wall_clock_millis();
                Value::Timespan {
                    millis: if *since { now - millis } else { millis - now },
                }
            }
            Expr::TBin(a, op, b) => {
                let x = self.eval(a, ex, host)?;
                let y = self.eval(b, ex, host)?;
                crate::timespans::arith(&x, *op, &y)?
            }
            Expr::SpanPart(unit, total, e) => match self.eval(e, ex, host)? {
                Value::Timespan { millis } => Value::Num(crate::timespans::span_part(millis, *unit, *total)),
                _ => Value::None,
            },
            Expr::W5c5(n) => {
                let mut vals = Vec::with_capacity(n.args.len());
                for a in &n.args {
                    vals.push(self.eval(a, ex, host)?);
                }
                crate::w5_chunk5::apply(n.op, &vals, host)?
            }
            Expr::W6c5(n) => {
                let mut vals = Vec::with_capacity(n.args.len());
                for a in &n.args {
                    vals.push(self.eval(a, ex, host)?);
                }
                crate::w6_chunk5::apply(n.op, &vals, host)?
            }
            Expr::W5c3(n) => self.eval_w5c3(n, ex, host)?,
            Expr::Rtw(n) => {
                let mut vals = Vec::with_capacity(n.args.len());
                for a in &n.args {
                    vals.push(self.eval(a, ex, host)?);
                }
                crate::w4_skbee_raytrace_world::apply(n.op, &vals, host)?
            }
            Expr::C6(n) => self.eval_c6(n, ex, host)?,
            Expr::W6C4(n) => self.w6c4_expr(n, ex, host)?,
            Expr::Z6(n) => self.eval_z6(n, ex, host)?,
            Expr::W7(n) => {
                let mut vals = Vec::with_capacity(n.args.len());
                for a in &n.args {
                    vals.push(self.eval(a, ex, host)?);
                }
                crate::w2_expressions_7::apply(n.op, &vals, host)?
            }
            Expr::T1(n) => {
                let mut vals = Vec::with_capacity(n.args.len());
                for a in &n.args {
                    vals.push(self.eval(a, ex, host)?);
                }
                crate::types_1::apply(&n.op, &vals)?
            }
            Expr::Items(s) => Value::List(self.eval_list(s, ex, host)?),
            Expr::Ext(f, args) => self.eval_ext(*f, args, ex, host)?,
            Expr::W5(w) => self.eval_w5(w, ex, host)?,
            Expr::Ph(p) => self.eval_ph(p, ex, host)?,
            Expr::W6(w) => self.eval_w6(w, ex, host)?,
            Expr::W4F(n) => self.eval_w4f(n, ex, host)?,
            Expr::ListAgg(a, s) => self.eval_agg(*a, s, ex, host)?,
            Expr::Sb(s) => self.sb_eval(s, ex, host)?,
            Expr::Ent(x) => self.ent_expr(x, ex, host)?,
            Expr::W2(w) => self.eval_w2(w, ex, host)?,
            Expr::RandomOf(v) => {
                let items = self.list_items(v, ex, host)?;
                if items.is_empty() {
                    Value::None
                } else {
                    let i = ((self.next_random(host) * items.len() as f64) as usize).min(items.len() - 1);
                    items.into_iter().nth(i).map(|(_, v)| v).unwrap_or(Value::None)
                }
            }
            Expr::Default(a, b) => match self.eval(a, ex, host)? {
                Value::None => self.eval(b, ex, host)?,
                v => v,
            },
            Expr::Math(f, args) => {
                let x = match args.first() {
                    Some(a) => self.num(a, ex, host)?,
                    None => 0.0,
                };
                let y = match args.get(1) {
                    Some(a) => self.num(a, ex, host)?,
                    None => 0.0,
                };
                Value::Num(match f {
                    MathFn::Floor => x.floor(),
                    MathFn::Ceil => x.ceil(),
                    MathFn::Round => x.round(),
                    MathFn::Abs => x.abs(),
                    MathFn::Sqrt => {
                        if x < 0.0 {
                            return Err(format!("sqrt of a negative number ({})", format_num(x)));
                        }
                        x.sqrt()
                    }
                    MathFn::Min => x.min(y),
                    MathFn::Max => x.max(y),
                    MathFn::Mod => {
                        if y == 0.0 {
                            return Err("mod by zero".into());
                        }
                        x.rem_euclid(y)
                    }
                })
            }
            Expr::Random(int, a, b) => {
                let (a, b) = (self.num(a, ex, host)?, self.num(b, ex, host)?);
                let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                let r = self.next_random(host);
                Value::Num(if *int {
                    let (lo, hi) = (lo.ceil(), hi.floor());
                    if hi < lo {
                        return Err("no whole number in that range".into());
                    }
                    (lo + (r * (hi - lo + 1.0)).floor()).min(hi)
                } else {
                    lo + r * (hi - lo)
                })
            }
            Expr::Prop(prop, of) => {
                let v = self.eval(of, ex, host)?;
                self.property(*prop, &v, ex, host)
            }
            Expr::LoopValue(d) => ex.loops.get(*d).map(LoopFrame::value).unwrap_or(Value::None),
            Expr::LoopIndex(d) => ex.loops.get(*d).map(LoopFrame::index).unwrap_or(Value::None),
            Expr::LoopIter(d) => ex
                .loops
                .get(*d)
                .map(|f| Value::Num(f.pos as f64))
                .unwrap_or(Value::None),
            Expr::Indices(_) | Expr::Elem(..) => return self.eval_list_fn(e, ex, host),
            Expr::Size(v) => {
                let key = self.key(v, ex, host)?;
                let prefix = key.strip_suffix('*').unwrap_or(&key);
                Value::Num(if v.local {
                    ex.locals.list_len(prefix)
                } else {
                    self.vars.list_len(prefix)
                } as f64)
            }
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
    fn effect(
        &mut self,
        e: &Effect,
        ex: &mut Exec,
        host: &mut dyn Host,
        line: usize,
    ) -> Result<Option<u64>, String> {
        if let Effect::Set(v, val) = e {
            let val = self.eval(val, ex, host)?;
            let k = self.key(v, ex, host)?;
            self.store(v.local, &k, val, ex, host);
            return Ok(None);
        }
        self.effect_rare(e, ex, host, line)
    }

    #[inline(never)]
    fn effect_rare(
        &mut self,
        e: &Effect,
        ex: &mut Exec,
        host: &mut dyn Host,
        line: usize,
    ) -> Result<Option<u64>, String> {
        match e {
            Effect::Send { text, to } => {
                let msg = self.text(text, ex, host)?;
                match to {
                    Some(to) => {
                        let p = if let Expr::Player = to {
                            self.player(to, ex, host)?
                        } else {
                            let v = self.eval(to, ex, host)?;
                            if let Value::Console = v {
                                if ex.ctx.sender == Recipient::Console {
                                    host.send_message(&Recipient::Console, &msg);
                                } else {
                                    host.send_console(&msg);
                                }
                                return Ok(None);
                            }
                            online(&v, &mut ex.seen, host)
                        };
                        if let Some(p) = p {
                            host.send_message(&Recipient::Player(p), &msg);
                        }
                    }
                    None => match &ex.ctx.sender {
                        Recipient::Console => host.send_message(&Recipient::Console, &msg),
                        Recipient::Player(p) => {
                            if let Some(name) = resolve(&mut ex.seen, &p.uuid, host) {
                                let to = PlayerRef {
                                    uuid: p.uuid.clone(),
                                    name: name.to_string(),
                                };
                                host.send_message(&Recipient::Player(to), &msg);
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
                if let Expr::Player = p {
                    if let Some(p) = self.player(p, ex, host)? {
                        host.set_health(&p, n.max(0.0));
                    }
                } else {
                    self.set_health_any(p, n, ex, host)?;
                }
            }
            Effect::Kill(p) => self.kill_any(p, ex, host)?,
            Effect::Teleport(p, x, y, z) => {
                let (x, y, z) = (
                    self.num(x, ex, host)?,
                    self.num(y, ex, host)?,
                    self.num(z, ex, host)?,
                );
                if let Some(p) = self.player(p, ex, host)? {
                    host.teleport(&p, x, y, z);
                }
            }
            Effect::TeleportTo(p, loc) => {
                let loc = self.eval(loc, ex, host)?;
                let Some((x, y, z, world)) = self.as_location(&loc, ex, host) else {
                    return Err(format!("expected a location, got {} ({loc})", loc.type_name()));
                };
                if let Some(p) = self.player(p, ex, host)? {
                    host.teleport_to(&p, x, y, z, world.as_deref());
                }
            }
            Effect::Give(p, n, item) => {
                let item = self.eval(item, ex, host)?;
                let Value::Item { id, count, meta } = item else {
                    return Err(format!("expected an item, got {} ({item})", item.type_name()));
                };
                let count = match n {
                    Some(n) => {
                        let n = self.num(n, ex, host)?;
                        if !n.is_finite() || n < 1.0 {
                            return Ok(None);
                        }
                        n.min(6400.0) as u32
                    }
                    None => count,
                };
                if let Some(p) = self.player(p, ex, host)? {
                    match meta {
                        Some(m) => host.give_item_meta(&p, &id, count, &m),
                        None => host.give_item(&p, &id, count),
                    }
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
                        &format!(
                            "{}:{}: 'cancel event' after a wait does nothing (the event already happened)",
                            ex.code.file, line
                        ),
                    );
                } else {
                    ex.cancelled = true;
                }
            }
            Effect::Wait(amount, 0) => {
                let v = self.eval(amount, ex, host)?;
                let Value::Timespan { millis } = v else {
                    return Err(format!(
                        "can't wait {} ({v}), use a timespan like '5 seconds'",
                        v.type_name()
                    ));
                };
                if millis < 0 {
                    return Err(format!("can't wait {v}"));
                }
                let ticks = crate::timespans::to_ticks(millis);
                if ticks > 0 {
                    return Ok(Some(ticks));
                }
            }
            Effect::Wait(amount, mult) => {
                let n = self.num(amount, ex, host)?;
                if !n.is_finite() || n < 0.0 {
                    return Err(format!("can't wait {}", format_num(n)));
                }
                let ticks = (n * *mult as f64).round() as u64;
                if ticks > 0 {
                    return Ok(Some(ticks));
                }
            }
            Effect::Set(v, val) => {
                let val = self.eval(val, ex, host)?;
                let k = self.key(v, ex, host)?;
                self.store(v.local, &k, val, ex, host);
            }
            Effect::Add(v, n) | Effect::Remove(v, n) => {
                let n = self.num(n, ex, host)?;
                let k = self.key(v, ex, host)?;
                let cur = if v.local {
                    ex.locals.get(&k)
                } else {
                    self.vars.get_ref(&k)
                };
                let base = match cur {
                    None | Some(Value::None) => 0.0,
                    Some(other) => to_num_opt(other).ok_or_else(|| {
                        format!(
                            "can't add to a variable holding {} ({})",
                            other.type_name(),
                            other
                        )
                    })?,
                };
                let res = if matches!(e, Effect::Add(..)) {
                    base + n
                } else {
                    base - n
                };
                self.store(v.local, &k, Value::Num(res), ex, host);
            }
            Effect::ListAdd(v, x) | Effect::ListRemove(v, x) => {
                let val = self.eval(x, ex, host)?;
                let key = self.key(v, ex, host)?.into_owned();
                let prefix = key.strip_suffix('*').unwrap_or(&key);
                let add = matches!(e, Effect::ListAdd(..));
                match (v.local, add) {
                    (true, true) => ex.locals.list_push(prefix, val),
                    (true, false) => ex.locals.list_remove(prefix, &val),
                    (false, true) => {
                        self.vars.list_push(prefix, val);
                        self.mark_dirty(host);
                    }
                    (false, false) => {
                        self.vars.list_remove(prefix, &val);
                        self.mark_dirty(host);
                    }
                }
            }
            Effect::Delete(v) => {
                let k = self.key(v, ex, host)?;
                if v.local {
                    ex.locals.delete(&k);
                } else {
                    self.vars.delete(&k);
                    self.mark_dirty(host);
                }
            }
            _ => return self.effect_ext(e, ex, host, line),
        }
        Ok(None)
    }

    /// Effects added after v1, kept apart so `effect_rare` keeps a small
    /// stack frame for the common ones.
    #[inline(never)]
    fn effect_ext(
        &mut self,
        e: &Effect,
        ex: &mut Exec,
        host: &mut dyn Host,
        _line: usize,
    ) -> Result<Option<u64>, String> {
        match e {
            Effect::W2b(w) => self.run_w2b(w, ex, host)?,
            Effect::W5c5(w) => {
                let mut vals = Vec::with_capacity(w.args.len());
                for a in &w.args {
                    vals.push(self.eval(a, ex, host)?);
                }
                crate::w5_chunk5::run_effect(w.op, &vals, host)?;
            }
            Effect::W5c3(w) => self.run_w5c3(w, ex, host)?,
            Effect::Rtw(w) => {
                let name = self.eval(&w.name, ex, host)?;
                crate::w4_skbee_raytrace_world::run_effect(w, &name, host)?;
            }
            Effect::Hf(h) => {
                let amount = match &h.amount {
                    Some(a) => self.num(a, ex, host)?,
                    None => 0.0,
                };
                if let Expr::Player = h.player {
                    if let Some(p) = self.player(&h.player, ex, host)? {
                        crate::health_food::run(h, amount, &p, host);
                    }
                } else {
                    self.hf_any(h, amount, ex, host)?;
                }
            }
            Effect::Admin(kind, target, text) => {
                let v = self.eval(target, ex, host)?;
                let text = match text {
                    Some(t) => Some(self.text(t, ex, host)?),
                    None => None,
                };
                let (name, p) = self.admin_target(&v, ex, host);
                crate::op_kick_ban::run_effect(host, *kind, &name, p.as_ref(), text.as_deref());
            }
            Effect::W2(w) => self.run_w2(w, ex, host)?,
            Effect::W2c(w) => {
                let v = self.eval(&w.value, ex, host)?;
                if let Some(p) = self.player(&w.player, ex, host)? {
                    crate::w2_expressions_3::run(w, &v, &p, host);
                }
            }
            Effect::W4(w) => {
                let vals = self.w4_args(&w.args, ex, host)?;
                crate::w2_expressions_4::run(w.op, &vals, host);
            }
            Effect::W6(w) => self.exec_w6(w, ex, host)?,
            Effect::SetGamemode(p, m) => {
                let m = self.text(m, ex, host)?;
                let Some(mode) = crate::gamemode::normalize(&m) else {
                    return Err(format!(
                        "'{m}' is not a game mode (survival, creative, adventure, spectator)"
                    ));
                };
                if let Some(p) = self.player(p, ex, host)? {
                    host.set_gamemode(&p, mode);
                }
            }
            Effect::Ext4(x) => match &**x {
                crate::w2_events_4::ExtEffect::SetVal(i, e) => {
                    let v = self.eval(e, ex, host)?;
                    let i = usize::from(*i);
                    if ex.ctx.args.len() <= i {
                        ex.ctx.args.resize(i + 1, Value::None);
                    }
                    ex.ctx.args[i] = v.clone();
                    self.ext_out.push((i as u8, v));
                }
            },
            Effect::W5(w) => self.run_w5_effect(w, ex, host)?,
            Effect::Xp(op, a, b) => {
                let (pe, ne) = if op.num_first { (b, a) } else { (a, b) };
                let n = self.num(ne, ex, host)?;
                if let Some(p) = self.player(pe, ex, host)? {
                    crate::xp_level::apply(host, &p, op.field, op.mode, n);
                }
            }
            Effect::X2(k, a) => self.run_x2(*k, a, ex, host)?,
            Effect::PlaySound(ps) => {
                let crate::compile::PlaySound {
                    sound,
                    volume,
                    pitch,
                    to,
                    at,
                } = &**ps;
                let id = crate::sounds::clean_id(&self.text(sound, ex, host)?)?;
                let volume = match volume {
                    Some(v) => self.num(v, ex, host)?,
                    None => 1.0,
                };
                let pitch = match pitch {
                    Some(v) => self.num(v, ex, host)?,
                    None => 1.0,
                };
                let at = match at {
                    Some(l) => {
                        let l = self.eval(l, ex, host)?;
                        let Some((x, y, z, _)) = self.as_location(&l, ex, host) else {
                            return Err(format!("expected a location, got {} ({l})", l.type_name()));
                        };
                        Some((x, y, z))
                    }
                    None => None,
                };
                match (to, at) {
                    // A location with no player: everyone online hears it.
                    (None, Some(_)) => {
                        for p in host.online_players() {
                            host.play_sound(&p, &id, volume, pitch, at);
                        }
                    }
                    (to, at) => {
                        let target = match to {
                            Some(t) => self.player(t, ex, host)?,
                            None => self.player(&Expr::Player, ex, host)?,
                        };
                        if let Some(p) = target {
                            host.play_sound(&p, &id, volume, pitch, at);
                        }
                    }
                }
            }
            Effect::StopSound(sound, from) => {
                let id = match sound {
                    Some(s) => Some(crate::sounds::clean_id(&self.text(s, ex, host)?)?),
                    None => None,
                };
                let target = match from {
                    Some(t) => self.player(t, ex, host)?,
                    None => self.player(&Expr::Player, ex, host)?,
                };
                if let Some(p) = target {
                    host.stop_sound(&p, id.as_deref());
                }
            }
            Effect::ExecAs(who, c) => {
                let who = self.eval(who, ex, host)?;
                let c = self.text(c, ex, host)?;
                let c = c.trim_start_matches('/');
                match who {
                    Value::Console => host.execute_command(c),
                    v => {
                        if let Some(p) = online(&v, &mut ex.seen, host) {
                            host.execute_command_as(&p, c);
                        }
                    }
                }
            }
            Effect::World(w) => self.world_effect(w, ex, host)?,
            Effect::Block(b) => self.block_effect(b, ex, host)?,
            Effect::Inv(i) => self.inv_effect(i, ex, host)?,
            Effect::Gap(g) => self.gap_effect(g, ex, host)?,
            Effect::C6(c) => self.exec_c6(c, ex, host)?,
            Effect::W6C4(c) => self.w6c4_effect(c, ex, host)?,
            Effect::Z6(c) => self.exec_z6(c, ex, host)?,
            Effect::C4(c) => self.c4_effect(c, ex, host)?,
            Effect::Bound(b) => self.bound_effect(b, ex, host)?,
            Effect::Wt(w) => self.wt_effect(w, ex, host)?,
            Effect::Sb(s) => self.sb_effect(s, ex, host)?,
            Effect::Sbt(s) => self.sbt_effect(s, ex, host)?,
            Effect::Nbt(n) => self.nbt_effect(n, ex, host)?,
            Effect::Ent(f) => self.ent_effect(f, ex, host)?,
            Effect::W5c1(f) => self.w5c1_effect(f, ex, host)?,
            Effect::W6c2(f) => self.w6c2_effect(f, ex, host)?,
            Effect::Combat(f) => self.combat_effect(f, ex, host)?,
            Effect::Skb(s) => self.skb_effect(s, ex, host)?,
            Effect::Server(f) => self.server_effect(f, ex, host)?,
            Effect::SetSplit(v, t, s) => {
                let t = self.text(t, ex, host)?;
                let s = self.text(s, ex, host)?;
                let key = self.key(v, ex, host)?.into_owned();
                let prefix = key.strip_suffix('*').unwrap_or(&key);
                if v.local {
                    ex.locals.delete(&key);
                    for x in crate::text_join_split::split_text(&t, &s) {
                        ex.locals.list_push(prefix, x);
                    }
                } else {
                    self.vars.delete(&key);
                    for x in crate::text_join_split::split_text(&t, &s) {
                        self.vars.list_push(prefix, x);
                    }
                    self.mark_dirty(host);
                }
            }
            Effect::Replace(v, from, to, all) => {
                let from = self.text(from, ex, host)?;
                let to = self.text(to, ex, host)?;
                let k = self.key(v, ex, host)?.into_owned();
                let cur = if v.local {
                    ex.locals.get(&k).cloned()
                } else {
                    self.vars.get_ref(&k).cloned()
                };
                if let Some(Value::Text(t)) = cur {
                    let new = crate::text_join_split::replace_text(&t, &from, &to, *all);
                    self.store(v.local, &k, Value::Text(new), ex, host);
                }
            }
            Effect::ListSet(..) | Effect::ListAddAll(..) | Effect::ListRemoveAll(..) => {
                self.list_bulk(e, ex, host)?;
            }
            Effect::SetList(v, s) => self.exec_set_list(v, s, ex, host)?,
            Effect::SortList(v, desc) => self.exec_sort_list(v, *desc, ex, host)?,
            Effect::W2e3(w) => self.exec_w2(w, ex, host)?,
            Effect::W2Transform(t) => self.exec_transform(t, ex, host)?,
            // The v1 effects are handled in `effect_rare`.
            _ => unreachable!(),
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

/// The non-number cases of `num`, out of line to keep the hot path small.
/// A date is not a number here, although `to_num_opt` accepts one.
#[inline(never)]
fn num_of(v: Option<&Value>) -> Result<f64, String> {
    match v {
        None => Err("expected a number, got <none> (<none>)".into()),
        Some(v @ Value::Date(_)) => Err(format!("expected a number, got {} ({})", v.type_name(), v)),
        Some(v) => to_num_opt(v).ok_or_else(|| format!("expected a number, got {} ({})", v.type_name(), v)),
    }
}

fn to_num_opt(v: &Value) -> Option<f64> {
    match v {
        Value::Num(n) => Some(*n),
        Value::Text(t) => t.trim().parse().ok(),
        Value::Date(d) => Some(*d as f64),
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
            ArgTy::Number => Value::Num(
                word.parse()
                    .map_err(|_| format!("'{word}' is not a number. Usage: {}", def.usage()))?,
            ),
            ArgTy::Player => Value::Player(
                host.find_player(word)
                    .ok_or_else(|| format!("Player '{word}' is not online"))?,
            ),
        });
    }
    if !rest.is_empty() {
        return Err(format!("Too many arguments. Usage: {}", def.usage()));
    }
    Ok(out)
}

/// A location from a value: a location itself, or an online player's position.
fn w2_loc(
    v: &Value,
    seen: &mut Vec<(String, Option<String>)>,
    host: &mut dyn Host,
) -> Option<(f64, f64, f64, Option<String>)> {
    if let Some((x, y, z, w)) = crate::w2_conditions_3::loc_parts(v) {
        return Some((x, y, z, w.map(str::to_string)));
    }
    let p = online(v, seen, host)?;
    host.player_location(&p).map(|(x, y, z, w)| (x, y, z, Some(w)))
}

impl Engine {
    /// The rare conditions of `w2_conditions_3`.
    #[inline(never)]
    fn cond_w2(
        &mut self,
        c: &crate::w2_conditions_3::W2Cond,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<bool, String> {
        use crate::w2_conditions_3 as w;
        use crate::w2_conditions_3::W2Cond as C;
        Ok(match c {
            C::Valid(e, negate) => {
                let v = self.eval(e, ex, host)?;
                let ok = match &v {
                    Value::None => false,
                    Value::Player(_) => online(&v, &mut ex.seen, host).is_some(),
                    _ => true,
                };
                ok != *negate
            }
            C::ScriptLoaded(e, negate) => {
                let name = self.text(e, ex, host)?;
                self.sources.keys().any(|f| w::script_name_matches(f, &name)) != *negate
            }
            C::Regex {
                text,
                pattern,
                whole,
                negate,
            } => {
                let t = self.text(text, ex, host)?;
                let p = self.text(pattern, ex, host)?;
                w::regex_test(&t, &p, *whole)? != *negate
            }
            C::WithinBox { loc, a, b, negate } => {
                let (l, a, b) = (
                    self.eval(loc, ex, host)?,
                    self.eval(a, ex, host)?,
                    self.eval(b, ex, host)?,
                );
                let (l, a, b) = (
                    w2_loc(&l, &mut ex.seen, host),
                    w2_loc(&a, &mut ex.seen, host),
                    w2_loc(&b, &mut ex.seen, host),
                );
                let ok = match (l, a, b) {
                    (Some(l), Some(a), Some(b)) => w::in_box(
                        (l.0, l.1, l.2, l.3.as_deref()),
                        (a.0, a.1, a.2, a.3.as_deref()),
                        (b.0, b.1, b.2, b.3.as_deref()),
                    ),
                    _ => return Ok(false),
                };
                ok != *negate
            }
            C::WithinRadius {
                loc,
                radius,
                center,
                negate,
            } => {
                let r = self.num(radius, ex, host)?;
                let (l, m) = (self.eval(loc, ex, host)?, self.eval(center, ex, host)?);
                let (l, m) = (w2_loc(&l, &mut ex.seen, host), w2_loc(&m, &mut ex.seen, host));
                let ok = match (l, m) {
                    (Some(l), Some(m)) => w::in_radius(
                        (l.0, l.1, l.2, l.3.as_deref()),
                        (m.0, m.1, m.2, m.3.as_deref()),
                        r,
                    ),
                    _ => return Ok(false),
                };
                ok != *negate
            }
            C::WithinWorld { loc, world, negate } => {
                let l = self.eval(loc, ex, host)?;
                let wv = self.eval(world, ex, host)?;
                let name = match &wv {
                    Value::World(n) | Value::Text(n) => n.clone(),
                    // `is within chunk ...` and `is within the border of ...`
                    Value::Chunk { .. } | Value::Border(_) => {
                        return Ok(crate::w6_chunk3::within_area(&l, &wv, &mut ex.seen, host) != *negate);
                    }
                    _ => return Ok(false),
                };
                let Some((_, _, _, Some(lw))) = w2_loc(&l, &mut ex.seen, host) else {
                    return Ok(false);
                };
                lw.eq_ignore_ascii_case(&name) != *negate
            }
            C::State(p, state, negate) => {
                let v = self.eval(p, ex, host)?;
                let Some(p) = online(&v, &mut ex.seen, host) else {
                    return Ok(false);
                };
                host.player_state(&p, *state) != *negate
            }
            C::Pvp { world, negate } => {
                let name = match world {
                    Some(e) => match self.eval(e, ex, host)? {
                        Value::World(n) | Value::Text(n) => Some(n),
                        _ => return Ok(false),
                    },
                    None => None,
                };
                match host.pvp_enabled(name.as_deref()) {
                    Some(on) => on != *negate,
                    None => false,
                }
            }
            C::Instant(e, negate) => {
                let t = self.text(e, ex, host)?;
                w::is_instant_effect(&t) != *negate
            }
            C::Block(e, tr, negate) => {
                let id = match self.eval(e, ex, host)? {
                    Value::Item { id, .. } => id,
                    Value::Text(t) => t,
                    // A placed block: its own state (stairs, slabs, open doors).
                    at @ Value::Location { .. } => {
                        return Ok(crate::w6_chunk3::placed_block_trait(&at, *tr, host) == Some(!*negate));
                    }
                    _ => return Ok(false),
                };
                match host.block_trait(&w::norm_id(&id), *tr) {
                    Some(b) => b != *negate,
                    None => false,
                }
            }
        })
    }
}

/// Placeholders (see `placeholders`).
impl Engine {
    #[inline(never)]
    fn eval_ph(
        &mut self,
        p: &crate::placeholders::Ph,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        use crate::placeholders::Ph;
        let (name_or_text, pl) = match p {
            Ph::Get(a, pl) | Ph::Parse(a, pl) => (a, pl),
        };
        let mut s = String::new();
        self.eval(name_or_text, ex, host)?.push_to(&mut s);
        let player = match pl {
            Some(e) => self.player(e, ex, host)?,
            None => self.player(&Expr::Player, ex, host)?,
        };
        Ok(match p {
            Ph::Get(..) => match self.placeholder(s.trim().trim_matches('%'), player.as_ref(), host) {
                Some(v) => Value::Text(v),
                None => Value::None,
            },
            Ph::Parse(..) => Value::Text(self.parse_placeholders(&s, player.as_ref(), host)),
        })
    }

    /// Value of one placeholder like `player_name` or `myplugin_thing` (no
    /// percent signs). `None` when nothing knows it.
    pub fn placeholder(
        &mut self,
        name: &str,
        player: Option<&PlayerRef>,
        host: &mut dyn Host,
    ) -> Option<String> {
        if let Some(v) = crate::placeholders::builtin(name, player, &self.vars, host) {
            return Some(v);
        }
        let (prefix, ident) = name.split_once('_').unwrap_or((name, ""));
        self.script_placeholder(&prefix.to_ascii_lowercase(), ident, player, host)
    }

    /// Replace the `%name%` placeholders in `text`. Unknown ones stay as written.
    pub fn parse_placeholders(
        &mut self,
        text: &str,
        player: Option<&PlayerRef>,
        host: &mut dyn Host,
    ) -> String {
        if text.len() > crate::placeholders::MAX_PARSE_LEN {
            return text.to_string();
        }
        crate::placeholders::replace_all(text, |n| self.placeholder(n, player, host))
    }

    /// Ask the `on placeholder request` triggers. The first one that sets
    /// `the result` wins.
    fn script_placeholder(
        &mut self,
        prefix: &str,
        ident: &str,
        player: Option<&PlayerRef>,
        host: &mut dyn Host,
    ) -> Option<String> {
        use crate::w2_events_4::ExtEvent;
        let kind = EventKind::Ext4(ExtEvent::PlaceholderRequest);
        if self.ph_depth >= crate::placeholders::MAX_DEPTH {
            return None;
        }
        let table = self.table.clone();
        let trigs = table.events.get(&kind)?;
        self.ph_depth += 1;
        let mut answer = None;
        for trig in trigs {
            let vals = vec![
                Value::Text(ident.to_string()),
                Value::None,
                Value::Text(prefix.to_string()),
            ];
            let ctx = crate::w2_events_4::ctx(player.cloned(), vals);
            let mut ex = Exec::new(trig, ctx);
            if self.budget == 0 {
                self.budget = self.max_instructions;
            }
            // A placeholder answers right away: a `wait` ends the run.
            let _ = self.run(&mut ex, host, 0);
            if let Some(v) = ex.ctx.args.get(1)
                && !matches!(v, Value::None)
            {
                let mut s = String::new();
                v.push_to(&mut s);
                answer = Some(s);
                break;
            }
        }
        self.ph_depth -= 1;
        answer
    }
}
