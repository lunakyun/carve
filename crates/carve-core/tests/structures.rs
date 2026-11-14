use std::collections::HashMap;

use carve_core::compile::{EventKind, compile_file};
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    online: HashMap<String, String>, // uuid -> name
    waits: Vec<(u64, u64)>,
    logs: Vec<(LogLevel, String)>,
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

impl Host for Mock {
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
    fn execute_command(&mut self, _: &str) {}
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
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
    fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, ticks: u64, token: u64) {
        self.waits.push((ticks, token));
    }
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
    fn online_players(&mut self) -> Vec<PlayerRef> {
        let mut v: Vec<PlayerRef> = self
            .online
            .iter()
            .map(|(u, n)| PlayerRef {
                uuid: u.clone(),
                name: n.clone(),
            })
            .collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn engine(src: &str) -> Engine {
    engine_files(&[("test.carve", src)])
}

fn engine_files(files: &[(&str, &str)]) -> Engine {
    let mut e = Engine::default();
    let files: Vec<(String, String)> = files
        .iter()
        .map(|(n, s)| (n.to_string(), s.to_string()))
        .collect();
    let (_, errs) = e.load(&files);
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

fn load_errors(files: &[(&str, &str)]) -> Vec<String> {
    let mut e = Engine::default();
    let files: Vec<(String, String)> = files
        .iter()
        .map(|(n, s)| (n.to_string(), s.to_string()))
        .collect();
    e.load(&files).1.iter().map(|e| e.to_string()).collect()
}

fn join(src: &str) -> Vec<String> {
    let mut e = engine(src);
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.errors().is_empty(), "runtime errors: {:?}", h.errors());
    h.take()
}

fn sends(lines: &[&str]) -> Vec<String> {
    lines.iter().map(|l| format!("send Steve {l}")).collect()
}

// ---------- variables: ----------

#[test]
fn variables_set_defaults_once() {
    let src = r#"variables:
    {motd} = "hello"
    {count} = 5
on load:
    send "%{motd}% %{count}%" to console
"#;
    let mut e = engine(src);
    let mut h = Mock::default();
    e.fire(EventKind::Load, EventCtx::console(), &mut h);
    assert_eq!(h.take(), ["send console hello 5"]);
    assert!(h.errors().is_empty());
}

#[test]
fn variables_keep_existing_values() {
    let src = "variables:\n    {motd} = \"default\"\non load:\n    send {motd} to console\n";
    let mut e = engine(src);
    e.vars.set("motd".into(), Value::Text("saved".into()));
    let mut h = Mock::default();
    e.fire(EventKind::Load, EventCtx::console(), &mut h);
    assert_eq!(h.take(), ["send console saved"]);
}

#[test]
fn variables_player_defaults_on_join() {
    let src = r#"variables:
    {coins::%player%} = 100
on join:
    send "coins: %{coins::%player%}%"
"#;
    let mut e = engine(src);
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.take(), sends(&["coins: 100"]));
    // A value changed by a script stays when the player joins again.
    e.vars.set("coins::steve".into(), Value::Num(7.0));
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.take(), sends(&["coins: 7"]));
}

#[test]
fn variables_player_defaults_for_players_online_at_load() {
    let src = "variables:\n    {coins::%player%} = 3\non load:\n    send \"x\" to console\n";
    let mut e = engine(src);
    let mut h = Mock::with_steve();
    e.fire(EventKind::Load, EventCtx::console(), &mut h);
    assert_eq!(e.vars.get("coins::steve"), Value::Num(3.0));
}

#[test]
fn variables_defaults_apply_before_a_command_runs() {
    let src = "variables:\n    {n} = 2\ncommand /n:\n    trigger:\n        send \"%{n}%\"\n";
    let mut e = engine(src);
    let mut h = Mock::with_steve();
    e.run_command("n", Recipient::Player(steve()), "", &mut h);
    assert_eq!(h.take(), sends(&["2"]));
}

#[test]
fn variables_value_can_be_an_expression() {
    let src = "variables:\n    {total} = 2 + 3\non load:\n    send \"%{total}%\" to console\n";
    let mut e = engine(src);
    let mut h = Mock::default();
    e.fire(EventKind::Load, EventCtx::console(), &mut h);
    assert_eq!(h.take(), ["send console 5"]);
}

#[test]
fn variables_errors() {
    let e = errors_of("variables:\n    coins = 0\n");
    assert_eq!(e[0].line, 2);
    assert!(e[0].msg.contains("{name} = value"), "{}", e[0]);
    let e = errors_of("variables:\n    {_x} = 0\n");
    assert!(e[0].msg.contains("global"), "{}", e[0]);
    let e = errors_of("variables:\n    {x} = nonsense words\n");
    assert_eq!(e[0].line, 2);
    let e = errors_of("variables:\n    {x} =\n");
    assert!(e[0].msg.contains("{name} = value"), "{}", e[0]);
}

#[test]
fn variables_use_options() {
    let src =
        "options:\n    start: 9\nvariables:\n    {n} = {@start}\non load:\n    send \"%{n}%\" to console\n";
    let mut e = engine(src);
    let mut h = Mock::default();
    e.fire(EventKind::Load, EventCtx::console(), &mut h);
    assert_eq!(h.take(), ["send console 9"]);
}

// ---------- aliases: ----------

#[test]
fn aliases_replace_names() {
    let src = r#"aliases:
    magic wand = stick
    gem = diamond
on join:
    if player has magic wand:
        send "wand"
    if player has gem:
        send "gem"
    send "a magic wand is a gem"
"#;
    let mut e = engine(src);
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    // The text in quotes stays as it is; the item names were replaced.
    assert!(h.errors().is_empty(), "{:?}", h.errors());
    assert!(h.take().contains(&"send Steve a magic wand is a gem".to_string()));
}

#[test]
fn alias_errors() {
    let e = errors_of("aliases:\n    gem\n");
    assert!(e[0].msg.contains("aliases look like"), "{}", e[0]);
    let e = errors_of("aliases:\n    gem = diamond\n    GEM = emerald\n");
    assert!(e[0].msg.contains("twice"), "{}", e[0]);
}

// ---------- inline conditions ----------

#[test]
fn inline_if_runs_the_effect_only_when_true() {
    let src = r#"on join:
    send "a" if {x} is not set
    send "b" if {x} is set
    do send "c" to player if 1 = 1
    send "d" unless 1 = 1
    send "e" unless 1 = 2
"#;
    assert_eq!(join(src), sends(&["a", "c", "e"]));
}

#[test]
fn inline_if_ignores_if_inside_text_and_variables() {
    let src =
        "on join:\n    send \"go if you can\"\n    set {a if b} to 1 if 1 = 1\n    send \"%{a if b}%\"\n";
    assert_eq!(join(src), sends(&["go if you can", "1"]));
}

#[test]
fn inline_if_works_with_stop_and_loops() {
    let src = r#"on join:
    loop 5 times:
        continue if loop-value is 2
        exit loop if loop-value is 4
        send "%loop-value%"
    stop if 1 = 1
    send "never"
"#;
    assert_eq!(join(src), sends(&["1", "3"]));
}

#[test]
fn inline_if_with_conditions_joined() {
    let src = "on join:\n    send \"ok\" if 1 = 1 and 2 = 2\n    send \"no\" if 1 = 2 or 2 = 3\n";
    assert_eq!(join(src), sends(&["ok"]));
}

#[test]
fn inline_if_bad_condition_reports_line() {
    let e = errors_of("on join:\n    send \"a\"\n    send \"b\" if the moon is blue\n");
    assert_eq!(e[0].line, 3);
    assert!(e[0].msg.contains("condition"), "{}", e[0]);
}

#[test]
fn inline_if_bad_effect_reports_line() {
    let e = errors_of("on join:\n    fly away if 1 = 1\n");
    assert_eq!(e[0].line, 2);
    assert!(e[0].msg.contains("effect"), "{}", e[0]);
}

// ---------- else if ----------

#[test]
fn else_if_chains_and_nesting() {
    let src = r#"on join:
    if {n} is 1:
        send "one"
    else if {n} is 2:
        send "two"
    else if {n} is 3:
        if {m} is 1:
            send "three-m"
        else if {m} is 2:
            send "three-2"
        else:
            send "three-else"
    else:
        send "other"
"#;
    let mut e = engine(src);
    for (n, m, want) in [
        (1.0, 0.0, "one"),
        (2.0, 0.0, "two"),
        (3.0, 1.0, "three-m"),
        (3.0, 2.0, "three-2"),
        (3.0, 9.0, "three-else"),
        (7.0, 0.0, "other"),
    ] {
        e.vars.set("n".into(), Value::Num(n));
        e.vars.set("m".into(), Value::Num(m));
        let mut h = Mock::with_steve();
        e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
        assert_eq!(h.take(), sends(&[want]), "n={n} m={m}");
    }
}

#[test]
fn else_if_without_if_is_an_error() {
    let e = errors_of("on join:\n    else if 1 = 1:\n        send \"x\"\n");
    assert_eq!(e[0].line, 2);
}

// ---------- parse if ----------

#[test]
fn parse_if_keeps_or_drops_the_section() {
    let src = r#"options:
    mode: debug
on join:
    parse if "{@mode}" is "debug":
        send "debug on"
    parse if "{@mode}" is not "debug":
        send "debug off"
    parse if 2 > 1 and not (1 = 2):
        send "math"
    parse if false or 3 >= 4:
        send "never"
    send "end"
"#;
    assert_eq!(join(src), sends(&["debug on", "math", "end"]));
}

#[test]
fn parse_if_needs_fixed_values() {
    let e = errors_of("on join:\n    parse if {x} is 1:\n        send \"a\"\n");
    assert_eq!(e[0].line, 2);
    assert!(e[0].msg.contains("fixed values"), "{}", e[0]);
}

#[test]
fn parse_if_dropped_section_is_not_checked() {
    // Skipped code is never compiled, like in Skript.
    let src = "on join:\n    parse if false:\n        this is nonsense\n    send \"ok\"\n";
    assert_eq!(join(src), sends(&["ok"]));
}

// ---------- catch runtime errors ----------

#[test]
fn catch_takes_an_error_and_goes_on() {
    let src = r#"on join:
    send "before"
    catch runtime errors:
        send "in"
        send "%1 / 0%"
        send "never"
    send "after"
"#;
    assert_eq!(join(src), sends(&["before", "in", "after"]));
}

#[test]
fn catch_gives_the_error_message() {
    let src = r#"on join:
    catch runtime errors:
        send "%1 / 0%"
    send "caught: %last runtime error%"
    catch runtime errors:
        send "fine"
    send "set: %{_runtime error}%"
    if last caught error is set:
        send "still set"
"#;
    assert_eq!(
        join(src),
        sends(&["caught: division by zero", "fine", "set: <none>"])
    );
}

#[test]
fn catch_without_error_runs_everything() {
    let src = "on join:\n    catch runtime errors:\n        send \"a\"\n        send \"b\"\n    send \"c\"\n";
    assert_eq!(join(src), sends(&["a", "b", "c"]));
}

#[test]
fn catch_inside_a_loop_and_leaving_it() {
    let src = r#"on join:
    loop 4 times:
        catch runtime errors:
            continue if loop-value is 1
            exit loop if loop-value is 4
            send "%10 / (loop-value - 2)%"
            send "ok %loop-value%"
        send "end %loop-value%"
    send "done"
"#;
    // 1: continue (skips the rest of the loop body). 2: division by zero is
    // caught, then 'end 2'. 3: 10 and ok 3, then 'end 3'. 4: exit loop.
    assert_eq!(join(src), sends(&["end 2", "10", "ok 3", "end 3", "done"]));
}

#[test]
fn catch_error_in_a_called_function() {
    let src = r#"function boom(x: number):
    send "boom %{_x}%"
    send "%1 / 0%"
    send "never"

on join:
    catch runtime errors:
        boom(5)
        send "never"
    send "after: %last runtime error%"
"#;
    assert_eq!(join(src), sends(&["boom 5", "after: division by zero"]));
}

#[test]
fn catch_error_in_a_function_used_as_a_value() {
    let src = r#"function half(x: number) :: number:
    return {_x} / 0

on join:
    catch runtime errors:
        send "%half(4)%"
        send "never"
    send "after: %last runtime error%"
    send "again %half(1)%"
    send "end"
"#;
    let mut e = engine(src);
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    // The second call is outside any section: the run stops with a log.
    assert_eq!(h.take(), sends(&["after: division by zero"]));
    assert_eq!(h.errors().len(), 1);
    assert!(h.errors()[0].contains("division by zero"));
}

#[test]
fn catch_inside_a_function() {
    let src = r#"function safe() :: text:
    catch runtime errors:
        send "%1 / 0%"
        return "not caught"
    return "caught %last runtime error%"

on join:
    send "%safe()%"
"#;
    assert_eq!(join(src), sends(&["caught division by zero"]));
}

#[test]
fn catch_return_inside_a_function_closes_the_section() {
    let src = r#"function f() :: number:
    catch runtime errors:
        return 7
    return 0

on join:
    send "%f()%"
    send "%1 / 0%"
    send "never"
"#;
    let mut e = engine(src);
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    // The `return` left the section; the later error is not swallowed.
    assert_eq!(h.take(), sends(&["7"]));
    assert_eq!(h.errors().len(), 1);
}

#[test]
fn catch_nested_sections() {
    let src = r#"on join:
    catch runtime errors:
        catch runtime errors:
            send "%1 / 0%"
        send "inner: %last runtime error%"
        send "%2 / 0%"
        send "never"
    send "outer: %last runtime error%"
"#;
    assert_eq!(
        join(src),
        sends(&["inner: division by zero", "outer: division by zero"])
    );
}

#[test]
fn catch_survives_a_wait() {
    let src = r#"on join:
    catch runtime errors:
        send "a"
        wait 1 second
        send "%1 / 0%"
    send "b"
"#;
    let mut e = engine(src);
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.take(), sends(&["a"]));
    let token = h.waits[0].1;
    e.resume(token, &mut h);
    assert_eq!(h.take(), sends(&["b"]));
    assert!(h.errors().is_empty());
}

#[test]
fn errors_outside_catch_still_log() {
    let src = "on join:\n    catch runtime errors:\n        send \"ok\"\n    send \"%1 / 0%\"\n";
    let mut e = engine(src);
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.errors().len(), 1);
    assert!(
        h.errors()[0].starts_with("test.carve:4: division by zero"),
        "{:?}",
        h.errors()
    );
}

#[test]
fn catch_runaway_script_is_not_caught() {
    let src = "on join:\n    catch runtime errors:\n        while true is true:\n            set {x} to 1\n    send \"after\"\n";
    let mut e = engine(src);
    e.max_instructions = 500;
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.take().is_empty());
    assert_eq!(h.errors().len(), 1);
}

// ---------- local function ----------

#[test]
fn local_functions_can_share_names_across_files() {
    let a = "local function label() :: text:\n    return \"one\"\n\non join:\n    send \"%label()%\"\n";
    let b = "local function label() :: text:\n    return \"two\"\n\non quit:\n    send \"%label()%\"\n";
    let mut e = engine_files(&[("a.carve", a), ("b.carve", b)]);
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    e.fire(EventKind::Quit, EventCtx::player(steve()), &mut h);
    assert_eq!(h.take(), sends(&["one", "two"]));
    assert!(h.errors().is_empty());
}

#[test]
fn local_function_is_hidden_from_other_files() {
    let a = "local function secret() :: text:\n    return \"s\"\n";
    let b = "on join:\n    send \"%secret()%\"\n";
    let errs = load_errors(&[("a.carve", a), ("b.carve", b)]);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(errs[0].starts_with("b.carve:2"), "{errs:?}");
    assert!(errs[0].contains("unknown function"), "{errs:?}");
}

#[test]
fn local_function_wins_over_a_global_in_its_own_file() {
    let a = "function who() :: text:\n    return \"global\"\n\non quit:\n    send \"%who()%\"\n";
    let b = "local function who() :: text:\n    return \"local\"\n\non join:\n    send \"%who()%\"\n";
    let mut e = engine_files(&[("a.carve", a), ("b.carve", b)]);
    let mut h = Mock::with_steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    e.fire(EventKind::Quit, EventCtx::player(steve()), &mut h);
    assert_eq!(h.take(), sends(&["local", "global"]));
}

#[test]
fn local_function_clashes() {
    let errs = errors_of("local function f():\n    stop\nlocal function f():\n    stop\n");
    assert_eq!(errs.len(), 1);
    assert!(errs[0].msg.contains("defined twice"), "{}", errs[0]);
    let errs = errors_of("function f():\n    stop\nlocal function f():\n    stop\n");
    assert!(errs[0].msg.contains("defined twice"), "{}", errs[0]);
    // Two globals still clash across files.
    let errs = load_errors(&[
        ("a.carve", "function f():\n    stop\n"),
        ("b.carve", "function f():\n    stop\n"),
    ]);
    assert_eq!(errs.len(), 1, "{errs:?}");
}

#[test]
fn local_function_can_call_other_local_functions_and_take_arguments() {
    let src = r#"local function twice(n: number) :: number:
    return {_n} * 2

local function quad(n: number) :: number:
    return twice(twice({_n}))

on join:
    send "%quad(3)%"
"#;
    assert_eq!(join(src), sends(&["12"]));
}

// ---------- using ----------

#[test]
fn using_lines_are_accepted() {
    let src = "using something experimental\non join:\n    send \"ok\"\n";
    assert_eq!(join(src), sends(&["ok"]));
}

// ---------- everything together ----------

#[test]
fn structures_work_together() {
    let src = r#"options:
    start: 10
aliases:
    bonus = 5
variables:
    {points::%player%} = {@start}
function award(p: player, n: number):
    add {_n} to {points::%{_p}%}

on join:
    catch runtime errors:
        award(player, bonus)
        award(player, 1 / 0)
    send "points %{points::%player%}%" if {points::%player%} > 0
    send "error: %last runtime error%" unless {_runtime error} is not set
"#;
    assert_eq!(join(src), sends(&["points 15", "error: division by zero"]));
}
