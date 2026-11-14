//! The `/carve` admin command: reload, enable/disable, list, info, errors,
//! variables, run and timings. File access goes through [`ScriptStore`], so the
//! whole command can be tested without a server.

use std::collections::BTreeSet;
use std::time::Instant;

use crate::engine::{Engine, LoadSummary};
use crate::error::ScriptError;
use crate::host::Host;
use crate::value::{PlayerRef, Recipient, Value};

pub const PREFIX: &str = "&8[&6Carve&8]&r ";
const SCRIPT_EXT: &str = ".carve";
/// Disabled scripts start with this (Skript's convention).
const DISABLED: char = '-';

/// The scripts folder.
pub trait ScriptStore {
    /// File names in the folder (`x.carve`, and `-x.carve` when disabled).
    fn list(&mut self) -> Result<Vec<String>, String>;
    fn read(&mut self, name: &str) -> Result<String, String>;
    fn rename(&mut self, from: &str, to: &str) -> Result<(), String>;
}

/// Who runs the command and what the plugin knows about itself.
pub struct AdminEnv<'a> {
    pub sender: Recipient,
    /// Does the sender have this permission node (`carve:reload`)?
    pub allowed: &'a dyn Fn(&str) -> bool,
    /// Script commands registered with the server at load.
    pub registered: &'a BTreeSet<String>,
    pub version: &'a str,
    pub api: &'a str,
    pub data_folder: &'a str,
}

#[derive(Debug, Default)]
pub struct AdminOut {
    /// Lines to send, with `&` color codes.
    pub lines: Vec<String>,
    /// All scripts were (re)loaded: the caller fires `on load`.
    pub loaded: bool,
}

impl AdminOut {
    fn say(&mut self, s: impl Into<String>) {
        self.lines.push(format!("{PREFIX}{}", s.into()));
    }
    fn raw(&mut self, s: impl Into<String>) {
        self.lines.push(s.into());
    }
}

/// (name, usage, description, permission)
pub const SUBCOMMANDS: &[(&str, &str, &str, &str)] = &[
    ("help", "help", "this list", ""),
    (
        "reload",
        "reload [all|scripts|<script>]",
        "reload every script or one file",
        "carve:reload",
    ),
    (
        "enable",
        "enable <script|all>",
        "enable a disabled script",
        "carve:enable",
    ),
    (
        "disable",
        "disable <script|all>",
        "disable a script (renames it to -name)",
        "carve:enable",
    ),
    (
        "list",
        "list [enabled|disabled]",
        "scripts with trigger and error counts",
        "carve:admin",
    ),
    (
        "info",
        "info [<script>]",
        "versions and counts, or what a script contains",
        "carve:admin",
    ),
    (
        "errors",
        "errors [<script>]",
        "errors of the last load",
        "carve:admin",
    ),
    (
        "vars",
        "vars list [<prefix>] | get|set|delete <name> [<value>]",
        "look at and edit variables",
        "carve:vars",
    ),
    ("run", "run <effect>", "run one line of Carve now", "carve:run"),
    (
        "timings",
        "timings [on|off|reset]",
        "how long triggers take",
        "carve:admin",
    ),
];

/// Permission nodes the plugin registers for `/carve`.
pub const PERMISSIONS: &[&str] = &[
    "carve:admin",
    "carve:reload",
    "carve:enable",
    "carve:vars",
    "carve:run",
];

fn can(env: &AdminEnv, node: &str) -> bool {
    node.is_empty()
        || matches!(env.sender, Recipient::Console)
        || (env.allowed)("carve:admin")
        || (env.allowed)(node)
}

/// The file in `names` that `arg` means: with or without `.carve`, any case,
/// `/` or `\` for folders.
fn find_script(arg: &str, names: &[String]) -> Option<String> {
    let mut want = arg.trim().trim_start_matches(DISABLED).replace('\\', "/").to_ascii_lowercase();
    if !want.ends_with(SCRIPT_EXT) {
        want.push_str(SCRIPT_EXT);
    }
    names
        .iter()
        .find(|n| n.replace('\\', "/").to_ascii_lowercase() == want)
        .cloned()
}

/// Skript's message for a script that isn't there.
fn not_found(arg: &str) -> String {
    format!("Can't find the script &f'{}'&c.", arg.trim())
}

fn scripts(store: &mut dyn ScriptStore) -> Result<(Vec<String>, Vec<String>), String> {
    let mut on = Vec::new();
    let mut off = Vec::new();
    for f in store.list()? {
        if !f.ends_with(SCRIPT_EXT) {
            continue;
        }
        match f.strip_prefix(DISABLED) {
            Some(rest) => off.push(rest.to_string()),
            None => on.push(f),
        }
    }
    on.sort();
    off.sort();
    Ok((on, off))
}

fn read_enabled(store: &mut dyn ScriptStore) -> Result<Vec<(String, String)>, String> {
    let (on, _) = scripts(store)?;
    let mut files = Vec::new();
    for f in on {
        let src = store.read(&f)?;
        files.push((f, src));
    }
    Ok(files)
}

/// `(12ms)`, like Skript.
fn ms(t: Instant) -> String {
    format!("({}ms)", t.elapsed().as_millis())
}

fn plural(n: usize, one: &str) -> String {
    if n == 1 { format!("{n} {one}") } else { format!("{n} {one}s") }
}

/// Errors in Skript's layout; at most 10 in chat, the rest are in `/carve errors`.
fn show_errors(out: &mut AdminOut, errors: &[ScriptError]) {
    for e in errors.iter().take(10) {
        let mut lines = e.skript_lines().into_iter();
        if let Some(head) = lines.next() {
            out.say(head);
        }
        for l in lines {
            out.raw(l);
        }
    }
    if errors.len() > 10 {
        out.say(format!(
            "&c... and {} more (see /carve errors or the server log)",
            errors.len() - 10
        ));
    }
}

/// Errors of `files`, plus the ones elsewhere that are new since `before`
/// (another script calling a function that is gone, for example).
fn errors_for(engine: &Engine, files: &[String], before: &[ScriptError]) -> Vec<ScriptError> {
    engine
        .errors
        .iter()
        .filter(|e| files.contains(&e.file) || !before.contains(e))
        .cloned()
        .collect()
}

/// Skript's closing line: `Successfully reloaded x.` or `Encountered 2 errors while reloading x!`.
fn finish(out: &mut AdminOut, errors: &[ScriptError], done: &str, doing: &str, what: &str, t0: Instant) {
    show_errors(out, errors);
    if errors.is_empty() {
        out.say(format!("&aSuccessfully {done} &f{what}&a. &7{}", ms(t0)));
    } else {
        out.say(format!(
            "&cEncountered &f{} &c{} while {doing} &f{what}&c! &7{}",
            errors.len(),
            if errors.len() == 1 { "error" } else { "errors" },
            ms(t0)
        ));
    }
}

/// Console lines for the load at server start, like Skript's:
/// each error, then `Loaded 12 scripts with a total of 56 structures in 0.04 seconds`.
/// `true` marks error lines.
pub fn load_report(sum: &LoadSummary, errors: &[ScriptError], secs: f64) -> Vec<(bool, String)> {
    let mut out = Vec::new();
    for e in errors {
        for l in e.plain_lines() {
            out.push((true, l));
        }
    }
    if !errors.is_empty() {
        out.push((
            true,
            format!("Encountered {} while loading the scripts!", plural(errors.len(), "error")),
        ));
    }
    out.push((
        false,
        format!(
            "Loaded {} with a total of {} in {secs:.2} seconds",
            plural(sum.files, "script"),
            plural(sum.triggers + sum.functions, "structure")
        ),
    ));
    out
}

fn restart_note(out: &mut AdminOut, engine: &Engine, env: &AdminEnv) {
    let new: Vec<String> = engine
        .table()
        .command_names()
        .into_iter()
        .filter(|c| !env.registered.contains(c))
        .map(|c| format!("/{c}"))
        .collect();
    if !new.is_empty() {
        out.say(format!(
            "&eNew commands need a server restart: {}",
            new.join(", ")
        ));
    }
}

/// Run `/carve <args>`.
pub fn admin(
    engine: &mut Engine,
    store: &mut dyn ScriptStore,
    host: &mut dyn Host,
    env: &AdminEnv,
    args: &str,
) -> AdminOut {
    let mut out = AdminOut::default();
    let args = args.trim();
    let (sub, rest) = args.split_once(char::is_whitespace).unwrap_or((args, ""));
    let sub = sub.to_ascii_lowercase();
    let rest = rest.trim();
    let sub = if sub.is_empty() { "help".to_string() } else { sub };
    let Some(&(_, usage, _, perm)) = SUBCOMMANDS.iter().find(|(n, ..)| *n == sub) else {
        out.say(format!("&cUnknown subcommand '{sub}'. Try /carve help"));
        return out;
    };
    if !can(env, perm) {
        out.say(format!("&cYou need the {perm} permission for /carve {sub}."));
        return out;
    }
    let r = match sub.as_str() {
        "help" => {
            out.say(format!("&6Carve {} &7commands:", env.version));
            for (_, u, d, p) in SUBCOMMANDS {
                if can(env, p) {
                    out.raw(format!("&e/carve {u} &7- {d}"));
                }
            }
            Ok(())
        }
        "reload" => reload(engine, store, env, rest, &mut out),
        "enable" | "disable" => toggle(engine, store, env, rest, sub == "enable", &mut out),
        "list" => list(engine, store, rest, &mut out),
        "info" => info(engine, store, env, rest, &mut out),
        "errors" => errors(engine, store, rest, &mut out),
        "vars" => vars(engine, host, rest, usage, &mut out),
        "run" => {
            if rest.is_empty() {
                Err(format!("Usage: /carve {usage}"))
            } else {
                match engine.run_line(rest, env.sender.clone(), host) {
                    Ok(()) => Ok(()),
                    Err(errs) => {
                        show_errors(&mut out, &errs);
                        Ok(())
                    }
                }
            }
        }
        "timings" => timings(engine, rest, &mut out),
        _ => Err(format!("Usage: /carve {usage}")),
    };
    if let Err(e) = r {
        out.say(format!("&c{e}"));
    }
    out
}

fn errors(engine: &Engine, store: &mut dyn ScriptStore, rest: &str, out: &mut AdminOut) -> Result<(), String> {
    let want = if rest.is_empty() {
        None
    } else {
        let (on, _) = scripts(store)?;
        let loaded: Vec<String> = engine.errors.iter().map(|e| e.file.clone()).collect();
        let all: Vec<String> = on.into_iter().chain(loaded).collect();
        Some(find_script(rest, &all).ok_or_else(|| not_found(rest))?)
    };
    let errs: Vec<ScriptError> = engine
        .errors
        .iter()
        .filter(|e| want.as_ref().is_none_or(|w| e.file == *w))
        .cloned()
        .collect();
    let what = want.as_deref().unwrap_or("the scripts");
    if errs.is_empty() {
        out.say(format!("&aNo errors in &f{what}&a."));
    } else {
        show_errors(out, &errs);
        out.say(format!(
            "&f{} &cin &f{what}&c (last load).",
            plural(errs.len(), "error")
        ));
    }
    Ok(())
}

fn reload(
    engine: &mut Engine,
    store: &mut dyn ScriptStore,
    env: &AdminEnv,
    rest: &str,
    out: &mut AdminOut,
) -> Result<(), String> {
    let t0 = Instant::now();
    if matches!(rest.to_ascii_lowercase().as_str(), "" | "all" | "scripts") {
        out.say("&7Reloading all scripts...");
        let files = read_enabled(store)?;
        let (_, errs) = engine.reload(&files);
        finish(out, &errs, "reloaded", "reloading", "all scripts", t0);
        restart_note(out, engine, env);
        out.loaded = true;
        return Ok(());
    }
    let (on, off) = scripts(store)?;
    let Some(name) = find_script(rest, &on) else {
        if let Some(name) = find_script(rest, &off) {
            return Err(format!(
                "&f{name} &cis disabled. Use &f/carve enable {}&c.",
                name.trim_end_matches(SCRIPT_EXT)
            ));
        }
        return Err(not_found(rest));
    };
    out.say(format!("&7Reloading &f{name}&7..."));
    let src = store.read(&name)?;
    let before = engine.errors.clone();
    engine.reload_file(&name, Some(src));
    let errs = errors_for(engine, std::slice::from_ref(&name), &before);
    finish(out, &errs, "reloaded", "reloading", &name, t0);
    restart_note(out, engine, env);
    Ok(())
}

fn toggle(
    engine: &mut Engine,
    store: &mut dyn ScriptStore,
    env: &AdminEnv,
    rest: &str,
    enable: bool,
    out: &mut AdminOut,
) -> Result<(), String> {
    let t0 = Instant::now();
    if rest.is_empty() {
        return Err(format!(
            "Usage: /carve {} <script|all>",
            if enable { "enable" } else { "disable" }
        ));
    }
    let (on, off) = scripts(store)?;
    let all = rest.eq_ignore_ascii_case("all");
    let names: Vec<String> = if all {
        if enable { off.clone() } else { on.clone() }
    } else {
        let (want, other) = if enable { (&off, &on) } else { (&on, &off) };
        match find_script(rest, want) {
            Some(n) => vec![n],
            None => {
                return Err(match find_script(rest, other) {
                    Some(n) => format!(
                        "&f{n} &cis already {}.",
                        if enable { "enabled" } else { "disabled" }
                    ),
                    None => not_found(rest),
                });
            }
        }
    };
    if names.is_empty() {
        out.say(format!(
            "&7Nothing to {}.",
            if enable { "enable" } else { "disable" }
        ));
        return Ok(());
    }
    let disabled = |n: &str| format!("{DISABLED}{n}");
    for name in &names {
        if enable {
            store.rename(&disabled(name), name)?;
        } else {
            store.rename(name, &disabled(name))?;
        }
    }
    let what = if all {
        "all scripts".to_string()
    } else {
        names[0].clone()
    };
    if enable {
        out.say(format!("&7Enabling &f{what}&7..."));
    } else {
        out.say(format!("&7Disabling &f{what}&7..."));
    }
    // Apply right away.
    let before = engine.errors.clone();
    if all {
        let files = read_enabled(store)?;
        engine.reload(&files);
    } else if enable {
        let src = store.read(&names[0])?;
        engine.reload_file(&names[0], Some(src));
    } else {
        engine.reload_file(&names[0], None);
    }
    let errs = if enable {
        errors_for(engine, &names, &before)
    } else {
        // Errors that disabling caused elsewhere (calls of its functions, say).
        errors_for(engine, &[], &before)
    };
    if enable {
        finish(out, &errs, "enabled and loaded", "enabling", &what, t0);
        restart_note(out, engine, env);
    } else {
        finish(out, &errs, "disabled", "disabling", &what, t0);
    }
    out.loaded = all;
    Ok(())
}

fn list(engine: &Engine, store: &mut dyn ScriptStore, rest: &str, out: &mut AdminOut) -> Result<(), String> {
    let (on, off) = scripts(store)?;
    let show_on = rest != "disabled";
    let show_off = rest != "enabled";
    if show_on {
        out.say(format!("&aEnabled ({}):", on.len()));
        for f in &on {
            let triggers = engine
                .table()
                .file_info
                .iter()
                .find(|i| i.name == *f)
                .map(|i| i.triggers.len());
            let errors = engine.errors.iter().filter(|e| e.file == *f).count();
            let state = match (triggers, errors) {
                (Some(t), 0) => format!("&7{t} trigger(s)"),
                (Some(t), e) => format!("&7{t} trigger(s), &c{e} error(s) in the last load"),
                (None, e) => format!("&cnot loaded, {e} error(s)"),
            };
            out.raw(format!("&f  {f} {state}"));
        }
    }
    if show_off {
        out.say(format!("&7Disabled ({}):", off.len()));
        for f in &off {
            out.raw(format!("&8  {f}"));
        }
    }
    Ok(())
}

/// `name: value` lines of a file's `options:` blocks.
fn options_of(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indented = line.starts_with(' ') || line.starts_with('\t');
        if !indented {
            inside = trimmed.eq_ignore_ascii_case("options:");
        } else if inside {
            out.push(trimmed.to_string());
        }
    }
    out
}

fn info(
    engine: &Engine,
    store: &mut dyn ScriptStore,
    env: &AdminEnv,
    rest: &str,
    out: &mut AdminOut,
) -> Result<(), String> {
    let t = engine.table();
    if rest.is_empty() {
        out.say(format!(
            "&6Carve {} &7(Pumpkin plugin API {})",
            env.version, env.api
        ));
        out.raw(format!(
            "&7Scripts: &f{}&7, triggers: &f{}&7, commands: &f{}&7, functions: &f{}",
            t.files,
            t.triggers,
            t.command_names().len(),
            t.function_count()
        ));
        out.raw(format!(
            "&7Variables: &f{}&7, waiting runs: &f{}&7, last load errors: &f{}",
            engine.vars.len(),
            engine.pending_waits(),
            engine.errors.len()
        ));
        out.raw(format!("&7Data folder: &f{}", env.data_folder));
        return Ok(());
    }
    let loaded: Vec<String> = t.file_info.iter().map(|f| f.name.clone()).collect();
    let Some(name) = find_script(rest, &loaded) else {
        return Err(not_found(rest));
    };
    let Some(fi) = t.file_info.iter().find(|f| f.name == name) else {
        return Err(not_found(rest));
    };
    out.say(format!("&6{name}"));
    out.raw(format!(
        "&7Triggers ({}): &f{}",
        fi.triggers.len(),
        fi.triggers.join(", ")
    ));
    if !fi.commands.is_empty() {
        out.raw(format!("&7Commands: &f{}", fi.commands.join(", ")));
    }
    if !fi.functions.is_empty() {
        out.raw(format!("&7Functions: &f{}", fi.functions.join(", ")));
    }
    if let Ok(src) = store.read(&name) {
        let opts = options_of(&src);
        if !opts.is_empty() {
            out.raw(format!("&7Options: &f{}", opts.join(", ")));
        }
    }
    Ok(())
}

/// `{name}` or `name` -> `name`, lowercased like script variables.
fn var_name(s: &str) -> String {
    s.trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .to_ascii_lowercase()
}

/// `5` -> number, `true`/`false`, `"text"` or bare text.
fn parse_value(s: &str) -> Value {
    let s = s.trim();
    if let Ok(n) = s.parse::<f64>()
        && n.is_finite()
    {
        return Value::Num(n);
    }
    match s.to_ascii_lowercase().as_str() {
        "true" | "yes" => return Value::Bool(true),
        "false" | "no" => return Value::Bool(false),
        _ => {}
    }
    let t = s.strip_prefix('"').and_then(|t| t.strip_suffix('"')).unwrap_or(s);
    Value::Text(t.to_string())
}

fn show_value(v: &Value) -> String {
    match v {
        Value::Text(t) => format!("\"{t}\""),
        Value::Player(PlayerRef { name, uuid }) => format!("player {name} ({uuid})"),
        other => format!("{other} &8({})", other.type_name()),
    }
}

fn vars(
    engine: &mut Engine,
    host: &mut dyn Host,
    rest: &str,
    usage: &str,
    out: &mut AdminOut,
) -> Result<(), String> {
    let (op, arg) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    let arg = arg.trim();
    match op.to_ascii_lowercase().as_str() {
        "list" | "" => {
            let prefix = var_name(arg);
            let prefix = prefix.trim_end_matches('*');
            let keys = engine.vars.keys_with_prefix(prefix);
            out.say(format!(
                "&7{} variable(s){}:",
                keys.len(),
                if prefix.is_empty() {
                    String::new()
                } else {
                    format!(" starting with '{prefix}'")
                }
            ));
            for k in keys.iter().take(20) {
                out.raw(format!("&f  {{{k}}} &7= &f{}", show_value(&engine.vars.get(k))));
            }
            if keys.len() > 20 {
                out.raw(format!(
                    "&7  ... and {} more; narrow it down with a prefix",
                    keys.len() - 20
                ));
            }
        }
        "get" => {
            let k = var_name(arg);
            if k.is_empty() {
                return Err(format!("Usage: /carve {usage}"));
            }
            out.say(format!("&f{{{k}}} &7= &f{}", show_value(&engine.vars.get(&k))));
        }
        "set" => {
            let (k, v) = arg
                .split_once(char::is_whitespace)
                .ok_or_else(|| format!("Usage: /carve {usage}"))?;
            let k = var_name(k);
            if k.starts_with('_') || k.ends_with("::*") {
                return Err("Only single global variables can be set here.".into());
            }
            let v = parse_value(v);
            out.say(format!("&aSet {{{k}}} to {}.", show_value(&v)));
            engine.vars.set(k, v);
            engine.vars_changed(host);
        }
        "delete" | "remove" => {
            let k = var_name(arg);
            if k.is_empty() {
                return Err(format!("Usage: /carve {usage}"));
            }
            let before = engine.vars.len();
            engine.vars.delete(&k);
            engine.vars_changed(host);
            out.say(format!("&aDeleted {} variable(s).", before - engine.vars.len()));
        }
        _ => return Err(format!("Usage: /carve {usage}")),
    }
    Ok(())
}

fn timings(engine: &mut Engine, rest: &str, out: &mut AdminOut) -> Result<(), String> {
    match rest {
        "on" => {
            engine.timings.get_or_insert_with(Default::default);
            out.say("&aTimings on. Use /carve timings to see them.");
        }
        "off" => {
            engine.timings = None;
            out.say("&7Timings off (and cleared).");
        }
        "reset" => {
            if let Some(t) = engine.timings.as_mut() {
                t.clear();
            }
            out.say("&7Timings cleared.");
        }
        "" => {
            let Some(t) = engine.timings.as_ref() else {
                out.say("&7Timings are off. Turn them on with /carve timings on");
                return Ok(());
            };
            let mut rows: Vec<_> = t.iter().collect();
            rows.sort_by_key(|r| std::cmp::Reverse(r.1.total));
            out.say(format!("&6Top triggers by total time ({} tracked):", rows.len()));
            for (label, tm) in rows.iter().take(10) {
                out.raw(format!(
                    "&f  {label} &7{} run(s), total &f{:.2} ms&7, max &f{:.3} ms",
                    tm.calls,
                    tm.total.as_secs_f64() * 1000.0,
                    tm.max.as_secs_f64() * 1000.0
                ));
            }
        }
        _ => return Err("Usage: /carve timings [on|off|reset]".into()),
    }
    Ok(())
}

/// Tab completion for `/carve <args>`: the start of the word to replace (as a
/// byte offset into `args`) and the choices.
pub fn suggest(store: &mut dyn ScriptStore, args: &str) -> (usize, Vec<String>) {
    let words: Vec<&str> = args.split_whitespace().collect();
    let ends_space = args.ends_with(char::is_whitespace) || args.is_empty();
    let (index, current) = if ends_space {
        (words.len(), "")
    } else {
        (words.len().saturating_sub(1), *words.last().unwrap_or(&""))
    };
    let start = args.len() - current.len();
    let cur = current.to_ascii_lowercase();
    let options: Vec<String> = match index {
        0 => SUBCOMMANDS.iter().map(|(n, ..)| n.to_string()).collect(),
        1 => {
            let (on, off) = scripts(store).unwrap_or_default();
            let strip = |v: Vec<String>| {
                v.into_iter()
                    .map(|f| f.trim_end_matches(SCRIPT_EXT).to_string())
                    .collect::<Vec<_>>()
            };
            match words.first().map(|w| w.to_ascii_lowercase()).as_deref() {
                Some("reload") => ["all".to_string()].into_iter().chain(strip(on)).collect(),
                Some("disable") => ["all".to_string()].into_iter().chain(strip(on)).collect(),
                Some("enable") => ["all".to_string()].into_iter().chain(strip(off)).collect(),
                Some("info") | Some("errors") => strip(on),
                Some("list") => vec!["enabled".into(), "disabled".into()],
                Some("vars") => vec!["list".into(), "get".into(), "set".into(), "delete".into()],
                Some("timings") => vec!["on".into(), "off".into(), "reset".into()],
                _ => Vec::new(),
            }
        }
        _ => Vec::new(),
    };
    (
        start,
        options
            .into_iter()
            .filter(|o| o.to_ascii_lowercase().starts_with(&cur))
            .collect(),
    )
}
