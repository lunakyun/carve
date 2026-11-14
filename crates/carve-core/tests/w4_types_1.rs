use carve_core::compile::{EventKind, check_registry};
use carve_core::types_1::{ENUMS, EnumVal, T_COLOR, parse_text};
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.calls.push(text.to_string());
    }
    fn broadcast(&mut self, _t: &str) {}
    fn send_title(&mut self, _p: &PlayerRef, _t: &str, _s: Option<&str>) {}
    fn send_action_bar(&mut self, _p: &PlayerRef, _t: &str) {}
    fn heal(&mut self, _p: &PlayerRef) {}
    fn set_health(&mut self, _p: &PlayerRef, _h: f64) {}
    fn kill(&mut self, _p: &PlayerRef) {}
    fn teleport(&mut self, _p: &PlayerRef, _x: f64, _y: f64, _z: f64) {}
    fn execute_command(&mut self, _c: &str) {}
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
    }
    fn find_player(&mut self, _n: &str) -> Option<PlayerRef> {
        None
    }
    fn has_permission(&mut self, _p: &PlayerRef, _n: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _t: u64, _k: u64) {}
    fn now(&mut self) -> u64 {
        0
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

fn source(lines: &[&str]) -> String {
    let mut src = String::from("on join:\n");
    for l in lines {
        src.push_str("    ");
        src.push_str(l);
        src.push('\n');
    }
    src
}

fn run(lines: &[&str]) -> Vec<String> {
    run_src(&source(lines)).0
}

fn run_src(src: &str) -> (Vec<String>, Engine) {
    let mut h = Mock::default();
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.to_string())]);
    assert!(
        errs.is_empty(),
        "errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let bad: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(bad.is_empty(), "runtime errors: {bad:?}");
    (h.calls, e)
}

fn load_errors(lines: &[&str]) -> Vec<String> {
    let mut e = Engine::default();
    e.load(&[("t.carve".into(), source(lines))])
        .1
        .iter()
        .map(|e| e.to_string())
        .collect()
}

fn one(expr: &str) -> String {
    run(&[&format!("send \"%{expr}%\"")])
        .into_iter()
        .next()
        .unwrap_or_default()
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn table_is_sound() {
    assert!(ENUMS.len() < 200);
    for d in ENUMS {
        assert!(!d.values.is_empty(), "{} has no values", d.name);
        let mut seen = std::collections::HashSet::new();
        for v in d.values {
            assert!(seen.insert(*v), "{}: duplicate value {v}", d.name);
            assert_eq!(
                *v,
                v.to_ascii_lowercase().replace([' ', '-'], "_"),
                "{}: value {v} is not a plain id",
                d.name
            );
        }
    }
    let mut names = std::collections::HashSet::new();
    for d in ENUMS {
        assert!(names.insert(d.name), "duplicate type {}", d.name);
    }
}

#[test]
fn bare_words_print_and_compare() {
    assert_eq!(one("shift left"), "shift left");
    assert_eq!(one("full moon"), "full moon");
    assert_eq!(one("waxing crescent"), "waxing crescent");
    assert_eq!(one("segmented 10"), "segmented 10");
    assert_eq!(
        run(&["if shift left is \"shift_left\":", "    send \"yes\""]),
        vec!["yes"]
    );
    assert_eq!(
        run(&["if full moon is not new moon:", "    send \"differs\""]),
        vec!["differs"]
    );
    assert_eq!(
        run(&["if full moon is full moon:", "    send \"same\""]),
        vec!["same"]
    );
}

#[test]
fn parsed_as_makes_typed_values() {
    assert_eq!(one("\"left\" parsed as click type"), "left");
    assert_eq!(one("\"Shift Right\" parsed as click type"), "shift right");
    assert_eq!(
        run(&[
            "set {_c} to \"nope\" parsed as click type",
            "if {_c} is not set:",
            "    send \"none\""
        ]),
        vec!["none"]
    );
    assert_eq!(one("\"warm\" parsed as chicken variant"), "warm");
    assert_eq!(one("\"nether\" parsed as world environment"), "nether");
    assert_eq!(one("\"hard\" parsed as difficulty"), "hard");
    assert_eq!(one("\"survival\" parsed as spawn reason"), "<none>");
}

#[test]
fn type_names_work_in_functions() {
    let src = "function describe(c: click type) :: text:\n    return \"click %{_c}%\"\n\
function moon(m: moon phase) :: moon phase:\n    return {_m}\n\
on join:\n    send describe(left)\n    send describe(\"right\")\n    send \"%moon(new moon)%\"\n";
    assert_eq!(run_src(src).0, vec!["click left", "click right", "new moon"]);
}

#[test]
fn wrong_type_is_a_load_error() {
    let src = "function f(c: click type):\n    send \"%{_c}%\"\non join:\n    f(5)\n";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(!errs.is_empty());
    assert!(errs[0].to_string().contains("click type"), "{}", errs[0]);
}

#[test]
fn loose_type_names_are_accepted() {
    for t in [
        "entity",
        "living entity",
        "projectile",
        "vehicle",
        "display entity",
        "nameable",
        "metadata holder",
        "audience",
        "any named thing",
        "any valued thing",
        "slot",
        "inventory",
    ] {
        let src = format!("function f(x: {t}):\n    send \"ok\"\non join:\n    f(player)\n");
        assert_eq!(run_src(&src).0, vec!["ok"], "type {t}");
    }
}

#[test]
fn plural_type_is_a_list() {
    let src = "function f(c: click types):\n    send \"%size of {_c::*}%\"\non join:\n    set {_l::*} to left, right\n    f({_l::*})\n";
    assert_eq!(run_src(src).0, vec!["2"]);
}

#[test]
fn colors() {
    assert_eq!(one("rgb(255, 128, 0)"), "#ff8000");
    assert_eq!(one("red"), "#b02e26");
    assert_eq!(one("light blue"), "#3ab3da");
    assert_eq!(
        run(&["set {_h} to \"#00ff00\" parsed as color", "send \"%{_h}%\""]),
        vec!["#00ff00"]
    );
    assert_eq!(one("\"lime\" parsed as color"), "#80c71f");
    assert_eq!(
        run(&["send \"%color from hex \"\"#102030\"\"%\""]),
        vec!["#102030"]
    );
    assert_eq!(one("hex of rgb(1, 2, 3)"), "#010203");
    assert_eq!(one("rgb(300, -4, 0)"), "#ff0000");
    assert_eq!(
        run(&["if rgb(1, 2, 3) is rgb(1, 2, 3):", "    send \"eq\""]),
        vec!["eq"]
    );
    assert_eq!(one("red value of rgb(255, 128, 0)"), "255");
    assert_eq!(one("green value of rgb(255, 128, 0)"), "128");
}

#[test]
fn quaternion_queue_chunk_border_timeperiod() {
    assert_eq!(one("quaternion(1, 0, 0, 0)"), "w: 1, x: 0, y: 0, z: 0");
    assert_eq!(one("identity quaternion"), "w: 1, x: 0, y: 0, z: 0");
    assert_eq!(one("timeperiod(0, 12000)"), "6:00 to 18:00");
    assert_eq!(one("time period(18000, 6000)"), "0:00 to 12:00");
    assert_eq!(one("queue(1, 2, 3)"), "queue of 1, 2, 3");
    assert_eq!(one("queue()"), "queue of nothing");
    assert_eq!(one("chunk(2, -3)"), "chunk 2, -3");
    assert_eq!(one("chunk of location(17, 64, -1)"), "chunk 1, -1");
    assert_eq!(
        one("chunk of location(17, 64, -1, world \"w\")"),
        "chunk 1, -1 in 'w'"
    );
    assert_eq!(one("world border of world \"w\""), "world border of 'w'");
}

#[test]
fn values_survive_variables_json() {
    let src = source(&[
        "set {c} to shift left",
        "set {col} to rgb(1, 2, 3)",
        "set {q} to quaternion(1, 2, 3, 4)",
        "set {p} to timeperiod(0, 6000)",
        "set {k} to chunk(5, 6, world \"w\")",
        "set {b} to world border of world \"w\"",
        "set {m} to queue(1, 2)",
    ]);
    let (_, mut e) = run_src(&src);
    let json = e.save_now().unwrap();
    let back = Variables::from_json(&json).unwrap();
    assert_eq!(back.get("c"), parse_text(5, "shift_left"));
    assert!(matches!(back.get("c"), Value::Enum(EnumVal { ty: 5, .. })));
    assert_eq!(back.get("col"), Value::Color { r: 1, g: 2, b: 3 });
    assert_eq!(
        back.get("q"),
        Value::Quaternion {
            w: 1.0,
            x: 2.0,
            y: 3.0,
            z: 4.0
        }
    );
    assert_eq!(back.get("p"), Value::Timeperiod { start: 0, end: 6000 });
    assert_eq!(
        back.get("k"),
        Value::Chunk {
            x: 5,
            z: 6,
            world: Some("w".into())
        }
    );
    assert_eq!(back.get("b"), Value::Border("w".into()));
    assert_eq!(back.get("m"), Value::None);
    let _ = T_COLOR;
}

#[test]
fn variable_holds_and_prints_value() {
    assert_eq!(
        run(&["set {_t} to bite", "send \"state: %{_t}%\""]),
        vec!["state: bite"]
    );
}

#[test]
fn errors_are_reported_per_line() {
    let errs = load_errors(&["send rgb(1, 2)", "send quaternion(1)", "send \"ok\""]);
    assert_eq!(errs.len(), 2, "{errs:?}");
    assert!(errs[0].contains("rgb"), "{errs:?}");
    assert!(errs[1].contains("quaternion"), "{errs:?}");
}

#[test]
fn existing_words_keep_their_meaning() {
    // `survival` stays plain text, `diamond` stays an item.
    assert_eq!(one("diamond"), "diamond");
    assert_eq!(one("survival"), "survival");
}

#[test]
fn moon_phase_in_variable() {
    assert_eq!(one("\"waxing crescent\" parsed as moon phase"), "waxing crescent");
    assert_eq!(
        run(&[
            "set {_m} to \"waxing crescent\" parsed as moon phase",
            "send \"%{_m}%\""
        ]),
        vec!["waxing crescent".to_string()]
    );
}
