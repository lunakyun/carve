use carve_core::compile::{EventKind, compile_file};
use carve_core::*;

#[derive(Default)]
struct Mock {
    sent: Vec<String>,
    errors: Vec<String>,
    compass: Option<(f64, f64, f64)>,
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
        if level == LogLevel::Error {
            self.errors.push(msg.to_string());
        }
    }
    fn player_location(&mut self, _p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((10.5, 64.0, -20.0, "world".into()))
    }
    fn compass_target(&mut self, _p: &PlayerRef) -> Option<(f64, f64, f64, Option<String>)> {
        self.compass.map(|(x, y, z)| (x, y, z, Some("world".into())))
    }
    fn set_compass_target(&mut self, _p: &PlayerRef, x: f64, y: f64, z: f64, _w: Option<&str>) {
        self.compass = Some((x, y, z));
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn run_on(event: &str, kind: EventKind, ctx: EventCtx, body: &str) -> (Vec<String>, Mock) {
    let src = format!(
        "on {event}:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    );
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    assert!(
        errs.is_empty(),
        "{}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    let mut h = Mock::default();
    e.fire(kind, ctx, &mut h);
    assert!(h.errors.is_empty(), "{:?}", h.errors);
    (std::mem::take(&mut h.sent), h)
}

fn run(body: &str) -> Vec<String> {
    run_on("join", EventKind::Join, EventCtx::player(steve()), body).0
}

fn one(expr: &str) -> String {
    run(&format!("set {{_x}} to {expr}\nsend \"%{{_x}}%\"")).remove(0)
}

fn fails(expr: &str) -> bool {
    let src = format!("on join:\n    set {{_x}} to {expr}\n");
    compile_file("bad.carve", &src).is_err()
}

#[test]
fn registry_still_valid() {
    carve_core::compile::check_registry().unwrap();
}

#[test]
fn codepoint_of() {
    assert_eq!(one("codepoint of \"A\""), "65");
    assert_eq!(one("unicode of \"a\""), "97");
    assert_eq!(one("code point of \"abc\""), "97");
    assert_eq!(one("codepoint of \"\""), "<none>");
    assert_eq!(one("codepoint of \"€\""), "8364");
}

#[test]
fn character_from_codepoint() {
    assert_eq!(one("character from codepoint 65"), "A");
    assert_eq!(one("character from unicode 98"), "b");
    assert_eq!(one("char from code point 100 + 1"), "e");
    assert_eq!(one("character from codepoint -3"), "<none>");
    assert_eq!(one("character from codepoint 55296"), "<none>");
    assert_eq!(one("character from codepoint 1.5"), "<none>");
}

#[test]
fn characters_between() {
    assert_eq!(
        run("loop characters between \"a\" and \"e\":\n    send loop-value"),
        ["a", "b", "c", "d", "e"]
    );
    assert_eq!(
        run("loop characters between \"c\" and \"a\":\n    send loop-value"),
        ["c", "b", "a"]
    );
    assert_eq!(
        run("loop alphanumeric characters between \"8\" and \"B\":\n    send loop-value"),
        ["8", "9", "A", "B"]
    );
    assert_eq!(
        run("loop all alphanumeric characters between \"~\" and \"{\":\n    send loop-value"),
        Vec::<String>::new()
    );
    assert_eq!(
        run("set {_l::*} to characters between \"a\" and \"d\"\nsend \"%size of {_l::*}%\""),
        ["4"]
    );
}

#[test]
fn difference() {
    assert_eq!(one("difference between 3 and 10"), "7");
    assert_eq!(one("difference between 10 and 3"), "7");
    assert_eq!(one("difference between 2 + 1 and 1"), "2");
    assert_eq!(
        one("difference between (1 minute) and (90 seconds)"),
        "30 seconds"
    );
    assert_eq!(one("difference between \"a\" and 3"), "<none>");
}

#[test]
fn offsets() {
    let out = run(
        "set {_l} to 3 blocks above player's location\nsend \"%x of {_l}% %y of {_l}% %z of {_l}%\"\n\
         set {_n} to 2 blocks north of player's location\nsend \"%z of {_n}%\"\n\
         set {_s} to 1 meter south of player's location\nsend \"%z of {_s}%\"\n\
         set {_e} to 4 blocks east player's location\nsend \"%x of {_e}%\"\n\
         set {_w} to 1 block west of player's location\nsend \"%x of {_w}%\"\n\
         set {_d} to 5 blocks below player's location\nsend \"%y of {_d}%\"",
    );
    assert_eq!(out, ["10.5 67 -20", "-22", "-19", "14.5", "9.5", "59"]);
}

#[test]
fn offset_keeps_other_item_syntax() {
    assert!(!fails("2 diamonds"));
    assert!(fails("2 blocks above 5"));
}

#[test]
fn chunk_numbers() {
    assert_eq!(one("chunk x of player's location"), "0");
    assert_eq!(one("chunk z of player's location"), "-2");
    assert_eq!(one("chunk x of player"), "0");
}

#[test]
fn compass_target() {
    let (out, h) = run_on(
        "join",
        EventKind::Join,
        EventCtx::player(steve()),
        "set compass target of player to 3 blocks above player's location\n\
         send \"%x of compass target of player% %y of player's compass target%\"\n\
         reset player's compass target",
    );
    assert_eq!(out, ["10.5 67"]);
    assert_eq!(h.compass, Some((0.0, 0.0, 0.0)));
}

#[test]
fn death_message() {
    let mut ctx = EventCtx::player(steve());
    ctx.death_message = Some("Steve fell".into());
    let (out, _) = run_on("death", EventKind::Death, ctx, "send death message");
    assert_eq!(out, ["Steve fell"]);
    assert!(compile_file("bad.carve", "on join:\n    send death message\n").is_err());
}

#[test]
fn damage_cause() {
    let mut ctx = EventCtx::player(steve());
    ctx.damage = Some(2.0);
    ctx.damage_cause = Some("fall".into());
    let (out, _) = run_on(
        "damage",
        EventKind::Damage,
        ctx,
        "if damage cause is \"fall\":\n    send \"ouch\"\nsend \"%damage cause%\"",
    );
    assert_eq!(out, ["ouch", "fall"]);
    assert!(compile_file("bad.carve", "on join:\n    send damage cause\n").is_err());
}

#[test]
fn chat_recipients() {
    let mut ctx = EventCtx::chat(steve(), "hi");
    ctx.recipients = Some(vec![
        steve(),
        PlayerRef {
            uuid: "u2".into(),
            name: "Alex".into(),
        },
    ]);
    let (out, _) = run_on("chat", EventKind::Chat, ctx, "send \"%size of chat recipients%\"");
    assert_eq!(out, ["2"]);
    assert!(compile_file("bad.carve", "on join:\n    send chat recipients\n").is_err());
}
