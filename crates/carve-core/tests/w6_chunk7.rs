use carve_core::compile::{EventKind, compile_file};
use carve_core::w6_chunk7::{alias_names, axis_angle};
use carve_core::*;

#[derive(Default)]
struct Mock {
    sent: Vec<String>,
    errors: Vec<String>,
    commands: Vec<String>,
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.sent.push(text.to_string());
    }
    fn broadcast(&mut self, _text: &str) {}
    fn send_title(&mut self, _p: &PlayerRef, _t: &str, _s: Option<&str>) {}
    fn send_action_bar(&mut self, _p: &PlayerRef, _t: &str) {}
    fn heal(&mut self, _p: &PlayerRef) {}
    fn set_health(&mut self, _p: &PlayerRef, _h: f64) {}
    fn kill(&mut self, _p: &PlayerRef) {}
    fn teleport(&mut self, _p: &PlayerRef, _x: f64, _y: f64, _z: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.commands.push(c.to_string());
    }
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
    }
    fn find_player(&mut self, n: &str) -> Option<PlayerRef> {
        (n == "Steve").then(steve)
    }
    fn has_permission(&mut self, _p: &PlayerRef, _n: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _t: u64, _k: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn entity_by_uuid(&mut self, uuid: &str) -> Option<EntityRef> {
        (uuid == "z1").then(|| EntityRef {
            uuid: "z1".into(),
            kind: "zombie".into(),
        })
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        if level == LogLevel::Error {
            self.errors.push(msg.to_string());
        }
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn script_for(body: &str) -> String {
    format!(
        "on join:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    )
}

fn run_src(src: String) -> Vec<String> {
    run_cmds(src).0
}

fn run_cmds(src: String) -> (Vec<String>, Vec<String>) {
    let mut h = Mock::default();
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    assert!(
        errs.is_empty(),
        "{}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.errors.is_empty(), "{:?}", h.errors);
    (h.sent, h.commands)
}

fn run(body: &str) -> Vec<String> {
    run_src(script_for(body))
}

#[test]
fn min_and_max_take_any_number_of_arguments() {
    assert_eq!(run("send \"%max(1, 9, 4, 7)%\""), ["9"]);
    assert_eq!(run("send \"%min(5, 3, 8, 2, 6)%\""), ["2"]);
    assert_eq!(run("send \"%max(2, 3)%\""), ["3"]);
    assert_eq!(run("send \"%min(4, -1, 2)%\""), ["-1"]);
}

#[test]
fn round_takes_decimal_places() {
    assert_eq!(run("send \"%round(3.14159, 2)%\""), ["3.14"]);
    assert_eq!(run("send \"%round(2.5)%\""), ["3"]);
    assert_eq!(run("send \"%round(1234.5678, 0)%\""), ["1235"]);
}

#[test]
fn world_call_form() {
    assert_eq!(run("send \"%world(\"nether\")%\""), ["nether"]);
}

#[test]
fn entity_from_uuid_finds_players_and_other_entities() {
    assert_eq!(run("send \"%entity(\"u1\")%\""), ["Steve"]);
    assert_eq!(
        run("set {_e} to entity from uuid \"z1\"\nif {_e} is set:\n    send \"found\""),
        ["found"]
    );
    assert_eq!(
        run("set {_e} to entity(\"nope\")\nif {_e} is not set:\n    send \"none\""),
        ["none"]
    );
}

#[test]
fn axis_angle_builds_a_unit_quaternion() {
    let Ok(Value::Quaternion { w, x, y, z }) = axis_angle(180.0, 0.0, 2.0, 0.0) else {
        panic!("not a quaternion")
    };
    assert!(w.abs() < 1e-9 && x == 0.0 && (y - 1.0).abs() < 1e-9 && z == 0.0);
    assert!(axis_angle(90.0, 0.0, 0.0, 0.0).is_err());
}

#[test]
fn axis_angle_in_scripts() {
    let out = run("set {_q} to axisAngle(90, 0, 1, 0)\nsend \"%{_q}%\"");
    assert_eq!(out.len(), 1);
    assert!(out[0].contains("0.7071"), "{}", out[0]);
    let out = run("set {_q} to axisAngle(90, vector(0, 1, 0))\nsend \"%{_q}%\"");
    assert!(out[0].contains("0.7071"), "{}", out[0]);
}

#[test]
fn for_each_loop_sets_the_variable() {
    let out = run("set {_l::*} to 1, 2 and 3\nfor each {_x} in {_l::*}:\n    send \"v%{_x}%\"");
    assert_eq!(out, ["v1", "v2", "v3"]);
    let out = run("set {_l::*} to 4 and 5\nfor each value in {_l::*}:\n    send \"w%loop-value%\"");
    assert_eq!(out, ["w4", "w5"]);
}

#[test]
fn for_each_nests_with_break() {
    let out = run(
        "set {_l::*} to 1, 2 and 3\nfor each {_a} in {_l::*}:\n    for each {_b} in {_l::*}:\n        if {_b} > {_a}:\n            stop loop\n        send \"%{_a}%-%{_b}%\"",
    );
    assert_eq!(out, ["1-1", "2-1", "2-2", "3-1", "3-2", "3-3"]);
}

#[test]
fn alias_patterns_expand() {
    assert_eq!(
        alias_names("my (red|blue) sword"),
        ["my red sword", "my blue sword"]
    );
    assert_eq!(alias_names("the [big] key"), ["the big key", "the key"]);
    assert_eq!(alias_names("plain"), ["plain"]);
}

#[test]
fn aliases_with_patterns_compile() {
    for name in ["my fast sword", "my quick sword", "my quick swords"] {
        let src = format!(
            "aliases:\n    my (fast|quick) sword = diamond sword\n\non join:\n    if player has {name}:\n        send \"a\"\n"
        );
        assert!(compile_file("t.carve", &src).is_ok(), "{name}");
    }
    let src = "aliases:\n    my (fast|quick) sword = diamond sword\n\non join:\n    if player has my slow sword:\n        send \"a\"\n";
    assert!(compile_file("t.carve", src).is_err());
}

#[test]
fn defining_a_plural_yourself_is_not_a_duplicate() {
    let src = "aliases:\n    blade = diamond sword\n    blades = iron sword\n\non join:\n    send \"x\"\n";
    assert!(compile_file("t.carve", src).is_ok());
}

#[test]
fn suppress_warning_and_using_lines_are_accepted() {
    let out = run_src(
        "using 3d vectors\n\non join:\n    locally suppress conflict warnings\n    send \"ok\"\n".to_string(),
    );
    assert_eq!(out, ["ok"]);
}

#[test]
fn team_death_message_visibility_uses_team_modify() {
    let (out, cmds) = run_cmds(script_for(
        "register team \"t\"
set death message visibility of team \"t\" to hide for other teams
send \"%team death message visibility of \"\"t\"\"%\"",
    ));
    assert_eq!(out, ["hide for other teams"]);
    assert!(
        cmds.iter()
            .any(|c| c == "team modify t deathMessageVisibility hideForOtherTeams"),
        "{cmds:?}"
    );
}
