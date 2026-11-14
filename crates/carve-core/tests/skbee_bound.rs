use std::collections::HashMap;

use carve_core::compile::{EventKind, compile_file};
use carve_core::skbee_bound::Bounds;
use carve_core::*;

/// A host with players at set positions. `broadcast` lines are recorded.
#[derive(Default)]
struct Mock {
    said: Vec<String>,
    online: HashMap<String, String>,
    pos: HashMap<String, (f64, f64, f64, String)>,
    clock: u64,
    warns: Vec<String>,
}

impl Mock {
    fn steve_at(x: f64, y: f64, z: f64) -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m.pos.insert("u1".into(), (x, y, z, "world".into()));
        m
    }
    fn move_to(&mut self, x: f64, y: f64, z: f64) {
        self.pos.insert("u1".into(), (x, y, z, "world".into()));
    }
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.said.push(text.to_string());
    }
    fn broadcast(&mut self, text: &str) {
        self.said.push(text.to_string());
    }
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
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        self.online
            .iter()
            .find(|(_, n)| n.eq_ignore_ascii_case(name))
            .map(|(u, n)| PlayerRef {
                uuid: u.clone(),
                name: n.clone(),
            })
    }
    fn has_permission(&mut self, _p: &PlayerRef, _n: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _t: u64, _k: u64) {}
    fn now(&mut self) -> u64 {
        self.clock
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        if level == LogLevel::Warn {
            self.warns.push(msg.to_string());
        }
    }
    fn online_players(&mut self) -> Vec<PlayerRef> {
        let mut v: Vec<PlayerRef> = self
            .online
            .iter()
            .map(|(u, n)| PlayerRef {
                uuid: u.clone(),
                name: n.clone(),
            })
            .collect();
        v.sort_by(|a, b| a.uuid.cmp(&b.uuid));
        v
    }
    fn player_location(&mut self, p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        self.pos.get(&p.uuid).cloned()
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn errors_of(src: &str) -> Vec<ScriptError> {
    compile_file("bad.carve", src).err().unwrap_or_default()
}

fn script(body: &str) -> String {
    let lines: String = body.lines().map(|l| format!("    {l}\n")).collect();
    format!("on join:\n{lines}")
}

fn load_src(src: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("test.carve".into(), src.to_string())]);
    assert!(
        errs.is_empty(),
        "errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn load(body: &str) -> Engine {
    load_src(&script(body))
}

fn run(e: &mut Engine, host: &mut Mock) {
    e.fire(EventKind::Join, EventCtx::player(steve()), host);
}

const MAKE: &str = "create bound with id \"spawn\" within location(0, 60, 0) and location(10, 70, 10)";

fn made() -> Engine {
    let mut e = load(MAKE);
    let mut h = Mock::steve_at(5.0, 64.0, 5.0);
    run(&mut e, &mut h);
    e
}

fn say(body: &str, h: &mut Mock) -> Vec<String> {
    let mut e = load(&format!("{MAKE}\n{body}"));
    run(&mut e, h);
    std::mem::take(&mut h.said)
}

#[test]
fn create_and_lookup() {
    let e = made();
    let b = e.bounds.get("SPAWN").unwrap();
    assert_eq!(b.min, [0, 60, 0]);
    assert_eq!(b.max, [10, 70, 10]);
    assert_eq!(b.world, "world");
}

#[test]
fn corners_in_any_order() {
    let mut e = load("create bound with id \"a\" within location(10, 70, 10) and location(0, 60, 0)");
    run(&mut e, &mut Mock::steve_at(0.0, 0.0, 0.0));
    let b = e.bounds.get("a").unwrap();
    assert_eq!((b.min, b.max), ([0, 60, 0], [10, 70, 10]));
}

#[test]
fn within_conditions() {
    let mut h = Mock::steve_at(5.5, 64.0, 5.5);
    let out = say(
        "if player is within bound \"spawn\":\n    broadcast \"in\"\nif location(11, 64, 5) is within bound \"spawn\":\n    broadcast \"bad\"\nif location(10.9, 70.9, 0) is within bound \"spawn\":\n    broadcast \"edge\"\nif player is not within bound \"spawn\":\n    broadcast \"bad2\"\nif player is in bound \"spawn\":\n    broadcast \"in2\"",
        &mut h,
    );
    assert_eq!(out, ["in", "edge", "in2"]);
}

#[test]
fn bound_at_location() {
    let mut h = Mock::steve_at(5.0, 64.0, 5.0);
    let out = say(
        "set {_b} to bound at player's location\nbroadcast \"%{_b}%\"\nset {_c} to bound at location(50, 64, 50)\nif {_c} is not set:\n    broadcast \"none\"\nif bound at location(1, 61, 1) exists:\n    broadcast \"exists\"\nif bound at location(99, 61, 1) does not exist:\n    broadcast \"gone\"",
        &mut h,
    );
    assert_eq!(out, ["spawn", "none", "exists", "gone"]);
}

#[test]
fn bound_exists_and_delete() {
    let mut h = Mock::steve_at(5.0, 64.0, 5.0);
    let out = say(
        "if bound \"spawn\" exists:\n    broadcast \"yes\"\ndelete bound \"spawn\"\nif bound \"spawn\" does not exist:\n    broadcast \"deleted\"\nif bound with id \"nope\" doesn't exist:\n    broadcast \"never\"",
        &mut h,
    );
    assert_eq!(out, ["yes", "deleted", "never"]);
}

#[test]
fn owners_and_members() {
    let mut h = Mock::steve_at(5.0, 64.0, 5.0);
    let out = say(
        "add player to owners of bound \"spawn\"\nif player is an owner of bound \"spawn\":\n    broadcast \"owner\"\nif player is not a member of bound \"spawn\":\n    broadcast \"not member\"\nadd player to members of bound \"spawn\"\nloop members of bound \"spawn\":\n    broadcast \"m %loop-value%\"\nremove player from owners of bound \"spawn\"\nif player is not an owner of bound \"spawn\":\n    broadcast \"no longer\"\nset owner of bound \"spawn\" to player\nset {_o::*} to owners of bound \"spawn\"\nbroadcast \"%size of {_o::*}%\"",
        &mut h,
    );
    assert_eq!(out, ["owner", "not member", "m Steve", "no longer", "1"]);
}

#[test]
fn owners_not_duplicated() {
    let mut e = load(&format!(
        "{MAKE}\nadd player to owners of bound \"spawn\"\nadd player to owners of bound \"spawn\""
    ));
    run(&mut e, &mut Mock::steve_at(0.0, 0.0, 0.0));
    assert_eq!(e.bounds.get("spawn").unwrap().owners.len(), 1);
}

#[test]
fn flags() {
    let mut h = Mock::steve_at(5.0, 64.0, 5.0);
    let out = say(
        "if bound \"spawn\" has flag \"pvp\":\n    broadcast \"bad\"\nset flag \"pvp\" of bound \"spawn\" to true\nif bound \"spawn\" has flag \"pvp\":\n    broadcast \"pvp on\"\nbroadcast \"%flag \"pvp\" of bound \"spawn\"%\"\nset flag \"pvp\" of bound \"spawn\" to false\nif bound \"spawn\" does not have flag \"pvp\":\n    broadcast \"pvp off\"\nset flag \"pvp\" of bound \"spawn\" to true\ndelete flag \"pvp\" of bound \"spawn\"\nbroadcast \"%flag \"pvp\" of bound \"spawn\"%\"",
        &mut h,
    );
    assert_eq!(out, ["pvp on", "true", "pvp off", "false"]);
}

#[test]
fn corners_world_volume() {
    let mut h = Mock::steve_at(5.0, 64.0, 5.0);
    let out = say(
        "broadcast \"%lesser corner of bound \"spawn\"%\"\nbroadcast \"%x of greater corner of bound \"spawn\"%\"\nbroadcast \"%world of bound \"spawn\"%\"\nbroadcast \"%volume of bound \"spawn\"%\"\nset greater corner of bound \"spawn\" to location(20, 70, 10)\nbroadcast \"%x of greater corner of bound \"spawn\"%\"",
        &mut h,
    );
    assert_eq!(out, ["x: 0, y: 60, z: 0 in 'world'", "10", "world", "1331", "20"]);
}

#[test]
fn all_bounds_and_bounds_at() {
    let mut h = Mock::steve_at(5.0, 64.0, 5.0);
    let out = say(
        "create bound with id \"inner\" within location(4, 60, 4) and location(6, 70, 6)\nloop bounds at player's location:\n    broadcast \"at %loop-value%\"\nloop all bounds:\n    broadcast \"all %loop-value%\"",
        &mut h,
    );
    assert_eq!(out, ["at spawn", "at inner", "all spawn", "all inner"]);
}

#[test]
fn temporary_is_not_saved() {
    let mut e = load(&format!(
        "{MAKE}\ncreate temporary bound with id \"tmp\" within location(0, 0, 0) and location(1, 1, 1)"
    ));
    run(&mut e, &mut Mock::steve_at(0.0, 0.0, 0.0));
    let json = e.bounds.save_now().unwrap();
    assert!(json.contains("spawn"));
    assert!(!json.contains("tmp"));
}

#[test]
fn save_and_load_round_trip() {
    let mut e = load(&format!(
        "{MAKE}\nadd player to owners of bound \"spawn\"\nset flag \"pvp\" of bound \"spawn\" to true"
    ));
    let mut h = Mock::steve_at(5.0, 64.0, 5.0);
    run(&mut e, &mut h);
    assert!(e.bounds.save_due(1000).is_none(), "debounce");
    let json = e.bounds.save_due(10_000).unwrap();
    let b = Bounds::from_json(&json).unwrap();
    let s = b.get("spawn").unwrap();
    assert_eq!(s.owners, vec![steve()]);
    assert_eq!(s.flags.get("pvp"), Some(&true));
    assert_eq!(b.ids_at("world", 5.0, 64.0, 5.0), ["spawn"]);
    assert!(Bounds::from_json("{").is_err());
    assert!(Bounds::from_json("{\"bounds\":[{\"id\":\"x\"}]}").is_err());
}

#[test]
fn enter_and_exit_events() {
    let src = format!(
        "{}on bound enter:\n    broadcast \"enter %event-bound% %player%\"\non bound exit:\n    broadcast \"exit %event-bound% %player%\"\n",
        script(MAKE)
    );
    let mut e = load_src(&src);
    let mut h = Mock::steve_at(50.0, 64.0, 50.0);
    run(&mut e, &mut h);
    assert!(e.bounds_wanted());
    e.tick(2, &mut h);
    assert!(h.said.is_empty());
    h.move_to(5.0, 64.0, 5.0);
    e.tick(4, &mut h);
    assert_eq!(h.said, ["enter spawn Steve"]);
    e.tick(6, &mut h);
    assert_eq!(h.said.len(), 1);
    h.move_to(5.5, 64.0, 5.5);
    e.tick(8, &mut h);
    assert_eq!(h.said.len(), 1);
    h.move_to(50.0, 64.0, 50.0);
    e.tick(10, &mut h);
    assert_eq!(h.said, ["enter spawn Steve", "exit spawn Steve"]);
}

#[test]
fn poll_runs_only_on_even_ticks() {
    let src = format!("{}on bound enter:\n    broadcast \"enter\"\n", script(MAKE));
    let mut e = load_src(&src);
    let mut h = Mock::steve_at(5.0, 64.0, 5.0);
    run(&mut e, &mut h);
    e.tick(3, &mut h);
    assert!(h.said.is_empty());
    e.tick(4, &mut h);
    assert_eq!(h.said, ["enter"]);
}

#[test]
fn moved_api_fires_without_poll() {
    let src = format!("{}on bound enter:\n    broadcast \"enter %event-bound%\"\n", script(MAKE));
    let mut e = load_src(&src);
    let mut h = Mock::steve_at(0.0, 0.0, 0.0);
    run(&mut e, &mut h);
    e.bounds_moved(&steve(), 1.0, 61.0, 1.0, "world", &mut h);
    e.bounds_moved(&steve(), 1.2, 61.0, 1.2, "world", &mut h);
    assert_eq!(h.said, ["enter spawn"]);
}

#[test]
fn overlapping_bounds_and_other_world() {
    let src = format!(
        "{}on bound enter:\n    broadcast \"enter %event-bound%\"\n",
        script(&format!(
            "{MAKE}\ncreate bound with id \"inner\" within location(4, 60, 4) and location(6, 70, 6)"
        ))
    );
    let mut e = load_src(&src);
    let mut h = Mock::steve_at(5.0, 64.0, 5.0);
    run(&mut e, &mut h);
    e.tick(2, &mut h);
    assert_eq!(h.said, ["enter spawn", "enter inner"]);
    h.said.clear();
    h.pos.insert("u1".into(), (5.0, 64.0, 5.0, "nether".into()));
    e.tick(4, &mut h);
    assert!(h.said.is_empty());
}

#[test]
fn deleted_bound_fires_no_exit() {
    let src = format!(
        "{}on bound exit:\n    broadcast \"exit\"\ncommand /x:\n    trigger:\n        delete bound \"spawn\"\n",
        script(MAKE)
    );
    let mut e = load_src(&src);
    let mut h = Mock::steve_at(5.0, 64.0, 5.0);
    run(&mut e, &mut h);
    e.tick(2, &mut h);
    e.run_command("x", Recipient::Console, "", &mut h);
    h.move_to(50.0, 64.0, 5.0);
    e.tick(4, &mut h);
    assert!(h.said.is_empty());
}

#[test]
fn big_bound_uses_fallback_list() {
    let mut e =
        load("create bound with id \"huge\" within location(-100000, 0, -100000) and location(100000, 255, 100000)");
    run(&mut e, &mut Mock::steve_at(0.0, 0.0, 0.0));
    assert_eq!(e.bounds.ids_at("world", 99999.0, 5.0, -5000.0), ["huge"]);
    assert!(e.bounds.ids_at("world", 100001.0, 5.0, 0.0).is_empty());
}

#[test]
fn same_id_replaces() {
    let mut e = load(&format!(
        "{MAKE}\ncreate bound with id \"SPAWN\" within location(0, 0, 0) and location(1, 1, 1)"
    ));
    run(&mut e, &mut Mock::steve_at(0.0, 0.0, 0.0));
    assert_eq!(e.bounds.len(), 1);
    assert_eq!(e.bounds.get("spawn").unwrap().max, [1, 1, 1]);
}

#[test]
fn missing_bound_warns() {
    let mut e = load("add player to owners of bound \"ghost\"\ndelete bound \"ghost\"");
    let mut h = Mock::steve_at(0.0, 0.0, 0.0);
    run(&mut e, &mut h);
    assert_eq!(h.warns.len(), 2);
}

#[test]
fn compile_errors() {
    assert!(!errors_of(&script("create bound with id \"a\" within 5 and 6")).is_empty());
    assert!(!errors_of(&script("broadcast \"%event-bound%\"")).is_empty());
    assert!(errors_of("on bound enter:\n    broadcast \"%event-bound%\"\n").is_empty());
    assert!(
        errors_of("on player enter bound:\n    broadcast \"x\"\non player exit of bound:\n    broadcast \"x\"\n")
            .is_empty()
    );
}

#[test]
fn unrelated_lines_still_work() {
    let mut e = load("broadcast \"a bound is a box\"\nset {_x} to 5\nbroadcast \"%{_x}%\"");
    let mut h = Mock::steve_at(0.0, 0.0, 0.0);
    run(&mut e, &mut h);
    assert_eq!(h.said, ["a bound is a box", "5"]);
}
