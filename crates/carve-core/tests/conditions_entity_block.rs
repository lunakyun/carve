use std::collections::HashSet;

use carve_core::compile::{EventKind, check_registry};
use carve_core::conditions_entity_block::{BlockTrait, PlayerState};
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    states: HashSet<PlayerState>,
    pvp_on: Vec<String>,
    solid: Vec<&'static str>,
    held: Option<String>,
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
    fn player_location(&mut self, p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        (p.uuid == "u1").then(|| (10.0, 64.0, 10.0, "world".to_string()))
    }
    fn held_item(&mut self, _: &PlayerRef) -> Option<(String, u32)> {
        self.held.clone().map(|h| (h, 1))
    }
    fn player_state(&mut self, _: &PlayerRef, s: PlayerState) -> bool {
        self.states.contains(&s)
    }
    fn pvp_enabled(&mut self, world: Option<&str>) -> Option<bool> {
        Some(
            self.pvp_on
                .iter()
                .any(|w| Some(w.as_str()) == world.or(Some("world"))),
        )
    }
    fn block_trait(&mut self, id: &str, t: BlockTrait) -> Option<bool> {
        let known = ["stone", "glass", "air", "grass"];
        known.contains(&id).then(|| match t {
            BlockTrait::Solid => self.solid.contains(&id),
            BlockTrait::Transparent => matches!(id, "glass" | "air"),
            BlockTrait::Passable => matches!(id, "air" | "grass"),
        })
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn run_with(mut h: Mock, src: &str) -> (Vec<String>, Vec<(LogLevel, String)>) {
    let mut e = Engine::default();
    let files = vec![
        ("t.carve".to_string(), src.to_string()),
        (
            "scripts/Shop.carve".to_string(),
            "on quit:\n    send \"bye\"\n".to_string(),
        ),
    ];
    let (_, errs) = e.load(&files);
    assert!(
        errs.is_empty(),
        "errors: {:?}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>()
    );
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    (h.calls, h.logs)
}

fn check_with(h: Mock, cond: &str) -> bool {
    let src = format!("on join:\n    if {cond}:\n        send \"yes\"\n    else:\n        send \"no\"\n");
    let (calls, logs) = run_with(h, &src);
    assert!(
        logs.iter().all(|(l, _)| *l != LogLevel::Error),
        "runtime errors: {logs:?}"
    );
    calls[0].ends_with("yes")
}

fn check(cond: &str) -> bool {
    check_with(Mock::default(), cond)
}

fn compile_error(cond: &str) -> bool {
    let src = format!("on join:\n    if {cond}:\n        send \"yes\"\n");
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    !errs.is_empty()
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn is_valid() {
    assert!(check("player is valid"));
    assert!(!check("player isn't valid"));
    assert!(!check("{_nothing} is valid"));
    assert!(check("{_nothing} is not valid"));
    assert!(check("(\"abc\") is valid"));
    assert!(check("location(1, 2, 3) is valid"));
}

#[test]
fn script_loaded() {
    assert!(check("script \"shop\" is loaded"));
    assert!(check("script \"Shop.carve\" is loaded"));
    assert!(check("script \"t\" is loaded"));
    assert!(!check("script \"nope\" is loaded"));
    assert!(check("script \"nope\" isn't loaded"));
    assert!(!check("script \"shop\" is not loaded"));
}

#[test]
fn regex_matches() {
    assert!(check("\"abc123\" matches \"[a-z]+\\d+\""));
    assert!(!check("\"abc123x\" matches \"[a-z]+\\d+\""));
    assert!(check("\"abc123x\" doesn't match \"[a-z]+\\d+\""));
    assert!(check("\"abc123x\" does not match \"\\d\""));
    assert!(check("\"Hello\" matches \"(?i)hello\""));
    assert!(check("\"foo bar baz\" contains regex \"b[a-z]r\""));
    assert!(!check("\"foo bar baz\" contains regex \"^bar\""));
    assert!(check("\"foo\" doesn't contain regex \"x+\""));
    assert!(check("\"a.b\" matches \"a\\.b\""));
    assert!(!check("\"axb\" matches \"a\\.b\""));
}

#[test]
fn regex_with_variables() {
    let src = "on join:\n    set {_t} to \"ab12\"\n    set {_p} to \"[a-z]{2}\\d{2}\"\n    if {_t} matches {_p}:\n        send \"ok\"\n";
    let (calls, _) = run_with(Mock::default(), src);
    assert_eq!(calls, ["send Steve ok"]);
}

#[test]
fn bad_regex_is_a_runtime_error() {
    let src = "on join:\n    if \"a\" matches \"(\":\n        send \"yes\"\n    send \"after\"\n";
    let (calls, logs) = run_with(Mock::default(), src);
    assert!(
        logs.iter()
            .any(|(l, m)| *l == LogLevel::Error && m.contains("invalid regex")),
        "{logs:?}"
    );
    assert!(!calls.iter().any(|c| c.ends_with("yes")));
}

#[test]
fn runaway_regex_is_stopped() {
    let src = "on join:\n    set {_t} to \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab\"\n    if {_t} contains regex \"(a+)+$\":\n        send \"yes\"\n";
    let (_, logs) = run_with(Mock::default(), src);
    assert!(logs.iter().any(|(_, m)| m.contains("too complex")), "{logs:?}");
}

#[test]
fn text_conditions_still_work() {
    assert!(check("\"hello\" contains \"ell\""));
    assert!(check("\"hello\" is \"HELLO\""));
    assert!(check("\"hello\" doesn't contain \"x\""));
}

#[test]
fn within_box() {
    assert!(check(
        "location(5, 5, 5) is within location(0, 0, 0) and location(10, 10, 10)"
    ));
    assert!(check(
        "location(5, 5, 5) is within location(10, 10, 10) to location(0, 0, 0)"
    ));
    assert!(!check(
        "location(11, 5, 5) is within location(0, 0, 0) and location(10, 10, 10)"
    ));
    assert!(check(
        "location(11, 5, 5) isn't within location(0, 0, 0) and location(10, 10, 10)"
    ));
    assert!(check(
        "location(10, 10, 10) is within location(0, 0, 0) and location(10, 10, 10)"
    ));
    // The player is at 10, 64, 10 in world.
    assert!(check(
        "player is within location(0, 60, 0) and location(20, 70, 20)"
    ));
    assert!(!check("player is within location(0, 0, 0) and location(5, 5, 5)"));
    assert!(check("player is within player's location and player's location"));
}

#[test]
fn within_box_checks_worlds() {
    assert!(!check(
        "location(5, 5, 5, world \"nether\") is within location(0, 0, 0, world \"world\") and location(10, 10, 10, world \"world\")"
    ));
    assert!(check(
        "location(5, 5, 5, world \"nether\") is within location(0, 0, 0) and location(10, 10, 10)"
    ));
}

#[test]
fn within_radius() {
    assert!(check("location(3, 0, 4) is within 5 blocks of location(0, 0, 0)"));
    assert!(!check(
        "location(3, 0, 5) is within 5 blocks around location(0, 0, 0)"
    ));
    assert!(check(
        "location(3, 0, 5) isn't within 5 blocks of location(0, 0, 0)"
    ));
    assert!(check(
        "location(0, 0, 1) is within 1 block from location(0, 0, 0)"
    ));
    assert!(check("location(0, 0, 1) is within 2 meters of location(0, 0, 0)"));
    assert!(check("player is within 1 block of player"));
    assert!(check("player is within 3 blocks of location(10, 64, 12)"));
    assert!(!check("player is within 3 blocks of location(10, 64, 14)"));
}

#[test]
fn within_world() {
    assert!(check("player is within world \"world\""));
    assert!(!check("player is within world \"nether\""));
    assert!(check("player isn't within world \"nether\""));
    assert!(check(
        "location(1, 2, 3, world \"nether\") is within world \"nether\""
    ));
}

#[test]
fn player_states() {
    for (word, st) in [
        ("poisoned", PlayerState::Poisoned),
        ("silent", PlayerState::Silent),
        ("riding", PlayerState::Riding),
    ] {
        let on = || Mock {
            states: [st].into_iter().collect(),
            ..Mock::default()
        };
        assert!(check_with(on(), &format!("player is {word}")), "{word}");
        assert!(!check(&format!("player is {word}")), "{word} off");
        assert!(check(&format!("player isn't {word}")), "isn't {word}");
        assert!(check(&format!("player is not {word}")), "not {word}");
        assert!(
            !check_with(on(), &format!("player is not {word}")),
            "not {word} on"
        );
    }
}

#[test]
fn states_are_independent() {
    let h = Mock {
        states: [PlayerState::Silent].into_iter().collect(),
        ..Mock::default()
    };
    assert!(!check_with(h, "player is poisoned"));
}

#[test]
fn pvp() {
    let on = || Mock {
        pvp_on: vec!["world".into()],
        ..Mock::default()
    };
    assert!(check_with(on(), "pvp is enabled"));
    assert!(!check_with(on(), "pvp is disabled"));
    assert!(check_with(on(), "pvp is enabled in world \"world\""));
    assert!(!check_with(on(), "pvp is enabled in world \"nether\""));
    assert!(check_with(on(), "pvp is disabled in world \"nether\""));
    assert!(check_with(on(), "pvp is not enabled in world \"nether\""));
    assert!(!check("pvp is enabled"));
    assert!(check("pvp is disabled"));
}

#[test]
fn instant_effects() {
    assert!(check("\"instant health\" is instant"));
    assert!(check("\"minecraft:instant_damage\" is instant"));
    assert!(check("\"saturation\" is instant"));
    assert!(!check("\"poison\" is instant"));
    assert!(check("\"poison\" isn't instant"));
    assert!(check("\"speed\" is not instant"));
}

#[test]
fn block_traits() {
    let with = |item: &str| Mock {
        held: Some(item.into()),
        solid: vec!["stone", "grass"],
        ..Mock::default()
    };
    assert!(check_with(with("stone"), "player's tool is solid"));
    assert!(!check_with(with("glass"), "player's tool is solid"));
    assert!(check_with(with("glass"), "player's tool isn't solid"));
    assert!(check_with(with("glass"), "player's tool is transparent"));
    assert!(!check_with(with("stone"), "player's tool is transparent"));
    assert!(check_with(with("grass"), "player's tool is passable"));
    assert!(check_with(with("stone"), "player's tool is not passable"));
    // Unknown block: every form answers no.
    assert!(!check_with(with("unobtainium"), "player's tool is solid"));
    assert!(!check_with(with("unobtainium"), "player's tool isn't solid"));
}

#[test]
fn old_syntax_still_works() {
    assert!(check("1 is 1"));
    assert!(check("player is player"));
    assert!(check("{_x} is not set"));
    assert!(check("5 is between 1 and 10"));
}

#[test]
fn combine_with_and_or() {
    let h = Mock {
        states: [PlayerState::Silent].into_iter().collect(),
        ..Mock::default()
    };
    assert!(check_with(h, "player is silent and \"ab\" matches \"a.\""));
    assert!(check(
        "player is poisoned or location(1, 1, 1) is within 2 blocks of location(1, 1, 2)"
    ));
}

#[test]
fn typos_are_errors() {
    assert!(compile_error("player is within"));
    assert!(compile_error("script is loaded"));
}
