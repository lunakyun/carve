use std::collections::HashMap;

use carve_core::compile::{EventKind, check_registry};
use carve_core::xp_level::XpField;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    level: f64,
    points: f64,
    progress: f64,
    /// When false, xp_get is unavailable and the command fallback is used.
    api: bool,
    logs: Vec<(LogLevel, String)>,
    online: HashMap<String, String>,
}

impl Mock {
    fn new(api: bool) -> Self {
        let mut m = Mock {
            api,
            level: 5.0,
            points: 100.0,
            progress: 0.5,
            ..Mock::default()
        };
        m.online.insert("u1".into(), "Steve".into());
        m
    }
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.calls.push(format!("send {text}"));
    }
    fn broadcast(&mut self, _t: &str) {}
    fn send_title(&mut self, _p: &PlayerRef, _t: &str, _s: Option<&str>) {}
    fn send_action_bar(&mut self, _p: &PlayerRef, _t: &str) {}
    fn heal(&mut self, _p: &PlayerRef) {}
    fn set_health(&mut self, _p: &PlayerRef, _h: f64) {}
    fn kill(&mut self, _p: &PlayerRef) {}
    fn teleport(&mut self, _p: &PlayerRef, _x: f64, _y: f64, _z: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.calls.push(format!("cmd {c}"));
    }
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
    fn xp_get(&mut self, _p: &PlayerRef, f: XpField) -> Option<f64> {
        self.api.then_some(match f {
            XpField::Level => self.level,
            XpField::Points => self.points,
            XpField::Progress => self.progress,
        })
    }
    fn xp_set(&mut self, _p: &PlayerRef, f: XpField, v: f64) {
        if !self.api {
            self.calls.push(format!("set {f:?} {v}"));
            return;
        }
        match f {
            XpField::Level => self.level = v,
            XpField::Points => self.points = v,
            XpField::Progress => self.progress = v,
        }
    }
    fn xp_add(&mut self, _p: &PlayerRef, f: XpField, v: f64) {
        self.calls.push(format!("add {f:?} {v}"));
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn run(src: &str, h: &mut Mock) {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
    let bad: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(bad.is_empty(), "runtime errors: {bad:?}");
}

fn body(lines: &[&str]) -> String {
    let mut s = String::from("on join:\n");
    for l in lines {
        s.push_str("    ");
        s.push_str(l);
        s.push('\n');
    }
    s
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn reads() {
    let mut h = Mock::new(true);
    run(
        &body(&[
            r#"send "l=%level of player%""#,
            r#"send "p=%player's level%""#,
            r#"send "x=%xp of player% %experience of player% %total experience of player% %player's xp%""#,
            r#"send "pr=%level progress of player% %player's level progress%""#,
            "set {a} to level of player + 1",
            r#"send "a=%{a}%""#,
        ]),
        &mut h,
    );
    assert_eq!(
        h.calls,
        vec![
            "send l=5",
            "send p=5",
            "send x=100 100 100 100",
            "send pr=0.5 0.5",
            "send a=6"
        ]
    );
}

#[test]
fn set_effects() {
    let mut h = Mock::new(true);
    run(
        &body(&[
            "set level of player to 30",
            "set player's xp to 250",
            "set level progress of player to 0.25",
        ]),
        &mut h,
    );
    assert_eq!((h.level, h.points, h.progress), (30.0, 250.0, 0.25));
}

#[test]
fn add_remove_clamp() {
    let mut h = Mock::new(true);
    run(
        &body(&[
            "add 3 to level of player",
            "add 2 levels to player",
            "give player 4 levels",
            "give 1 level to player",
            "remove 1 from player's level",
        ]),
        &mut h,
    );
    assert_eq!(h.level, 14.0);
    run(&body(&["remove 100 levels from player"]), &mut h);
    assert_eq!(h.level, 0.0);
    run(&body(&["add 5 to level progress of player"]), &mut h);
    assert_eq!(h.progress, 1.0);
    run(&body(&["remove 3 from level progress of player"]), &mut h);
    assert_eq!(h.progress, 0.0);
}

#[test]
fn xp_points() {
    let mut h = Mock::new(true);
    run(
        &body(&[
            "give player 10 xp",
            "give 5 exp points to player",
            "add 20 xp to player",
            "add 1 to experience of player",
            "remove 6 xp from player",
        ]),
        &mut h,
    );
    assert_eq!(h.points, 130.0);
}

#[test]
fn command_fallback() {
    let mut h = Mock::new(false);
    run(
        &body(&[
            "set level of player to 7",
            "give player 10 xp",
            "remove 2 levels from player",
            "add 1 to level progress of player",
            r#"send "%level of player%""#,
        ]),
        &mut h,
    );
    assert_eq!(
        h.calls,
        vec!["set Level 7", "add Points 10", "add Level -2", "send <none>"]
    );
}

#[test]
fn works_with_variables() {
    let mut h = Mock::new(true);
    run(
        &body(&[
            "set {n} to 12",
            "set level of player to {n} * 2",
            "add {n} to level of player",
        ]),
        &mut h,
    );
    assert_eq!(h.level, 36.0);
}
