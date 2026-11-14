use std::collections::HashMap;

use carve_core::compile::{EventKind, check_registry};
use carve_core::w2_expressions_7::Query;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
    online: HashMap<String, String>,
}

impl Mock {
    fn new() -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m
    }
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
        self.online.get(uuid).cloned()
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
    fn player_location(&mut self, _p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((10.0, 64.0, -5.0, "world".into()))
    }
    fn w2e7_query(&mut self, q: Query<'_>) -> Option<Value> {
        Some(match q {
            Query::Tps => Value::Num(19.5),
            Query::Mspt => Value::Num(32.0),
            Query::ServerVersion => Value::Text("0.1.0".into()),
            Query::MinecraftVersion => Value::Text("1.21.9".into()),
            Query::ServerViewDistance => Value::Num(10.0),
            Query::ClientViewDistance(_) => Value::Num(8.0),
            Query::PlayTicks(_) => Value::Num(72000.0),
            Query::LivedTicks(_) => Value::Num(1200.0),
            Query::Yaw(_) => Value::Num(90.0),
            Query::Pitch(_) => Value::Num(0.0),
            Query::Worlds => Value::List(vec![Value::World("world".into()), Value::World("nether".into())]),
            Query::Environment(n) => Value::Text(
                if n == "nether" {
                    "minecraft:the_nether"
                } else {
                    "minecraft:overworld"
                }
                .into(),
            ),
        })
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn run(lines: &[&str]) -> Vec<String> {
    let mut src = String::from("on join:\n");
    for l in lines {
        src.push_str("    ");
        src.push_str(l);
        src.push('\n');
    }
    let mut h = Mock::new();
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    assert!(
        errs.is_empty(),
        "errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let bad: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(bad.is_empty(), "runtime errors: {bad:?}");
    h.calls
}

fn load_errors(lines: &[&str]) -> usize {
    let mut src = String::from("on join:\n");
    for l in lines {
        src.push_str("    ");
        src.push_str(l);
        src.push('\n');
    }
    let mut e = Engine::default();
    e.load(&[("t.carve".into(), src)]).1.len()
}

fn one(expr: &str) -> String {
    let out = run(&[&format!("send \"%{expr}%\"")]);
    out.into_iter().next().unwrap_or_default()
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn vector_create_and_print() {
    assert_eq!(one("vector(1, 2, 3)"), "x: 1, y: 2, z: 3");
    assert_eq!(one("vector from 1, 2, 3"), "x: 1, y: 2, z: 3");
    assert_eq!(one("a vector from 1, -2 and 3.5"), "x: 1, y: -2, z: 3.5");
    assert_eq!(one("vector 4, 5, 6"), "x: 4, y: 5, z: 6");
    assert_eq!(one("vector(1 + 1, 2 * 2, 0)"), "x: 2, y: 4, z: 0");
}

#[test]
fn vector_components() {
    let out = run(&[
        "set {v} to vector(1, 2, 3)",
        r#"send "%x of {v}% %y of {v}% %z of {v}%""#,
        r#"send "%{v}'s x% %x component of {v}% %x-coordinate of {v}%""#,
        "set {w} to vector(x of {v} * 10, y of {v}, z of {v})",
        r#"send "%{w}%""#,
    ]);
    assert_eq!(out, vec!["1 2 3", "1 1 1", "x: 10, y: 2, z: 3"]);
}

#[test]
fn vector_arithmetic() {
    assert_eq!(one("vector(1, 2, 3) + vector(1, 1, 1)"), "x: 2, y: 3, z: 4");
    assert_eq!(one("vector(1, 2, 3) - vector(1, 1, 1)"), "x: 0, y: 1, z: 2");
    assert_eq!(one("vector(1, 2, 3) * 2"), "x: 2, y: 4, z: 6");
    assert_eq!(one("3 * vector(1, 2, 3)"), "x: 3, y: 6, z: 9");
    assert_eq!(one("vector(2, 4, 6) / 2"), "x: 1, y: 2, z: 3");
    assert_eq!(one("vector(1, 2, 3) * vector(2, 2, 2)"), "x: 2, y: 4, z: 6");
    assert_eq!(one("-vector(1, -2, 3)"), "x: -1, y: 2, z: -3");
}

#[test]
fn vector_arithmetic_on_variables() {
    let out = run(&[
        "set {a} to vector(1, 0, 0)",
        "set {b} to vector(0, 1, 0)",
        r#"send "%{a} + {b}%""#,
        r#"send "%{a} * 5%""#,
        r#"send "%{a} - {b} * 2%""#,
    ]);
    assert_eq!(out, vec!["x: 1, y: 1, z: 0", "x: 5, y: 0, z: 0", "x: 1, y: -2, z: 0"]);
}

#[test]
fn vector_division_by_zero_is_an_error() {
    let mut src = String::from("on join:\n    set {a} to vector(1, 1, 1)\n    set {b} to {a} / 0\n");
    src.push_str("    send \"after\"\n");
    let mut h = Mock::new();
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    assert!(errs.is_empty());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.logs.iter().any(|(l, m)| *l == LogLevel::Error && m.contains("division by zero")));
    assert!(!h.calls.iter().any(|c| c == "after"));
}

#[test]
fn vector_length_and_normalize() {
    assert_eq!(one("length of vector(3, 4, 0)"), "5");
    assert_eq!(one("vector length of vector(0, 0, 2)"), "2");
    assert_eq!(one("squared length of vector(3, 4, 0)"), "25");
    assert_eq!(one("squared vector length of vector(1, 2, 2)"), "9");
    assert_eq!(one("normalized vector(0, 0, 9)"), "x: 0, y: 0, z: 1");
    assert_eq!(one("vector(0, 7, 0) normalized"), "x: 0, y: 1, z: 0");
    assert_eq!(one("normalized vector(0, 0, 0)"), "x: 0, y: 0, z: 0");
    let out = run(&["set {v} to vector(3, 4, 0)", r#"send "%length of {v}%""#]);
    assert_eq!(out, vec!["5"]);
}

#[test]
fn text_length_still_works() {
    let out = run(&[
        r#"set {t} to "hello""#,
        r#"send "%length of {t}%""#,
        r#"send "%length of "abc"%""#,
    ]);
    assert_eq!(out, vec!["5", "3"]);
}

#[test]
fn dot_cross_angle_projection() {
    assert_eq!(one("vector(1, 2, 3) dot vector(4, 5, 6)"), "32");
    assert_eq!(one("vector(1, 0, 0) cross vector(0, 1, 0)"), "x: 0, y: 0, z: 1");
    assert_eq!(one("angle between vector(1, 0, 0) and vector(0, 1, 0)"), "90");
    assert_eq!(one("angle between vector(1, 0, 0) and vector(2, 0, 0)"), "0");
    assert_eq!(
        one("vector(3, 4, 0) projected onto vector(1, 0, 0)"),
        "x: 3, y: 0, z: 0"
    );
    assert_eq!(
        one("projection of vector(3, 4, 0) onto vector(0, 2, 0)"),
        "x: 0, y: 4, z: 0"
    );
    let out = run(&[
        "set {a} to vector(1, 0, 0)",
        "set {b} to vector(0, 1, 0)",
        r#"send "%{a} dot {b}% %{a} cross {b}%""#,
    ]);
    assert_eq!(out, vec!["0 x: 0, y: 0, z: 1"]);
}

#[test]
fn yaw_and_pitch_of_vectors() {
    assert_eq!(one("yaw of vector(0, 0, 1)"), "0");
    assert_eq!(one("yaw of vector(-1, 0, 0)"), "90");
    assert_eq!(one("pitch of vector(0, -1, 0)"), "90");
    assert_eq!(one("pitch of vector(0, 0, 5)"), "0");
    assert_eq!(one("vector from yaw 0 and pitch 0"), "x: 0, y: 0, z: 1");
    assert_eq!(one("vector from yaw 90 and pitch 0"), "x: -1, y: 0, z: 0");
    let out = run(&[r#"send "%vector from yaw 0 and pitch 90%""#]);
    assert!(out[0].starts_with("x: 0, y: -1, z: 0"), "{out:?}");
}

#[test]
fn yaw_and_pitch_of_player() {
    assert_eq!(one("yaw of player"), "90");
    assert_eq!(one("pitch of player"), "0");
    assert_eq!(one("vector from direction of player"), "x: -1, y: 0, z: 0");
}

#[test]
fn direction_words() {
    assert_eq!(one("vector from direction north"), "x: 0, y: 0, z: -1");
    assert_eq!(one("vector from direction east"), "x: 1, y: 0, z: 0");
    assert_eq!(one("vector from direction up"), "x: 0, y: 1, z: 0");
    assert_eq!(load_errors(&[r#"set {v} to vector from direction sideways"#]), 1);
}

#[test]
fn locations_and_vectors() {
    assert_eq!(one("vector from location(1, 2, 3)"), "x: 1, y: 2, z: 3");
    assert_eq!(one("vector of player"), "x: 10, y: 64, z: -5");
    assert_eq!(
        one("vector from location(1, 2, 3) to location(4, 6, 3)"),
        "x: 3, y: 4, z: 0"
    );
    assert_eq!(
        one("vector between location(0, 0, 0) and location(1, 1, 1)"),
        "x: 1, y: 1, z: 1"
    );
    assert_eq!(
        one("location(1, 2, 3) offset by vector(1, 1, 1)"),
        "x: 2, y: 3, z: 4"
    );
    assert_eq!(
        one("location(1, 2, 3, world \"nether\") + vector(1, 0, 0)"),
        "x: 2, y: 2, z: 3 in 'nether'"
    );
    assert_eq!(one("location(1, 2, 3) - vector(1, 2, 3)"), "x: 0, y: 0, z: 0");
    assert_eq!(
        one("location from vector vector(1, 2, 3) in world \"nether\""),
        "x: 1, y: 2, z: 3 in 'nether'"
    );
    assert_eq!(one("vector(1, 2, 3) to location"), "x: 1, y: 2, z: 3");
    assert_eq!(
        one("vector(1, 2, 3) to location in world \"world\""),
        "x: 1, y: 2, z: 3 in 'world'"
    );
    let out = run(&["set {l} to location(0, 0, 0)", "set {v} to vector(1, 2, 3)", r#"send "%{l} + {v}%""#]);
    assert_eq!(out, vec!["x: 1, y: 2, z: 3"]);
}

#[test]
fn shapes() {
    assert_eq!(
        one("spherical vector radius 2, yaw 0, pitch 0"),
        "x: 0, y: 0, z: 2"
    );
    assert_eq!(
        one("cylindrical vector radius 3, yaw 0, height 5"),
        "x: 0, y: 5, z: 3"
    );
    // The length of a spherical vector is its radius.
    assert_eq!(
        one("length of spherical vector radius 4, yaw 33, pitch 21"),
        "4"
    );
    assert_eq!(one("x of cylindrical vector with radius 1 and yaw 90 and height 0"), "-1");
}

#[test]
fn random_vector_is_unit_length() {
    let out = run(&[
        "loop 20 times:",
        "    set {v} to random vector",
        "    if abs(length of {v} - 1) > 0.0001:",
        "        send \"bad %{v}%\"",
        "send \"done\"",
    ]);
    assert_eq!(out, vec!["done"]);
    let out = run(&["set {a} to random vector", "set {b} to a random vector", r#"if {a} is {b}:"#, r#"    send "same""#]);
    assert!(out.is_empty());
}

#[test]
fn vector_variables_survive_save() {
    let mut vars = Variables::default();
    vars.set_str("v", Value::Vector { x: 1.0, y: 2.5, z: -3.0 });
    let json = vars.to_json();
    let back = Variables::from_json(&json).unwrap();
    assert_eq!(back.get("v"), Value::Vector { x: 1.0, y: 2.5, z: -3.0 });
}

#[test]
fn vector_equality() {
    let out = run(&[
        "set {a} to vector(1, 2, 3)",
        "if {a} is vector(1, 2, 3):",
        r#"    send "equal""#,
        "if {a} is not vector(3, 2, 1):",
        r#"    send "different""#,
    ]);
    assert_eq!(out, vec!["equal", "different"]);
}

#[test]
fn vector_function_parameters() {
    let src = "function len2(v: vector) :: number:\n    return squared length of {_v}\n\non join:\n    send \"%len2(vector(1, 2, 2))%\"\n";
    let mut h = Mock::new();
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty(), "{errs:?}");
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.calls, vec!["9"]);
}

#[test]
fn bad_vector_math_is_a_load_error() {
    assert_eq!(load_errors(&["set {v} to vector(1, 2, 3) + 5"]), 1);
    assert_eq!(load_errors(&["set {v} to 5 / vector(1, 2, 3)"]), 1);
    assert_eq!(load_errors(&["set {v} to vector(1, 2)"]), 1);
    assert_eq!(load_errors(&["set {v} to vector(1, 2, 3, 4)"]), 1);
}

#[test]
fn server_facts() {
    assert_eq!(one("tps"), "19.5");
    assert_eq!(one("server tps"), "19.5");
    assert_eq!(one("ticks per second"), "19.5");
    assert_eq!(one("mspt"), "32");
    assert_eq!(one("server version"), "0.1.0");
    assert_eq!(one("pumpkin version"), "0.1.0");
    assert_eq!(one("server version string"), "0.1.0");
    assert_eq!(one("version of server"), "0.1.0");
    assert_eq!(one("version string of server"), "0.1.0");
    assert_eq!(one("minecraft version"), "1.21.9");
    assert_eq!(one("version of minecraft"), "1.21.9");
    assert_eq!(one("view distance of server"), "10");
    assert_eq!(one("server view distance"), "10");
}

#[test]
fn tps_in_conditions() {
    let out = run(&["if tps < 20:", r#"    send "lag""#, "if tps > 15:", r#"    send "ok""#]);
    assert_eq!(out, vec!["lag", "ok"]);
}

#[test]
fn player_facts() {
    assert_eq!(one("view distance of player"), "8");
    assert_eq!(one("client view distance of player"), "8");
    assert_eq!(one("time played of player"), "1 hour");
    assert_eq!(one("time lived of player"), "1 minute");
    let out = run(&["if time played of player > 30 minutes:", r#"    send "veteran""#]);
    assert_eq!(out, vec!["veteran"]);
    assert_eq!(one("total minutes of time played of player"), "60");
}

#[test]
fn worlds_and_environment() {
    let out = run(&[
        "loop all worlds:",
        r#"    send "w=%loop-value%""#,
        "set {w::*} to all worlds",
        r#"send "n=%size of {w::*}%""#,
        "set {x::*} to worlds",
        r#"send "m=%size of {x::*}%""#,
    ]);
    assert_eq!(out, vec!["w=world", "w=nether", "n=2", "m=2"]);
    assert_eq!(one("environment of world \"world\""), "normal");
    assert_eq!(one("environment of world \"nether\""), "nether");
    assert_eq!(one("world environment of world \"nether\""), "nether");
    assert_eq!(one("environment of world of player"), "normal");
}

#[test]
fn type_of_and_item_count() {
    assert_eq!(one("type of 5 diamonds"), "diamond");
    assert_eq!(one("type of \"minecraft:oak log\""), "oak log");
    assert_eq!(one("3 of diamond"), "3 diamond");
    assert_eq!(one("64 of oak log"), "64 oak log");
    let out = run(&["set {i} to 2 diamonds", "set {j} to type of {i}", r#"send "%{j}%""#]);
    assert_eq!(out, vec!["diamond"]);
}

#[test]
fn whether_comparisons() {
    let out = run(&[
        "set {a} to 5",
        "set {t} to whether {a} is 5",
        r#"send "%{t}%""#,
        "set {t} to whether {a} is not 5",
        r#"send "%{t}%""#,
        "set {t} to whether {a} is greater than 3",
        r#"send "%{t}%""#,
        "set {t} to whether {a} is less than 3",
        r#"send "%{t}%""#,
        "set {t} to whether {a} is at least 5",
        r#"send "%{t}%""#,
        "set {t} to whether {a} is at most 4",
        r#"send "%{t}%""#,
        "set {t} to whether {a} is set",
        r#"send "%{t}%""#,
        "set {t} to whether {zzz} is not set",
        r#"send "%{t}%""#,
        "set {t} to whether {a} >= 6",
        r#"send "%{t}%""#,
        "set {t} to whether {a} + 1 = 6",
        r#"send "%{t}%""#,
    ]);
    assert_eq!(
        out,
        vec!["true", "false", "true", "false", "true", "false", "true", "true", "false", "true"]
    );
}

#[test]
fn existing_syntax_still_parses() {
    let out = run(&[
        "set {x} to 2 diamonds",
        r#"send "%x of location of player%""#,
        r#"send "%location of player%""#,
        r#"send "%5 minutes%""#,
        "set {n} to 7",
        r#"send "%{n} * 2 + 1%""#,
        r#"send "%size of {zz::*}%""#,
        r#"send "%{x}%""#,
    ]);
    assert_eq!(
        out,
        vec!["10", "x: 10, y: 64, z: -5 in 'world'", "5 minutes", "15", "0", "2 diamond"]
    );
}

#[test]
fn compass_and_loc_of_vector_prop() {
    assert_eq!(one("location of vector(1, 2, 3)"), "x: 1, y: 2, z: 3");
}
