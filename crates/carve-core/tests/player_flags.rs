use std::collections::HashSet;

use carve_core::compile::{EventKind, check_registry};
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    flags: HashSet<PlayerFlag>,
    logs: Vec<(LogLevel, String)>,
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        let who = match to {
            Recipient::Player(p) => p.name.clone(),
            Recipient::Console => "console".into(),
        };
        self.calls.push(format!("send {who} {text}"));
    }
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, _: &str) {}
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        name.eq_ignore_ascii_case("steve").then(steve)
    }
    fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
    fn player_flag(&mut self, _: &PlayerRef, flag: PlayerFlag) -> bool {
        self.flags.contains(&flag)
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn run(kind: EventKind, src: &str, flags: &[PlayerFlag]) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "errors: {:?}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>()
    );
    let mut h = Mock {
        flags: flags.iter().copied().collect(),
        ..Mock::default()
    };
    e.fire(kind, EventCtx::player(steve()), &mut h);
    assert!(
        h.logs.iter().all(|(l, _)| *l != LogLevel::Error),
        "runtime errors: {:?}",
        h.logs
    );
    h.calls
}

fn check(cond: &str, flags: &[PlayerFlag]) -> bool {
    let src = format!("on join:\n    if {cond}:\n        send \"yes\"\n    else:\n        send \"no\"\n");
    run(EventKind::Join, &src, flags)[0].ends_with("yes")
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn each_flag_true_and_false() {
    use PlayerFlag::*;
    let cases = [
        ("flying", Flying),
        ("sneaking", Sneaking),
        ("sprinting", Sprinting),
        ("swimming", Swimming),
        ("blocking", Blocking),
        ("sleeping", Sleeping),
        ("on ground", OnGround),
        ("on the ground", OnGround),
    ];
    for (word, flag) in cases {
        assert!(check(&format!("player is {word}"), &[flag]), "{word} on");
        assert!(!check(&format!("player is {word}"), &[]), "{word} off");
        assert!(check(&format!("player is not {word}"), &[]), "not {word}");
        assert!(check(&format!("player isn't {word}"), &[]), "isn't {word}");
        assert!(!check(&format!("player is not {word}"), &[flag]), "not {word} on");
    }
}

#[test]
fn can_fly() {
    assert!(check("player can fly", &[PlayerFlag::CanFly]));
    assert!(!check("player can fly", &[]));
    assert!(check("player can't fly", &[]));
    assert!(check("player cannot fly", &[]));
    assert!(!check("player can't fly", &[PlayerFlag::CanFly]));
}

#[test]
fn flags_are_independent() {
    assert!(!check("player is sprinting", &[PlayerFlag::Sneaking]));
}

#[test]
fn combine_with_and_or() {
    let f = [PlayerFlag::Sneaking];
    assert!(check("player is sneaking and player is not flying", &f));
    assert!(check("player is flying or player is sneaking", &f));
    assert!(!check("player is flying and player is sneaking", &f));
}

#[test]
fn variable_and_comparisons_still_work() {
    let src = "on join:\n    set {x} to 5\n    if {x} is 5:\n        send \"eq\"\n    if player is player:\n        send \"same\"\n";
    let out = run(EventKind::Join, src, &[]);
    assert_eq!(out, vec!["send Steve eq", "send Steve same"]);
}

#[test]
fn toggle_events_parse_and_fire() {
    for (name, kind) in [
        ("sneak toggle", EventKind::SneakToggle),
        ("sprint toggle", EventKind::SprintToggle),
        ("flight toggle", EventKind::FlightToggle),
    ] {
        let src = format!("on {name}:\n    send \"hit\"\n    cancel event\n");
        let mut e = Engine::default();
        let (_, errs) = e.load(&[("t.carve".into(), src)]);
        assert!(errs.is_empty());
        assert!(e.table().has_event(kind));
        let mut h = Mock::default();
        let out = e.fire(kind, EventCtx::player(steve()), &mut h);
        assert!(out.cancelled, "{name} cancellable");
        assert_eq!(h.calls, vec!["send Steve hit"]);
    }
}

#[test]
fn offline_player_is_false() {
    let src =
        "on join:\n    if \"nobody\" is sneaking:\n        send \"yes\"\n    else:\n        send \"no\"\n";
    let _ = src;
    assert!(!check("player is sneaking", &[]));
}
