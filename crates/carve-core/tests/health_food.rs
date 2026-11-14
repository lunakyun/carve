use carve_core::compile::{EventKind, check_registry, compile_file};
use carve_core::health_food::Vital;
use carve_core::*;

/// A player with health, max health, food and saturation.
struct Mock {
    health: f64,
    max: f64,
    food: f64,
    sat: f64,
    calls: Vec<String>,
    errors: Vec<String>,
}

impl Mock {
    fn new() -> Self {
        Mock {
            health: 10.0,
            max: 20.0,
            food: 8.0,
            sat: 2.0,
            calls: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        let _ = to;
        self.calls.push(format!("send {text}"));
    }
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {
        self.health = self.max;
    }
    fn set_health(&mut self, _: &PlayerRef, h: f64) {
        self.health = h;
    }
    fn kill(&mut self, _: &PlayerRef) {
        self.health = 0.0;
        self.calls.push("kill".into());
    }
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, _: &str) {}
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
    }
    fn find_player(&mut self, _: &str) -> Option<PlayerRef> {
        None
    }
    fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
        true
    }
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        if level == LogLevel::Error {
            self.errors.push(msg.to_string());
        }
    }
    fn health(&mut self, _: &PlayerRef) -> Option<f64> {
        Some(self.health)
    }
    fn vital(&mut self, _: &PlayerRef, v: Vital) -> Option<f64> {
        Some(match v {
            Vital::Health => self.health,
            Vital::MaxHealth => self.max,
            Vital::Food => self.food,
            Vital::Saturation => self.sat,
        })
    }
    fn set_vital(&mut self, _: &PlayerRef, v: Vital, x: f64) {
        match v {
            Vital::Health => self.health = x,
            Vital::MaxHealth => self.max = x,
            Vital::Food => self.food = x,
            Vital::Saturation => self.sat = x,
        }
    }
    fn damage(&mut self, _: &PlayerRef, a: f64) {
        self.calls.push(format!("damage {a}"));
        self.health = (self.health - a).max(0.0);
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

/// Run a join trigger with the given body lines.
fn run(body: &str) -> Mock {
    let src = format!(
        "on join:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    );
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    assert!(
        errs.is_empty(),
        "{}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    let mut h = Mock::new();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.errors.is_empty(), "runtime errors: {:?}", h.errors);
    h
}

fn bad(line: &str) -> bool {
    compile_file("b.carve", &format!("on join:\n    {line}\n")).is_err()
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn set_each_vital() {
    let h = run(
        "set health of player to 6\nset max health of player to 30\nset food level of player to 15\nset saturation of player to 5",
    );
    assert_eq!((h.health, h.max, h.food, h.sat), (6.0, 30.0, 15.0, 5.0));
}

#[test]
fn possessive_and_aliases() {
    let h = run(
        "set player's health to 4\nset player's hunger to 3\nset hunger of player to 9\nset player's max health to 40",
    );
    assert_eq!((h.health, h.food, h.max), (4.0, 9.0, 40.0));
}

#[test]
fn add_and_remove() {
    let h = run(
        "add 5 to health of player\nremove 2 from food level of player\nadd 3 to player's saturation\nadd 10 to max health of player",
    );
    assert_eq!((h.health, h.food, h.sat, h.max), (15.0, 6.0, 5.0, 30.0));
}

#[test]
fn clamping() {
    let h = run(
        "add 100 to health of player\nset food level of player to 99\nset saturation of player to -4\nset max health of player to 0",
    );
    assert_eq!((h.health, h.food, h.sat), (1.0, 20.0, 0.0));
    assert_eq!(h.max, 1.0);
}

#[test]
fn health_cannot_pass_max() {
    let h = run("add 100 to health of player");
    assert_eq!(h.health, 20.0);
}

#[test]
fn remove_to_zero_kills() {
    let h = run("remove 50 from health of player");
    assert_eq!(h.calls, vec!["kill"]);
    assert_eq!(h.health, 0.0);
}

#[test]
fn damage_heal_by_feed() {
    let h = run("damage player by 3");
    assert_eq!(h.health, 7.0);
    assert_eq!(h.calls, vec!["damage 3"]);
    let h = run("heal player by 4");
    assert_eq!(h.health, 14.0);
    let h = run("heal player by 400");
    assert_eq!(h.health, 20.0);
    let h = run("feed player");
    assert_eq!((h.food, h.sat), (20.0, 20.0));
    let h = run("feed player by 4");
    assert_eq!((h.food, h.sat), (12.0, 2.0));
}

#[test]
fn plain_heal_and_kill_still_work() {
    let h = run("heal player");
    assert_eq!(h.health, 20.0);
    let h = run("kill player");
    assert_eq!(h.calls, vec!["kill"]);
}

#[test]
fn read_each_vital() {
    let h = run(
        "send \"%max health of player% %player's max health% %food level of player% %player's food level% %hunger of player% %saturation of player% %player's saturation level% %health of player%\"",
    );
    assert_eq!(h.calls, vec!["send 20 20 8 8 8 2 2 10"]);
}

#[test]
fn read_in_conditions_and_math() {
    let h = run(
        "if food level of player < 10:\n    send \"hungry\"\nset {x} to max health of player - health of player\nsend \"%{x}%\"",
    );
    assert_eq!(h.calls, vec!["send hungry", "send 10"]);
}

#[test]
fn bad_syntax_is_rejected() {
    assert!(bad("damage player"));
    assert!(bad("damage player by \"a\""));
    assert!(bad("set food level of player to \"full\""));
    assert!(bad("feed 5"));
}

#[test]
fn default_host_damage_uses_health() {
    struct Min(f64, bool);
    impl Host for Min {
        fn send_message(&mut self, _: &Recipient, _: &str) {}
        fn broadcast(&mut self, _: &str) {}
        fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
        fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
        fn heal(&mut self, _: &PlayerRef) {}
        fn set_health(&mut self, _: &PlayerRef, h: f64) {
            self.0 = h;
        }
        fn kill(&mut self, _: &PlayerRef) {
            self.1 = true;
        }
        fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
        fn execute_command(&mut self, _: &str) {}
        fn resolve_player(&mut self, _: &str) -> Option<String> {
            Some("Steve".into())
        }
        fn find_player(&mut self, _: &str) -> Option<PlayerRef> {
            None
        }
        fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
            true
        }
        fn schedule_wait(&mut self, _: u64, _: u64) {}
        fn now(&mut self) -> u64 {
            0
        }
        fn log(&mut self, _: LogLevel, _: &str) {}
        fn health(&mut self, _: &PlayerRef) -> Option<f64> {
            Some(self.0)
        }
    }
    let mut m = Min(10.0, false);
    m.damage(&steve(), 4.0);
    assert_eq!(m.0, 6.0);
    m.damage(&steve(), 9.0);
    assert!(m.1);
}
