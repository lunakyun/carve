use std::collections::HashMap;

use carve_core::compile::{CONDITION_PATTERNS, EFFECT_PATTERNS, EventKind, check_registry, compile_file};
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    online: HashMap<String, String>, // uuid -> name
    perms: Vec<(String, String)>,    // (uuid, node)
    waits: Vec<(u64, u64)>,          // (ticks, token)
    now: u64,
    logs: Vec<(LogLevel, String)>,
    resolves: usize,
}

impl Mock {
    fn with_steve() -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m
    }
    fn take(&mut self) -> Vec<String> {
        std::mem::take(&mut self.calls)
    }
    fn errors(&self) -> Vec<&String> {
        self.logs
            .iter()
            .filter(|(l, _)| *l == LogLevel::Error)
            .map(|(_, m)| m)
            .collect()
    }
}

fn who(r: &Recipient) -> String {
    match r {
        Recipient::Player(p) => p.name.clone(),
        Recipient::Console => "console".into(),
    }
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        self.calls.push(format!("send {} {}", who(to), text));
    }
    fn broadcast(&mut self, text: &str) {
        self.calls.push(format!("broadcast {text}"));
    }
    fn send_title(&mut self, p: &PlayerRef, title: &str, sub: Option<&str>) {
        self.calls
            .push(format!("title {} {} / {}", p.name, title, sub.unwrap_or("-")));
    }
    fn send_action_bar(&mut self, p: &PlayerRef, text: &str) {
        self.calls.push(format!("actionbar {} {}", p.name, text));
    }
    fn heal(&mut self, p: &PlayerRef) {
        self.calls.push(format!("heal {}", p.name));
    }
    fn set_health(&mut self, p: &PlayerRef, h: f64) {
        self.calls.push(format!("health {} {h}", p.name));
    }
    fn kill(&mut self, p: &PlayerRef) {
        self.calls.push(format!("kill {}", p.name));
    }
    fn teleport(&mut self, p: &PlayerRef, x: f64, y: f64, z: f64) {
        self.calls.push(format!("tp {} {x} {y} {z}", p.name));
    }
    fn execute_command(&mut self, c: &str) {
        self.calls.push(format!("cmd {c}"));
    }
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        self.resolves += 1;
        self.online.get(uuid).cloned()
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        self.online
            .iter()
            .find(|(_, n)| n.eq_ignore_ascii_case(name))
            .map(|(u, n)| PlayerRef {
                uuid: u.clone(),
                name: n.clone(),
            })
    }
    fn has_permission(&mut self, p: &PlayerRef, node: &str) -> bool {
        self.perms.iter().any(|(u, n)| *u == p.uuid && n == node)
    }
    fn schedule_wait(&mut self, ticks: u64, token: u64) {
        self.waits.push((ticks, token));
    }
    fn now(&mut self) -> u64 {
        self.now
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn engine(src: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("test.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "unexpected errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn errors_of(src: &str) -> Vec<ScriptError> {
    compile_file("bad.carve", src).err().unwrap_or_default()
}

fn join(src: &str) -> Vec<String> {
    let mut e = engine(src);
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.errors().is_empty(), "runtime errors: {:?}", h.errors());
    h.take()
}

// ---------- parsing ----------

#[test]
fn registry_patterns_compile() {
    check_registry().unwrap();
}

#[test]
fn every_effect_pattern_parses() {
    let lines = [
        r#"send title "Hi" with subtitle "there" to player"#,
        r#"send title "Hi" to player"#,
        r#"send action bar "bar" to player"#,
        r#"send "hello""#,
        r#"send "hello" to player"#,
        r#"broadcast "all""#,
        "heal player",
        "set health of player to 10",
        "kill player",
        "teleport player to 1, 64, -2.5",
        r#"execute console command "say hi""#,
        "cancel event",
        "cancel the event",
        "wait 1 tick",
        "wait 5 ticks",
        "wait 1 second",
        "wait 2 seconds",
        "set {x} to 5",
        "add 1 to {x}",
        "remove 2 from {x}",
        "delete {x}",
    ];
    assert_eq!(lines.len(), EFFECT_PATTERNS.len() + 5);
    for l in lines {
        let src = format!("on join:\n    {l}\n");
        let errs = errors_of(&src);
        assert!(errs.is_empty(), "'{l}' failed: {:?}", errs);
    }
}

#[test]
fn every_condition_pattern_parses() {
    let conds = [
        "{x} is set",
        "{x} is not set",
        "{x} is not 5",
        "{x} != 5",
        "{x} is 5",
        "{x} = 5",
        "{x} >= 5",
        "{x} <= 5",
        "{x} > 5",
        "{x} < 5",
        r#"player has permission "a.b""#,
        r#"player does not have permission "a.b""#,
    ];
    assert_eq!(conds.len(), CONDITION_PATTERNS.len() + 2);
    for c in conds {
        let src = format!("on join:\n    if {c}:\n        send \"y\"\n");
        let errs = errors_of(&src);
        assert!(errs.is_empty(), "'{c}' failed: {:?}", errs);
    }
}

#[test]
fn tabs_and_comments() {
    let out = join("# a comment\non join:\n\tsend \"a # not a comment\" # real comment\n\n\tsend \"b\"\n");
    assert_eq!(out, ["send Steve a # not a comment", "send Steve b"]);
}

#[test]
fn unknown_effect_reports_line_and_did_you_mean() {
    let errs = errors_of("on join:\n    send \"ok\"\n    sned \"hi\" to player\n");
    assert_eq!(errs.len(), 1);
    let e = &errs[0];
    assert_eq!((e.file.as_str(), e.line), ("bad.carve", 3));
    assert_eq!(e.suggestions.len(), 3);
    assert!(
        e.suggestions.iter().any(|s| s.starts_with("send %text%")),
        "{:?}",
        e.suggestions
    );
    assert!(e.to_string().starts_with("bad.carve:3: "));
}

#[test]
fn type_errors_are_reported() {
    let errs = errors_of("on join:\n    kill \"Steve\"\n");
    assert_eq!(errs.len(), 1);
    assert!(errs[0].msg.contains("expected a player"), "{}", errs[0].msg);
    let errs = errors_of("on join:\n    set health of player to \"lots\"\n");
    assert!(errs[0].msg.contains("expected a number"), "{}", errs[0].msg);
}

#[test]
fn structural_errors() {
    assert!(
        errors_of("on jump:\n    send \"x\"\n")[0]
            .msg
            .contains("unknown event")
    );
    assert!(errors_of("on join:\n")[0].msg.contains("empty block"));
    assert!(
        errors_of("on join:\n    send \"x\"\n        send \"y\"\n")[0]
            .msg
            .contains("unexpected indentation")
    );
    assert!(
        errors_of("on join:\n    send \"x\"\n\tsend \"y\"\n")[0]
            .msg
            .contains("tabs")
    );
    assert!(
        errors_of("on join:\n    else:\n        send \"x\"\n")[0]
            .msg
            .contains("without an 'if'")
    );
    assert!(
        errors_of("on join:\n    send message\n")[0]
            .msg
            .contains("can't understand")
    );
    assert!(
        errors_of("command /x:\n    trigger:\n        cancel event\n")[0]
            .msg
            .contains("no event to cancel")
    );
    assert!(
        errors_of("command /x:\n    trigger:\n        send arg\n")[0]
            .msg
            .contains("can't understand")
    );
    assert!(
        errors_of("on join:\n    send \"100%\"\n")[0]
            .msg
            .contains("unclosed '%'")
    );
    assert!(
        errors_of("on join:\n    send \"oops\n")[0]
            .msg
            .contains("unclosed text")
    );
}

#[test]
fn multiple_errors_in_one_file() {
    let errs = errors_of("on join:\n    blah\n    send \"ok\"\n    bleh\non chat:\n    blub\n");
    let lines: Vec<usize> = errs.iter().map(|e| e.line).collect();
    assert_eq!(lines, [2, 4, 6]);
}

// ---------- runtime ----------

#[test]
fn templates_and_effects() {
    let out = join(
        r#"on join:
    send "Welcome %player%! 100%% fun"
    send title "Hi %name of player%" with subtitle "sub" to player
    send title "Plain" to player
    send action bar "bar" to player
    broadcast "%player% joined"
    heal player
    set health of player to 5
    teleport player to 1, 2 + 3, -4
    execute console command "/say hi %player%"
    kill player
"#,
    );
    assert_eq!(
        out,
        [
            "send Steve Welcome Steve! 100% fun",
            "title Steve Hi Steve / sub",
            "title Steve Plain / -",
            "actionbar Steve bar",
            "broadcast Steve joined",
            "heal Steve",
            "health Steve 5",
            "tp Steve 1 5 -4",
            "cmd say hi Steve",
            "kill Steve",
        ]
    );
}

#[test]
fn arithmetic() {
    let out = join(
        r#"on join:
    set {_a} to 2 + 3 * 4
    set {_b} to (2 + 3) * 4
    set {_c} to 10 / 4 - -1
    set {_d} to {_a} - {_b}
    send "%{_a}% %{_b}% %{_c}% %{_d}% %7 / 2%"
"#,
    );
    assert_eq!(out, ["send Steve 14 20 3.5 -6 3.5"]);
}

#[test]
fn division_by_zero_is_runtime_error_with_line() {
    let mut e = engine("on join:\n    send \"a\"\n    send \"%1 / 0%\"\n    send \"never\"\n");
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.take(), ["send Steve a"]);
    assert_eq!(h.errors().len(), 1);
    assert!(h.errors()[0].starts_with("test.carve:3: division by zero"));
}

#[test]
fn if_else_chains() {
    let src = r#"on join:
    if {n} = 1:
        send "one"
    else if {n} is 2:
        send "two"
    else if {n} > 2:
        send "big"
        if {n} >= 10:
            send "huge"
    else:
        send "small"
    send "end"
"#;
    let mut e = engine(src);
    let cases: [(f64, &[&str]); 5] = [
        (1.0, &["one", "end"]),
        (2.0, &["two", "end"]),
        (3.0, &["big", "end"]),
        (12.0, &["big", "huge", "end"]),
        (0.0, &["small", "end"]),
    ];
    for (n, want) in cases {
        e.vars.set("n".into(), Value::Num(n));
        let mut h = Mock::with_steve();
        e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
        let want: Vec<String> = want.iter().map(|w| format!("send Steve {w}")).collect();
        assert_eq!(h.take(), want, "n = {n}");
    }
}

#[test]
fn comparisons() {
    let out = join(
        r#"on join:
    set {_t} to "Hello"
    if {_t} is "hello":
        send "text eq ignores case"
    if {_t} is not "bye":
        send "ne"
    if {_t} != "hello":
        send "WRONG"
    if 3 <= 3:
        send "le"
    if 3 < 3:
        send "WRONG"
    if player is "steve":
        send "player eq name"
    if {_missing} is {_other}:
        send "none eq none"
    if {_t} > 1:
        send "WRONG"
"#,
    );
    assert_eq!(
        out,
        [
            "send Steve text eq ignores case",
            "send Steve ne",
            "send Steve le",
            "send Steve player eq name",
            "send Steve none eq none"
        ]
    );
}

#[test]
fn permissions_condition() {
    let src = "on join:\n    if player has permission \"vip\":\n        send \"vip\"\n    if player does not have permission \"vip\":\n        send \"pleb\"\n";
    let mut e = engine(src);
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.take(), ["send Steve pleb"]);
    h.perms.push(("u1".into(), "vip".into()));
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.take(), ["send Steve vip"]);
}

#[test]
fn variables_global_local_lists() {
    let src = r#"on join:
    add 1 to {joins::%player%}
    add 1 to {total}
    remove 0.5 from {total}
    set {_x} to "local"
    set {last} to player
    send "%{joins::steve}% %{total}% %{_x}% %{last}%"
"#;
    let mut e = engine(src);
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(
        h.take(),
        ["send Steve 1 0.5 local Steve", "send Steve 2 1 local Steve"]
    );
    // locals don't leak between runs
    assert_eq!(e.vars.get("_x"), Value::None);
    // list delete
    let mut e2 = engine(
        "on join:\n    set {l::a} to 1\n    set {l::b} to 2\n    set {keep} to 3\n    delete {l::*}\n    delete {keep2}\n",
    );
    e2.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(e2.vars.get("l::a"), Value::None);
    assert_eq!(e2.vars.get("keep"), Value::Num(3.0));
}

#[test]
fn add_to_text_variable_is_error() {
    let mut e = engine("on join:\n    set {x} to \"abc\"\n    add 1 to {x}\n");
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.errors().len(), 1);
    assert!(h.errors()[0].starts_with("test.carve:3:"));
}

#[test]
fn persistence_round_trip_and_debounce() {
    let mut e = engine(
        "on join:\n    set {n} to 4.5\n    set {t} to \"hi \"\"q\"\"\"\n    set {p} to player\n    set {list::a} to 1\n",
    );
    let mut h = Mock::with_steve();
    h.now = 1000;
    assert_eq!(e.save_due(1000), None);
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(e.save_due(5999), None, "debounce not elapsed");
    let json = e.save_due(6000).expect("save after 5s");
    assert_eq!(e.save_due(20000), None, "nothing new to save");
    let back = Variables::from_json(&json).unwrap();
    assert_eq!(back.get("n"), Value::Num(4.5));
    assert_eq!(back.get("t"), Value::Text("hi \"q\"".into()));
    assert_eq!(back.get("p"), Value::Player(steve()));
    assert_eq!(back.get("list::a"), Value::Num(1.0));
    assert_eq!(back.len(), 4);
    // unload save
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(e.save_now().is_some());
    assert!(e.save_now().is_none());
    assert!(Variables::from_json("not json").is_err());
    assert!(Variables::from_json("").unwrap().is_empty());
}

#[test]
fn wait_and_resume() {
    let mut e = engine(
        "on join:\n    send \"a\"\n    wait 2 seconds\n    send \"b\"\n    wait 3 ticks\n    send \"c %player%\"\n",
    );
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.take(), ["send Steve a"]);
    assert_eq!(h.waits.len(), 1);
    let (ticks, tok) = h.waits[0];
    assert_eq!(ticks, 40);
    e.resume(tok, &mut h);
    assert_eq!(h.take(), ["send Steve b"]);
    let (ticks, tok2) = h.waits[1];
    assert_eq!(ticks, 3);
    e.resume(tok2, &mut h);
    assert_eq!(h.take(), ["send Steve c Steve"]);
    assert_eq!(e.pending_waits(), 0);
    e.resume(tok2, &mut h); // stale token is a no-op
    assert!(h.take().is_empty());
}

#[test]
fn offline_player_after_wait_is_skipped() {
    let mut e = engine(
        "on join:\n    wait 1 tick\n    send \"hi\" to player\n    heal player\n    send \"you are %player%\"\n",
    );
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    h.online.clear();
    let tok = h.waits[0].1;
    e.resume(tok, &mut h);
    assert!(h.take().is_empty(), "effects on an offline player are skipped");
    assert!(h.errors().is_empty());
}

#[test]
fn cancel_before_and_after_wait() {
    let mut e = engine("on break:\n    cancel event\n");
    let mut h = Mock::with_steve();
    assert!(
        e.fire(EventKind::Break, EventCtx::player(steve()), &mut h)
            .cancelled
    );
    assert!(
        !e.fire(EventKind::Place, EventCtx::player(steve()), &mut h)
            .cancelled
    );

    let mut e = engine("on break:\n    wait 1 tick\n    cancel event\n");
    let out = e.fire(EventKind::Break, EventCtx::player(steve()), &mut h);
    assert!(!out.cancelled);
    let tok = h.waits[0].1;
    e.resume(tok, &mut h);
    let warns: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Warn).collect();
    assert_eq!(warns.len(), 1);
    assert!(warns[0].1.starts_with("test.carve:3:"));

    // cancel then wait: still cancelled
    let mut e = engine("on break:\n    cancel event\n    wait 1 tick\n    send \"later\"\n");
    assert!(
        e.fire(EventKind::Break, EventCtx::player(steve()), &mut h)
            .cancelled
    );
}

#[test]
fn chat_message() {
    let mut e = engine("on chat:\n    if message is \"hi\":\n        send \"you said %message%\"\n");
    let mut h = Mock::with_steve();
    e.fire(EventKind::Chat, EventCtx::chat(steve(), "HI"), &mut h);
    assert_eq!(h.take(), ["send Steve you said HI"]);
    assert!(
        errors_of("on join:\n    send message\n")[0]
            .msg
            .contains("can't understand")
    );
}

#[test]
fn command_args() {
    let src = r#"command /greet <player> [<text>]:
    permission: test.greet
    description: say hi
    trigger:
        if arg-2 is fancy:
            send "x"
        send "hi %arg-1% %arg-2%" to arg-1
command /add <number> <number>:
    trigger:
        send "%arg-1 + arg-2%"
command /echo [<text>]:
    trigger:
        send "echo: %arg%"
"#;
    // `is fancy` isn't syntax: expect exactly that one error.
    let errs = errors_of(src);
    assert_eq!(errs.len(), 1, "{errs:?}");
    let src = src.replace(
        "        if arg-2 is fancy:\n            send \"x\"\n",
        "        if arg-2 is not set:\n            send \"no text\"\n",
    );
    let mut e = engine(&src);
    assert_eq!(e.table().command_names(), ["add", "echo", "greet"]);
    assert_eq!(
        e.table().command("greet").unwrap().usage(),
        "/greet <player> [<text>]"
    );
    let mut h = Mock::with_steve();
    let me = Recipient::Player(steve());
    assert_eq!(
        e.run_command("greet", me.clone(), "steve", &mut h),
        CommandOutcome::NoPermission
    );
    h.perms.push(("u1".into(), "test.greet".into()));
    assert_eq!(
        e.run_command("greet", me.clone(), "steve how are you", &mut h),
        CommandOutcome::Ran
    );
    assert_eq!(
        e.run_command("/GREET", Recipient::Console, "steve", &mut h),
        CommandOutcome::Ran
    );
    assert_eq!(
        h.take(),
        [
            "send Steve hi Steve how are you",
            "send console no text",
            "send Steve hi Steve <none>"
        ]
    );
    assert!(matches!(
        e.run_command("greet", me.clone(), "", &mut h),
        CommandOutcome::Usage(_)
    ));
    assert!(
        matches!(e.run_command("greet", me.clone(), "alex", &mut h), CommandOutcome::Usage(m) if m.contains("not online"))
    );
    assert_eq!(
        e.run_command("add", me.clone(), "2 3.5", &mut h),
        CommandOutcome::Ran
    );
    assert!(matches!(
        e.run_command("add", me.clone(), "2 x", &mut h),
        CommandOutcome::Usage(_)
    ));
    assert!(matches!(
        e.run_command("add", me.clone(), "1 2 3", &mut h),
        CommandOutcome::Usage(_)
    ));
    assert_eq!(
        e.run_command("echo", Recipient::Console, "", &mut h),
        CommandOutcome::Ran
    );
    assert_eq!(
        e.run_command("nope", Recipient::Console, "", &mut h),
        CommandOutcome::NotFound
    );
    assert_eq!(h.take(), ["send Steve 5.5", "send console echo: <none>"]);
    assert!(
        errors_of("command /x <text>:\n    trigger:\n        send arg-2\n")[0]
            .msg
            .contains("can't understand")
    );
}

#[test]
fn reload_keeps_old_scripts_on_error() {
    let mut e = engine("on join:\n    send \"v1\"\n");
    let mut h = Mock::with_steve();
    let bad = [
        ("a.carve".to_string(), "on join:\n    send \"v2\"\n".to_string()),
        ("b.carve".into(), "on join:\n    oops\n".into()),
    ];
    let errs = e.reload(&bad).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!((errs[0].file.as_str(), errs[0].line), ("b.carve", 2));
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.take(), ["send Steve v1"]);
    let good = [("a.carve".to_string(), "on join:\n    send \"v2\"\n".to_string())];
    let sum = e.reload(&good).unwrap();
    assert_eq!((sum.files, sum.triggers), (1, 1));
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.take(), ["send Steve v2"]);
}

#[test]
fn in_flight_wait_finishes_on_old_code() {
    let mut e = engine("on join:\n    wait 1 tick\n    send \"old\"\n");
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    e.reload(&[("t.carve".into(), "on join:\n    send \"new\"\n".into())])
        .unwrap();
    let tok = h.waits[0].1;
    e.resume(tok, &mut h);
    assert_eq!(h.take(), ["send Steve old"]);
}

#[test]
fn initial_load_is_partial() {
    let mut e = Engine::default();
    let files = [
        ("good.carve".to_string(), "on join:\n    send \"g\"\n".to_string()),
        ("bad.carve".into(), "on join:\n    nope\n".into()),
    ];
    let (sum, errs) = e.load(&files);
    assert_eq!(sum.files, 1);
    assert_eq!(errs.len(), 1);
    let dup = [
        (
            "a.carve".to_string(),
            "command /x:\n    trigger:\n        send \"a\"\n".to_string(),
        ),
        (
            "b.carve".into(),
            "command /x:\n    trigger:\n        send \"b\"\n".into(),
        ),
    ];
    assert!(e.reload(&dup).unwrap_err()[0].msg.contains("defined twice"));
}

#[test]
fn runaway_guard() {
    let body: String = (0..50).map(|i| format!("    set {{_x}} to {i}\n")).collect();
    let mut e = engine(&format!("on join:\n{body}    send \"done\"\n"));
    e.max_instructions = 20;
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.take().is_empty());
    assert_eq!(h.errors().len(), 1);
    assert!(
        h.errors()[0].contains("more than 20 instructions"),
        "{:?}",
        h.errors()
    );
    // a wait starts a fresh slice
    let mut e = engine(
        "on join:\n    set {_a} to 1\n    set {_a} to 2\n    wait 1 tick\n    set {_a} to 3\n    send \"ok\"\n",
    );
    e.max_instructions = 3;
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let tok = h.waits[0].1;
    e.resume(tok, &mut h);
    assert_eq!(h.take(), ["send Steve ok"]);
    assert!(h.errors().is_empty());
}

#[test]
fn examples_compile() {
    let dir = format!("{}/../../examples", env!("CARGO_MANIFEST_DIR"));
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|x| x == "carve") {
            let src = std::fs::read_to_string(&path).unwrap();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let errs = errors_of(&src);
            assert!(errs.is_empty(), "{name}: {errs:?}");
            names.push(name);
        }
    }
    names.sort();
    for want in [
        "chat-filter.carve",
        "economy.carve",
        "heal.carve",
        "join-counter.carve",
        "kit.carve",
        "welcome.carve",
    ] {
        assert!(
            names.iter().any(|n| n == want),
            "missing example {want}, found {names:?}"
        );
    }
    // All examples together must load as one script set (no duplicate commands).
    let files: Vec<(String, String)> = names
        .iter()
        .map(|n| (n.clone(), std::fs::read_to_string(format!("{dir}/{n}")).unwrap()))
        .collect();
    let (_, errs) = Engine::default().load(&files);
    assert!(errs.is_empty(), "{errs:?}");
}

#[test]
fn docs_list_every_pattern() {
    let root = format!("{}/../..", env!("CARGO_MANIFEST_DIR"));
    let effects = std::fs::read_to_string(format!("{root}/docs/syntax/effects.md")).unwrap();
    let conditions = std::fs::read_to_string(format!("{root}/docs/syntax/conditions.md")).unwrap();
    for (pat, _) in EFFECT_PATTERNS {
        assert!(
            effects.contains(pat),
            "docs/syntax/effects.md is missing the pattern `{pat}`"
        );
    }
    for (pat, _) in CONDITION_PATTERNS {
        assert!(
            conditions.contains(pat),
            "docs/syntax/conditions.md is missing the pattern `{pat}`"
        );
    }
}

#[test]
fn economy_example_runs() {
    let src = std::fs::read_to_string(format!(
        "{}/../../examples/economy.carve",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let mut e = engine(&src);
    let mut h = Mock::with_steve();
    h.online.insert("u2".into(), "Alex".into());
    let alex = PlayerRef {
        uuid: "u2".into(),
        name: "Alex".into(),
    };
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    e.fire(EventKind::Join, EventCtx::player(alex.clone()), &mut h);
    let me = Recipient::Player(steve());
    assert_eq!(
        e.run_command("pay", me.clone(), "alex 30", &mut h),
        CommandOutcome::Ran
    );
    assert_eq!(
        e.run_command("pay", me.clone(), "alex 500", &mut h),
        CommandOutcome::Ran
    );
    assert_eq!(
        e.run_command("pay", me.clone(), "steve 1", &mut h),
        CommandOutcome::Ran
    );
    assert_eq!(e.run_command("balance", me, "", &mut h), CommandOutcome::Ran);
    assert_eq!(
        e.run_command("balance", Recipient::Player(alex), "", &mut h),
        CommandOutcome::Ran
    );
    assert_eq!(
        h.take(),
        [
            "send Steve &aYou paid 30 coins to Alex.",
            "send Alex &aSteve paid you 30 coins.",
            "send Steve &cYou only have 70 coins.",
            "send Steve &cYou can't pay yourself.",
            "send Steve &aYou have 70 coins.",
            "send Alex &aYou have 130 coins.",
        ]
    );
}

// ---------- performance semantics ----------

#[test]
fn player_resolved_once_per_slice() {
    let mut e = engine(
        "on join:
    send \"%player% %player%\" to player
    heal player
    wait 1 tick
    heal player
    send \"%player%\"
",
    );
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.resolves, 1, "one lookup before the wait");
    let tok = h.waits[0].1;
    e.resume(tok, &mut h);
    assert_eq!(
        h.resolves, 2,
        "the cache is cleared at the wait, so one more lookup after it"
    );
    assert_eq!(
        h.take(),
        [
            "send Steve Steve Steve",
            "heal Steve",
            "heal Steve",
            "send Steve Steve"
        ]
    );
}

#[test]
fn local_lists_and_static_keys() {
    let out = join(
        r#"on join:
    set {_l::a} to 1
    set {_l::b} to 2
    set {_keep} to 3
    delete {_l::*}
    set {_Mixed} to "m"
    add 2 to {_KEEP}
    send "%{_l::a}% %{_l::b}% %{_keep}% %{_mixed}%"
"#,
    );
    assert_eq!(out, ["send Steve <none> <none> 5 m"]);
}

#[test]
fn constant_folding_keeps_meaning() {
    let out = join(
        "on join:
    send \"%1 + 2 * 3% %-(4 - 1)% %10 / 4% %\"\"x\"\"% %%\"
",
    );
    assert_eq!(out, ["send Steve 7 -3 2.5 x %"]);
    // division by zero is still a runtime error with its line
    let mut e = engine(
        "on join:
    set {_a} to 2 / 0
",
    );
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.errors()[0].starts_with("test.carve:2: division by zero"));
}

#[test]
fn chat_filter_example_runs() {
    let src = std::fs::read_to_string(format!(
        "{}/../../examples/chat-filter.carve",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let mut e = engine(&src);
    let mut h = Mock::with_steve();
    assert!(
        !e.fire(EventKind::Chat, EventCtx::chat(steve(), "hello"), &mut h)
            .cancelled
    );
    for _ in 0..2 {
        assert!(
            e.fire(EventKind::Chat, EventCtx::chat(steve(), "SPAM"), &mut h)
                .cancelled
        );
    }
    assert!(
        !e.fire(EventKind::Chat, EventCtx::chat(steve(), "hi again"), &mut h)
            .cancelled
    );
    assert!(
        e.fire(EventKind::Chat, EventCtx::chat(steve(), "badword"), &mut h)
            .cancelled
    );
    assert_eq!(
        h.take(),
        [
            "send Steve &cPlease keep the chat friendly. Warning 1 of 3.",
            "send Steve &cPlease keep the chat friendly. Warning 2 of 3.",
            "cmd kick Steve Please keep the chat friendly",
        ]
    );
    assert_eq!(e.vars.get("warnings::steve"), Value::None);
}
