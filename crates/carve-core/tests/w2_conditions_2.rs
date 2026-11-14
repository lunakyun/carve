use std::collections::HashSet;

use carve_core::compile::{EventKind, check_registry};
use carve_core::*;

#[derive(Default)]
struct H {
    sent: Vec<String>,
    flags: HashSet<&'static str>,
    health: f64,
    now: i64,
}

impl Host for H {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.sent.push(text.to_string());
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
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        name.eq_ignore_ascii_case("steve").then(steve)
    }
    fn has_permission(&mut self, _p: &PlayerRef, _n: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _t: u64, _k: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        if level == LogLevel::Error {
            self.sent.push(format!("ERROR {msg}"));
        }
    }
    fn wall_clock_millis(&mut self) -> i64 {
        self.now
    }
    fn health(&mut self, _p: &PlayerRef) -> Option<f64> {
        Some(self.health)
    }
    fn player_location(&mut self, _p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((0.0, 64.0, 0.0, "world_nether".into()))
    }
    fn player_flag(&mut self, _p: &PlayerRef, flag: PlayerFlag) -> bool {
        self.flags.contains(flag.name())
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn host() -> H {
    H {
        health: 20.0,
        now: 1_000_000_000,
        ..H::default()
    }
}

/// Run `on join:` with one `if <cond>` line; true when the branch ran.
fn check_with(cond: &str, h: &mut H) -> bool {
    let src = format!("on join:\n\tif {cond}:\n\t\tsend \"yes\"\n\telse:\n\t\tsend \"no\"\n");
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    assert!(
        errs.is_empty(),
        "{cond}: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; ")
    );
    h.sent.clear();
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
    assert_eq!(h.sent.len(), 1, "{cond}: {:?}", h.sent);
    h.sent[0] == "yes"
}

fn check(cond: &str) -> bool {
    check_with(cond, &mut host())
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn divisible() {
    assert!(check("10 is evenly divisible by 5"));
    assert!(check("10 is divisible by 2"));
    assert!(!check("10 is evenly divisible by 3"));
    assert!(check("10 isn't evenly divisible by 3"));
    assert!(check("10 is not evenly divisible by 4"));
    assert!(!check("10 is evenly divisible by 0"));
    assert!(check("-9 is evenly divisible by 3"));
    assert!(check("0.5 is evenly divisible by 0.25"));
}

#[test]
fn empty() {
    assert!(check("\"\" is empty"));
    assert!(!check("\"a\" is empty"));
    assert!(check("\"a\" is not empty"));
    assert!(check("{nothing} is empty"));
    assert!(check("{list::*} is empty"));
}

#[test]
fn empty_list_variable() {
    let src = "on join:\n\tset {l::1} to 5\n\tif {l::*} is empty:\n\t\tsend \"e\"\n\telse:\n\t\tsend \"n\"\n\tdelete {l::*}\n\tif {l::*} is empty:\n\t\tsend \"e\"\n";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty());
    let mut h = host();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.sent, vec!["n", "e"]);
}

#[test]
fn online() {
    assert!(check("player is online"));
    assert!(!check("player is offline"));
    assert!(check("\"Steve\" is online"));
    assert!(check("\"Alex\" is offline"));
    assert!(check("\"Alex\" isn't online"));
    assert!(!check("\"Alex\" is online"));
}

#[test]
fn alive() {
    assert!(check("player is alive"));
    assert!(!check("player is dead"));
    let mut h = host();
    h.health = 0.0;
    assert!(!check_with("player is alive", &mut h));
    assert!(check_with("player is dead", &mut h));
    assert!(check_with("player isn't alive", &mut h));
}

#[test]
fn dates() {
    assert!(!check("now is in the future"));
    assert!(check("now - 1 hour is in the past"));
    assert!(check("now + 1 hour is in the future"));
    assert!(check("now + 1 hour isn't in the past"));
    assert!(!check("now + 1 hour is in the past"));
}

#[test]
fn worlds() {
    assert!(check("player is in world \"world_nether\""));
    assert!(check("player is in the world \"WORLD_NETHER\""));
    assert!(!check("player is in world \"world\""));
    assert!(check("player isn't in world \"world\""));
}

#[test]
fn item_traits() {
    assert!(check("\"bread\" is edible"));
    assert!(!check("\"stone\" is edible"));
    assert!(check("\"minecraft:golden_apple\" is edible"));
    assert!(check("\"coal\" is fuel"));
    assert!(check("\"oak_planks\" is fuel"));
    assert!(check("\"stripped_birch_log\" is a fuel"));
    assert!(!check("\"diamond\" is fuel"));
    assert!(check("\"netherite_sword\" is fire resistant"));
    assert!(!check("\"iron_sword\" is fire resistant"));
    assert!(check("\"red_wool\" is flammable"));
    assert!(!check("\"stone\" is flammable"));
    assert!(check("\"stone\" isn't flammable"));
}

#[test]
fn of_type() {
    assert!(check("\"diamond\" is of type \"diamond\""));
    assert!(check("\"Diamond Sword\" is of type \"diamond_sword\""));
    assert!(!check("\"diamond\" is of type \"emerald\""));
    assert!(check("\"diamond\" isn't of type \"emerald\""));
    assert!(check("player is of type \"player\""));
    assert!(!check("player is of type \"console\""));
}

#[test]
fn player_flags() {
    for (cond, flag) in [
        ("player is burning", "burning"),
        ("player is on fire", "burning"),
        ("player is invisible", "invisible"),
        ("player is invulnerable", "invulnerable"),
        ("player is gliding", "gliding"),
        ("player is frozen", "frozen"),
        ("player's custom name is visible", "custom name visible"),
        ("player has a resource pack", "resource pack"),
        ("player has resource pack loaded", "resource pack"),
    ] {
        let mut h = host();
        assert!(!check_with(cond, &mut h), "{cond} off");
        h.flags.insert(flag);
        assert!(check_with(cond, &mut h), "{cond} on");
    }
    let mut h = host();
    assert!(check_with("player isn't burning", &mut h));
    assert!(check_with("player doesn't have a resource pack", &mut h));
    assert!(check_with("player isn't invisible", &mut h));
}

#[test]
fn script_commands() {
    let src = "command /hello:\n\ttrigger:\n\t\tsend \"hi\"\n\ncommand /greet:\n\taliases: /gr\n\ttrigger:\n\t\tsend \"x\"\n\non join:\n\tif command \"hello\" is a script command:\n\t\tsend \"a\"\n\tif command \"/gr\" is a script command:\n\t\tsend \"b\"\n\tif command \"nope\" is a script command:\n\t\tsend \"c\"\n\tif command \"nope\" isn't a script command:\n\t\tsend \"d\"\n";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty(), "{errs:?}");
    let mut h = host();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.sent, vec!["a", "b", "d"]);
}

#[test]
fn existing_comparisons_still_work() {
    assert!(check("1 is 1"));
    assert!(check("\"a\" is not \"b\""));
    assert!(check("3 > 2"));
}
