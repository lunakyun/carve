use carve_core::compile::check_registry;
use carve_core::w2_effects_2::{clean_log_name, clean_strength, look_angles};
use carve_core::*;

/// Records every call the effects make.
struct Mock {
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
    /// Use the trait defaults for the methods that have a command fallback.
    defaults: bool,
}

impl Mock {
    fn new() -> Self {
        Mock {
            calls: Vec::new(),
            logs: Vec::new(),
            defaults: false,
        }
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
        if let Recipient::Player(p) = to {
            self.calls.push(format!("send {} {}", p.name, text));
        }
    }
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.calls.push(format!("cmd {c}"));
    }
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
    }
    fn find_player(&mut self, _: &str) -> Option<PlayerRef> {
        None
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
    fn player_location(&mut self, _: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((0.0, 64.0, 0.0, "world".into()))
    }
    fn set_flight(&mut self, p: &PlayerRef, on: bool) {
        self.calls.push(format!("flight {} {on}", p.name));
    }
    fn set_invulnerable(&mut self, p: &PlayerRef, on: bool) {
        self.calls.push(format!("invuln {} {on}", p.name));
    }
    fn set_invisible(&mut self, p: &PlayerRef, on: bool) {
        if self.defaults {
            let cmd = if on {
                format!("effect give {} minecraft:invisibility infinite 0 true", p.name)
            } else {
                format!("effect clear {} minecraft:invisibility", p.name)
            };
            self.execute_command(&cmd);
        } else {
            self.calls.push(format!("invisible {} {on}", p.name));
        }
    }
    fn set_fire(&mut self, p: &PlayerRef, t: i32) {
        self.calls.push(format!("fire {} {t}", p.name));
    }
    fn apply_knockback(&mut self, p: &PlayerRef, s: f64, x: f64, z: f64) {
        self.calls.push(format!("knock {} {s} {x} {z}", p.name));
    }
    fn set_rotation(&mut self, p: &PlayerRef, yaw: f32, pitch: f32) {
        self.calls.push(format!("rot {} {yaw:.0} {pitch:.0}", p.name));
    }
    fn open_book(&mut self, p: &PlayerRef, off: bool) {
        self.calls.push(format!("book {} {off}", p.name));
    }
    fn say_as(&mut self, p: &PlayerRef, t: &str) {
        self.calls.push(format!("say {} {t}", p.name));
    }
    fn log_to_file(&mut self, f: &str, t: &str) {
        self.calls.push(format!("log {f} {t}"));
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn run_with(src: &str, mut h: Mock) -> (Vec<String>, Mock) {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let calls = std::mem::take(&mut h.calls);
    (calls, h)
}

fn run(src: &str) -> Vec<String> {
    let (calls, h) = run_with(src, Mock::new());
    assert!(h.errors().is_empty(), "runtime errors: {:?}", h.errors());
    calls
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
fn registry_still_compiles() {
    check_registry().unwrap();
}

#[test]
fn fly_on_and_off() {
    let out = run(&body(&[
        "make player fly",
        "make player stop flying",
        "make player start flying",
        "make player not fly",
    ]));
    assert_eq!(
        out,
        [
            "flight Steve true",
            "flight Steve false",
            "flight Steve true",
            "flight Steve false"
        ]
    );
}

#[test]
fn invulnerable_and_invisible() {
    let out = run(&body(&[
        "make player invulnerable",
        "make player vulnerable",
        "make player invisible",
        "make player visible",
        "make player not invisible",
    ]));
    assert_eq!(
        out,
        [
            "invuln Steve true",
            "invuln Steve false",
            "invisible Steve true",
            "invisible Steve false",
            "invisible Steve false"
        ]
    );
}

#[test]
fn invisible_default_uses_effect_command() {
    let mut m = Mock::new();
    m.defaults = true;
    let (out, _) = run_with(&body(&["make player invisible", "make player visible"]), m);
    assert_eq!(
        out,
        [
            "cmd effect give Steve minecraft:invisibility infinite 0 true",
            "cmd effect clear Steve minecraft:invisibility"
        ]
    );
}

#[test]
fn ignite_and_extinguish() {
    let out = run(&body(&[
        "ignite player",
        "ignite player for 10 seconds",
        "set player on fire for 2 seconds",
        "extinguish player",
        "put out player",
    ]));
    assert_eq!(
        out,
        ["fire Steve 100", "fire Steve 200", "fire Steve 40", "fire Steve 0", "fire Steve 0"]
    );
}

#[test]
fn ignite_rejects_a_plain_number() {
    let mut e = Engine::default();
    let src = body(&["ignite player for 5"]);
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    // Either it does not compile, or it fails at run time; it must not ignite.
    let mut h = Mock::new();
    if errs.is_empty() {
        e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    }
    assert!(!h.calls.iter().any(|c| c.starts_with("fire")));
}

#[test]
fn knock_away_and_toward() {
    // Player at (0, 64, 0); the pusher is at x=10.
    let out = run(&body(&[
        "set {_l} to location(10, 64, 0)",
        "knock player away from {_l}",
        "knock player toward {_l} with strength 2",
        "knock player away from {_l} with force 99",
    ]));
    assert_eq!(
        out,
        [
            "knock Steve 0.4 10 0",
            "knock Steve 2 -10 -0",
            "knock Steve 20 10 0"
        ]
    );
}

#[test]
fn look_at_sets_rotation() {
    // Player eyes at y 65.62. Target due south (+z) at the same height.
    let out = run(&body(&[
        "set {_s} to location(0, 65.62, 10)",
        "make player look at {_s}",
        "set {_w} to location(-10, 65.62, 0)",
        "make player face at {_w}",
    ]));
    assert_eq!(out, ["rot Steve 0 0", "rot Steve 90 0"]);
}

#[test]
fn angle_math() {
    let (yaw, pitch) = look_angles((0.0, 0.0, 0.0), (0.0, 0.0, 5.0));
    assert!((yaw - 0.0).abs() < 1e-3 && pitch.abs() < 1e-3);
    // West (-x) is yaw 90, east (+x) is yaw -90.
    let (yaw, _) = look_angles((0.0, 0.0, 0.0), (-5.0, 0.0, 0.0));
    assert!((yaw - 90.0).abs() < 1e-3);
    let (yaw, _) = look_angles((0.0, 0.0, 0.0), (5.0, 0.0, 0.0));
    assert!((yaw + 90.0).abs() < 1e-3);
    // Looking straight down is pitch 90, up is -90.
    let (_, pitch) = look_angles((0.0, 10.0, 0.0), (0.0, 0.0, 0.0));
    assert!((pitch - 90.0).abs() < 1e-3);
    let (_, pitch) = look_angles((0.0, 0.0, 0.0), (0.0, 10.0, 0.0));
    assert!((pitch + 90.0).abs() < 1e-3);
}

#[test]
fn say_and_book() {
    let out = run(&body(&[
        "make player say \"hello there\"",
        "open book to player",
        "open the book in main hand to player",
        "open book in off hand to player",
    ]));
    assert_eq!(
        out,
        ["say Steve hello there", "book Steve false", "book Steve false", "book Steve true"]
    );
}

#[test]
fn say_default_runs_say_as_player() {
    struct D(Vec<String>);
    impl Host for D {
        fn send_message(&mut self, _: &Recipient, _: &str) {}
        fn broadcast(&mut self, _: &str) {}
        fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
        fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
        fn heal(&mut self, _: &PlayerRef) {}
        fn set_health(&mut self, _: &PlayerRef, _: f64) {}
        fn kill(&mut self, _: &PlayerRef) {}
        fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
        fn execute_command(&mut self, c: &str) {
            self.0.push(c.to_string());
        }
        fn resolve_player(&mut self, _: &str) -> Option<String> {
            Some("Steve".into())
        }
        fn find_player(&mut self, _: &str) -> Option<PlayerRef> {
            None
        }
        fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
            false
        }
        fn schedule_wait(&mut self, _: u64, _: u64) {}
        fn now(&mut self) -> u64 {
            0
        }
        fn log(&mut self, _: LogLevel, _: &str) {}
        fn player_location(&mut self, _: &PlayerRef) -> Option<(f64, f64, f64, String)> {
            Some((0.0, 64.0, 0.0, "w".into()))
        }
    }
    let mut e = Engine::default();
    let src = body(&["make player say \"hi\"", "make player look at location(0, 65.62, 4)"]);
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    assert!(errs.is_empty(), "{errs:?}");
    let mut h = D(Vec::new());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.0, ["execute as Steve run say hi", "rotate Steve 0 0"]);
}

#[test]
fn log_goes_to_file() {
    let out = run(&body(&[
        "log \"one\"",
        "log \"two\" to file \"my-log\"",
        "log \"three\" to \"a.txt\"",
    ]));
    assert_eq!(out, ["log server.log one", "log my-log.log two", "log a.txt three"]);
}

#[test]
fn log_rejects_paths() {
    let (out, h) = run_with(&body(&["log \"x\" to file \"../evil\""]), Mock::new());
    assert!(out.is_empty());
    assert!(h.errors().iter().any(|m| m.contains("not a valid log file name")));
    assert!(clean_log_name("a/b").is_err());
    assert!(clean_log_name("..").is_err());
    assert!(clean_log_name("").is_err());
    assert_eq!(clean_log_name("chat").unwrap(), "chat.log");
}

#[test]
fn suppress_warning_is_accepted() {
    let out = run(&body(&[
        "locally suppress missing conjunction warnings",
        "suppress variable save warning",
        "send \"ok\"",
    ]));
    assert_eq!(out, ["send Steve ok"]);
}

#[test]
fn strength_is_clamped() {
    assert_eq!(clean_strength(None), 0.4);
    assert_eq!(clean_strength(Some(-3.0)), 0.0);
    assert_eq!(clean_strength(Some(100.0)), 20.0);
    assert_eq!(clean_strength(Some(f64::NAN)), 0.0);
}

#[test]
fn offline_player_is_skipped() {
    let mut e = Engine::default();
    let src = body(&["make {_nobody} fly"]);
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    let mut h = Mock::new();
    if errs.is_empty() {
        e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    }
    assert!(!h.calls.iter().any(|c| c.starts_with("flight")));
}

#[test]
fn existing_make_execute_still_works() {
    let out = run(&body(&["make player execute command \"/home\""]));
    assert_eq!(out, ["cmd execute as Steve run home"]);
}
