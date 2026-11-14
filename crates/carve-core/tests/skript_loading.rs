//! Skript-style loading: broken lines, sections and triggers are reported and
//! left out, everything else loads; reload replaces the old version.

use std::collections::{BTreeMap, BTreeSet};

use carve_core::admin::{AdminEnv, ScriptStore, admin, load_report};
use carve_core::*;

#[derive(Default)]
struct Store {
    files: BTreeMap<String, String>,
}

impl ScriptStore for Store {
    fn list(&mut self) -> Result<Vec<String>, String> {
        Ok(self.files.keys().cloned().collect())
    }
    fn read(&mut self, name: &str) -> Result<String, String> {
        self.files
            .get(name)
            .cloned()
            .ok_or_else(|| format!("no file {name}"))
    }
    fn rename(&mut self, from: &str, to: &str) -> Result<(), String> {
        let v = self.files.remove(from).ok_or_else(|| format!("no file {from}"))?;
        self.files.insert(to.to_string(), v);
        Ok(())
    }
}

#[derive(Default)]
struct H {
    calls: Vec<String>,
}

impl Host for H {
    fn send_message(&mut self, _: &Recipient, text: &str) {
        self.calls.push(format!("send {text}"));
    }
    fn broadcast(&mut self, text: &str) {
        self.calls.push(text.to_string());
    }
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, _: &str) {}
    fn resolve_player(&mut self, _: &str) -> Option<String> {
        None
    }
    fn find_player(&mut self, _: &str) -> Option<PlayerRef> {
        None
    }
    fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, _: LogLevel, _: &str) {}
}

fn load(src: &str) -> (Engine, Vec<ScriptError>) {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    (e, errs)
}

fn join(e: &mut Engine) -> Vec<String> {
    let mut h = H::default();
    e.fire(EventKind::Join, EventCtx::console(), &mut h);
    h.calls
}

#[test]
fn good_trigger_runs_when_another_line_fails() {
    let (mut e, errs) = load("on join:\n    blorp\non join:\n    broadcast \"ok\"\n");
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert_eq!(errs[0].line, 2);
    assert_eq!(join(&mut e), ["ok"]);
}

#[test]
fn bad_statement_is_skipped_rest_of_trigger_runs() {
    let (mut e, errs) = load("on join:\n    broadcast \"a\"\n    blorp blorp\n    broadcast \"b\"\n");
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert_eq!(errs[0].code, "blorp blorp");
    assert_eq!(join(&mut e), ["a", "b"]);
}

#[test]
fn bad_if_skips_its_block_and_dangling_else() {
    let src = "on join:\n    if blorp is wibble wobble:\n        broadcast \"in if\"\n    else:\n        broadcast \"in else\"\n    broadcast \"after\"\n";
    let (mut e, errs) = load(src);
    assert_eq!(errs.len(), 2, "{errs:?}");
    assert_eq!(errs[0].line, 2);
    assert!(
        errs[0].msg.contains("can't understand this condition"),
        "{errs:?}"
    );
    assert_eq!(errs[1].line, 4);
    assert!(
        errs[1].msg.contains("'else' has to be placed just after"),
        "{errs:?}"
    );
    assert_eq!(join(&mut e), ["after"]);
}

#[test]
fn bad_else_if_ends_the_chain() {
    let src = "on join:\n    if 1 is 2:\n        broadcast \"a\"\n    else if blorp blorp:\n        broadcast \"b\"\n    else:\n        broadcast \"c\"\n    broadcast \"after\"\n";
    let (mut e, errs) = load(src);
    assert_eq!(
        errs.iter().map(|e| e.line).collect::<Vec<_>>(),
        [4, 6],
        "{errs:?}"
    );
    assert_eq!(join(&mut e), ["after"]);
}

#[test]
fn bad_while_and_loop_skip_their_block() {
    let src = "on join:\n    while blorp blorp:\n        broadcast \"w\"\n    loop blorp blorp:\n        broadcast \"l\"\n    broadcast \"after\"\n";
    let (mut e, errs) = load(src);
    assert_eq!(errs.len(), 2, "{errs:?}");
    assert_eq!(join(&mut e), ["after"]);
}

#[test]
fn bad_trigger_header_skips_only_that_trigger() {
    let src = "on wibble:\n    broadcast \"no\"\non join:\n    broadcast \"yes\"\ncommand /x <thing>:\n    trigger:\n        broadcast \"no\"\n";
    let (mut e, errs) = load(src);
    assert_eq!(
        errs.iter().map(|e| e.line).collect::<Vec<_>>(),
        [1, 5],
        "{errs:?}"
    );
    assert_eq!(join(&mut e), ["yes"]);
    assert!(e.table().command_names().is_empty());
}

#[test]
fn bad_indentation_and_options_skip_only_those_lines() {
    let src = "options:\n    greet: hi\non join:\n    broadcast \"{@greet}\"\n    broadcast \"{@nope}\"\n      broadcast \"odd\"\n    broadcast \"end\"\n";
    let (mut e, errs) = load(src);
    assert_eq!(errs.len(), 2, "{errs:?}");
    assert_eq!(join(&mut e), ["hi", "end"]);
}

#[test]
fn call_of_function_that_failed_to_load_is_reported_at_the_call() {
    let src = "function f(n: number = blorp blorp):\n    broadcast \"f\"\non join:\n    f(1)\n    broadcast \"after\"\n";
    let (mut e, errs) = load(src);
    assert_eq!(
        errs.iter().map(|e| e.line).collect::<Vec<_>>(),
        [1, 4],
        "{errs:?}"
    );
    assert!(errs[1].msg.contains("failed to load"), "{errs:?}");
    assert_eq!(join(&mut e), ["after"]);
}

#[test]
fn function_with_a_bad_line_still_loads() {
    let src = "function f():\n    blorp\n    broadcast \"f\"\non join:\n    f()\n";
    let (mut e, errs) = load(src);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert_eq!(join(&mut e), ["f"]);
}

#[test]
fn reload_replaces_with_the_partial_version() {
    let (mut e, _) = load("on join:\n    broadcast \"old\"\n");
    let (sum, errs) = e.reload(&[(
        "t.carve".into(),
        "on join:\n    broadcast \"new\"\n    blorp\n".into(),
    )]);
    assert_eq!(errs.len(), 1);
    assert_eq!(sum.triggers, 1);
    assert_eq!(join(&mut e), ["new"]);
    assert_eq!(e.errors.len(), 1);
}

struct T {
    engine: Engine,
    store: Store,
    registered: BTreeSet<String>,
}

impl T {
    fn new(files: &[(&str, &str)]) -> Self {
        let mut store = Store::default();
        for (n, s) in files {
            store.files.insert(n.to_string(), s.to_string());
        }
        let mut engine = Engine::default();
        let enabled: Vec<(String, String)> = store
            .files
            .iter()
            .filter(|(n, _)| !n.starts_with('-'))
            .map(|(n, s)| (n.clone(), s.clone()))
            .collect();
        engine.load(&enabled);
        let registered = engine.table().command_names().into_iter().collect();
        T {
            engine,
            store,
            registered,
        }
    }

    fn lines(&mut self, args: &str) -> Vec<String> {
        let allowed = |_: &str| false;
        let env = AdminEnv {
            sender: Recipient::Console,
            allowed: &allowed,
            registered: &self.registered,
            version: "0.2.0",
            api: "v0.1",
            data_folder: "plugins/data/carve",
        };
        let mut h = H::default();
        admin(&mut self.engine, &mut self.store, &mut h, &env, args).lines
    }
}

const P: &str = "&8[&6Carve&8]&r ";

#[test]
fn reload_messages_in_skript_layout() {
    let mut t = T::new(&[("bank.carve", "on join:\n    broadcast \"a\"\n")]);
    let ok = t.lines("reload bank");
    assert_eq!(ok[0], format!("{P}&7Reloading &fbank.carve&7..."));
    assert!(
        ok[1].starts_with(&format!("{P}&aSuccessfully reloaded &fbank.carve&a. &7(")),
        "{ok:?}"
    );
    assert!(ok[1].ends_with("ms)"), "{ok:?}");

    let bad = "function f() :: number:\n    return d...\non join:\n    broadcast \"a\"\n";
    t.store.files.insert("bank.carve".into(), bad.into());
    let out = t.lines("reload BANK.carve");
    assert_eq!(out[0], format!("{P}&7Reloading &fbank.carve&7..."));
    assert_eq!(out[1], format!("{P}&4Line 2: &c(bank.carve)"));
    assert_eq!(out[2], "    &cCan't understand this expression: 'd...'");
    assert_eq!(out[3], "    &7Line: return d...");
    assert!(
        out[4].starts_with(&format!(
            "{P}&cEncountered &f1 &cerror while reloading &fbank.carve&c! &7("
        )),
        "{out:?}"
    );
    // the partial version is live
    let mut h = H::default();
    t.engine.fire(EventKind::Join, EventCtx::console(), &mut h);
    assert_eq!(h.calls, ["a"]);

    t.store
        .files
        .insert("bank.carve".into(), "on join:\n    blorp\n    blorp\n".into());
    let all = t.lines("reload all");
    assert_eq!(all[0], format!("{P}&7Reloading all scripts..."));
    assert!(
        all.iter()
            .any(|l| l.contains("&cEncountered &f2 &cerrors while reloading &fall scripts&c!")),
        "{all:?}"
    );
}

#[test]
fn suggestions_are_an_extra_indented_line() {
    let mut t = T::new(&[("a.carve", "on join:\n    sendd \"hi\" to player\n")]);
    let out = t.lines("errors a");
    assert!(out[1].starts_with("    &c"), "{out:?}");
    assert!(
        out.iter().any(|l| l.starts_with("    &7Did you mean: ")),
        "{out:?}"
    );
    assert!(
        out.iter().any(|l| l == "    &7Line: sendd \"hi\" to player"),
        "{out:?}"
    );
}

#[test]
fn unknown_script_message() {
    let mut t = T::new(&[("a.carve", "on join:\n    broadcast \"a\"\n")]);
    let want = format!("{P}&cCan't find the script &f'arg'&c.");
    assert_eq!(t.lines("reload arg"), std::slice::from_ref(&want));
    assert_eq!(t.lines("enable arg"), std::slice::from_ref(&want));
    assert_eq!(t.lines("info arg"), std::slice::from_ref(&want));
    assert_eq!(t.lines("errors arg"), std::slice::from_ref(&want));
    // case and extension don't matter
    assert!(t.lines("reload A.CARVE")[1].contains("Successfully reloaded &fa.carve"));
}

#[test]
fn run_errors_use_the_same_layout() {
    let mut t = T::new(&[]);
    let out = t.lines("run blorp blorp");
    assert_eq!(out[0], format!("{P}&4Line 1: &c(run)"));
    assert!(
        out[1].starts_with("    &cCan't understand this effect: 'blorp blorp'"),
        "{out:?}"
    );
}

#[test]
fn server_start_report() {
    let mut e = Engine::default();
    let files = [
        (
            "a.carve".to_string(),
            "on join:\n    blorp\nfunction f():\n    stop\n".to_string(),
        ),
        ("b.carve".to_string(), "on quit:\n    stop\n".to_string()),
    ];
    let (sum, errs) = e.load(&files);
    let rep = load_report(&sum, &errs, 0.04);
    assert_eq!(rep[0], (true, "Line 2: (a.carve)".to_string()));
    assert_eq!(rep[2], (true, "    Line: blorp".to_string()));
    assert!(rep[3].1.starts_with("    Did you mean: "), "{rep:?}");
    assert_eq!(
        rep[4],
        (true, "Encountered 1 error while loading the scripts!".to_string())
    );
    assert_eq!(
        rep[5],
        (
            false,
            "Loaded 2 scripts with a total of 3 structures in 0.04 seconds".to_string()
        )
    );
}
