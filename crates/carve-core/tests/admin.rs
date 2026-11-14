use std::collections::{BTreeMap, BTreeSet};

use carve_core::admin::{AdminEnv, AdminOut, ScriptStore, admin, suggest};
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
    fn send_message(&mut self, to: &Recipient, text: &str) {
        let who = match to {
            Recipient::Player(p) => p.name.clone(),
            Recipient::Console => "console".into(),
        };
        self.calls.push(format!("send {who} {text}"));
    }
    fn broadcast(&mut self, text: &str) {
        self.calls.push(format!("broadcast {text}"));
    }
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.calls.push(format!("cmd {c}"));
    }
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
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

struct T {
    engine: Engine,
    store: Store,
    host: H,
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
            host: H::default(),
            registered,
        }
    }

    fn run_as(&mut self, sender: Recipient, perms: &[&str], args: &str) -> AdminOut {
        let perms: Vec<String> = perms.iter().map(|p| p.to_string()).collect();
        let allowed = move |n: &str| perms.iter().any(|p| p == n);
        let env = AdminEnv {
            sender,
            allowed: &allowed,
            registered: &self.registered,
            version: "0.2.0",
            api: "v0.1",
            data_folder: "plugins/data/carve",
        };
        admin(&mut self.engine, &mut self.store, &mut self.host, &env, args)
    }

    fn run(&mut self, args: &str) -> String {
        self.run_as(Recipient::Console, &[], args).lines.join("\n")
    }
}

fn steve() -> Recipient {
    Recipient::Player(PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    })
}

const A: &str = "on join:\n    broadcast \"a\"\n";
const B: &str = "function twice(n: number) :: number:\n    return {_n} * 2\ncommand /bb:\n    trigger:\n        send \"%twice(2)%\"\n";
const USES_B: &str = "on join:\n    broadcast \"%twice(3)%\"\n";

#[test]
fn help_shows_only_allowed_subcommands() {
    let mut t = T::new(&[("a.carve", A)]);
    let all = t.run("help");
    assert!(all.contains("/carve reload") && all.contains("/carve vars") && all.contains("Carve 0.2.0"));
    let some = t.run_as(steve(), &["carve:reload"], "").lines.join("\n");
    assert!(
        some.contains("/carve reload") && !some.contains("/carve vars"),
        "{some}"
    );
    let denied = t.run_as(steve(), &["carve:reload"], "vars list").lines.join("\n");
    assert!(denied.contains("need the carve:vars permission"), "{denied}");
    assert!(
        t.run_as(steve(), &["carve:admin"], "vars list")
            .lines
            .join("\n")
            .contains("variable(s)")
    );
    assert!(t.run("nope").contains("Unknown subcommand"));
}

#[test]
fn reload_all_and_one_file() {
    let mut t = T::new(&[("a.carve", A), ("b.carve", B)]);
    let out = t.run("reload");
    assert!(
        out.contains("Reloading all scripts...") && out.contains("Successfully reloaded &fall scripts"),
        "{out}"
    );
    // one broken line: reported, the rest of the new version loads (Skript)
    t.store.files.insert(
        "a.carve".into(),
        "on join:\n    blorp\n    broadcast \"a1\"\n".into(),
    );
    let out = t.run("reload a");
    assert!(
        out.contains("Encountered &f1 &cerror while reloading &fa.carve&c!"),
        "{out}"
    );
    assert!(
        out.contains("Line 2: &c(a.carve)") && out.contains("Did you mean") && out.contains("&7Line: blorp"),
        "{out}"
    );
    t.engine.fire(EventKind::Join, EventCtx::console(), &mut t.host);
    assert_eq!(t.host.calls, ["broadcast a1"]);
    assert!(t.run("errors").contains("Line 2: &c(a.carve)"));
    assert!(t.run("errors b").contains("No errors"));
    // fixed: reload just that file
    t.store
        .files
        .insert("a.carve".into(), "on join:\n    broadcast \"a2\"\n".into());
    assert!(
        t.run("reload a.carve")
            .contains("Successfully reloaded &fa.carve")
    );
    assert!(t.run("reload zzz").contains("Can't find the script &f'zzz'"));
}

#[test]
fn new_command_on_reload_needs_restart() {
    let mut t = T::new(&[("a.carve", A)]);
    t.store.files.insert("b.carve".into(), B.into());
    let out = t.run("reload");
    assert!(out.contains("New commands need a server restart: /bb"), "{out}");
}

#[test]
fn enable_disable_list() {
    let mut t = T::new(&[
        ("a.carve", A),
        ("-old.carve", "on join:\n    broadcast \"old\"\n"),
    ]);
    let out = t.run("list");
    assert!(
        out.contains("Enabled (1)")
            && out.contains("a.carve")
            && out.contains("Disabled (1)")
            && out.contains("old.carve"),
        "{out}"
    );
    assert!(
        t.run("enable old")
            .contains("Successfully enabled and loaded &fold.carve")
    );
    assert!(t.store.files.contains_key("old.carve"));
    t.engine.fire(EventKind::Join, EventCtx::console(), &mut t.host);
    assert_eq!(t.host.calls, ["broadcast a", "broadcast old"]);
    assert!(t.run("disable a").contains("Successfully disabled &fa.carve"));
    assert!(t.store.files.contains_key("-a.carve"));
    t.host.calls.clear();
    t.engine.fire(EventKind::Join, EventCtx::console(), &mut t.host);
    assert_eq!(t.host.calls, ["broadcast old"]);
    assert!(t.run("disable a").contains("already disabled"));
    assert!(t.run("list disabled").contains("a.carve") && !t.run("list disabled").contains("old.carve"));
    let out = t.run("disable all");
    assert!(out.contains("Successfully disabled &fall scripts"), "{out}");
    assert!(
        t.run("enable all")
            .contains("Successfully enabled and loaded &fall scripts")
    );
}

#[test]
fn disable_reports_errors_it_causes_elsewhere() {
    let mut t = T::new(&[("b.carve", B), ("c.carve", USES_B)]);
    let out = t.run("disable b");
    // Like Skript: the script is disabled; the broken call in c is reported.
    assert!(
        out.contains("Line 2: &c(c.carve)")
            && out.contains("Encountered &f1 &cerror while disabling &fb.carve"),
        "{out}"
    );
    assert!(t.store.files.contains_key("-b.carve"));
    let out = t
        .run_as(Recipient::Console, &[], "enable nothere")
        .lines
        .join("\n");
    assert!(out.contains("Can't find the script &f'nothere'"));
}

#[test]
fn info_general_and_per_script() {
    let mut t = T::new(&[("b.carve", &format!("options:\n    p: x\n{B}"))]);
    t.engine.vars.set("x".into(), Value::Num(1.0));
    let out = t.run("info");
    assert!(
        out.contains("Carve 0.2.0")
            && out.contains("v0.1")
            && out.contains("Scripts: &f1")
            && out.contains("functions: &f1")
            && out.contains("Variables: &f1")
            && out.contains("plugins/data/carve"),
        "{out}"
    );
    let out = t.run("info b");
    assert!(
        out.contains("command /bb") && out.contains("twice(n: number) :: number") && out.contains("p: x"),
        "{out}"
    );
    assert!(t.run("info zz").contains("Can't find the script"));
}

#[test]
fn vars_list_get_set_delete() {
    let mut t = T::new(&[("a.carve", A)]);
    assert!(
        t.run("vars set coins::steve 5")
            .contains("Set {coins::steve} to 5")
    );
    assert!(
        t.run("vars set {motd} \"Hello there\"")
            .contains("\"Hello there\"")
    );
    assert!(t.run("vars set flag true").contains("true"));
    assert_eq!(t.engine.vars.get("coins::steve"), Value::Num(5.0));
    assert_eq!(t.engine.vars.get("motd"), Value::Text("Hello there".into()));
    assert_eq!(t.engine.vars.get("flag"), Value::Bool(true));
    let out = t.run("vars list coins");
    assert!(
        out.contains("1 variable(s) starting with 'coins'") && out.contains("{coins::steve}"),
        "{out}"
    );
    assert!(t.run("vars get {motd}").contains("Hello there"));
    assert!(t.run("vars delete coins::*").contains("Deleted 1"));
    assert!(t.engine.save_now().is_some(), "changes are saved");
    assert!(t.run("vars set {_x} 1").contains("Only single global"));
    assert!(t.run("vars frob").contains("Usage"));
}

#[test]
fn run_one_line() {
    let mut t = T::new(&[("b.carve", B)]);
    let out = t.run("run broadcast \"hi %twice(4)%\"");
    assert!(out.is_empty(), "{out}");
    assert_eq!(t.host.calls, ["broadcast hi 8"]);
    let out = t
        .run_as(steve(), &["carve:run"], "run send \"you are %player%\"")
        .lines
        .join("\n");
    assert!(out.is_empty(), "{out}");
    assert_eq!(t.host.calls.last().unwrap(), "send Steve you are Steve");
    let out = t.run("run brodcast \"x\"");
    assert!(
        out.contains("Can't understand") && out.contains("Did you mean"),
        "{out}"
    );
}

#[test]
fn timings_on_show_reset_off() {
    let mut t = T::new(&[("a.carve", A)]);
    assert!(t.run("timings").contains("Timings are off"));
    t.run("timings on");
    for _ in 0..3 {
        t.engine.fire(EventKind::Join, EventCtx::console(), &mut t.host);
    }
    let out = t.run("timings");
    assert!(out.contains("a.carve:1 &73 run(s)"), "{out}");
    t.run("timings reset");
    assert!(t.run("timings").contains("(0 tracked)"));
    t.run("timings off");
    assert!(t.engine.timings.is_none());
}

#[test]
fn tab_completion() {
    let mut t = T::new(&[("a.carve", A), ("-old.carve", A)]);
    assert_eq!(
        suggest(&mut t.store, ""),
        (
            0,
            carve_core::admin::SUBCOMMANDS
                .iter()
                .map(|s| s.0.to_string())
                .collect()
        )
    );
    assert_eq!(suggest(&mut t.store, "re"), (0, vec!["reload".to_string()]));
    assert_eq!(
        suggest(&mut t.store, "reload "),
        (7, vec!["all".to_string(), "a".to_string()])
    );
    assert_eq!(suggest(&mut t.store, "enable o"), (7, vec!["old".to_string()]));
    assert_eq!(
        suggest(&mut t.store, "timings o"),
        (8, vec!["on".to_string(), "off".to_string()])
    );
    assert_eq!(suggest(&mut t.store, "vars get x y"), (11, Vec::<String>::new()));
}
