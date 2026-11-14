use std::collections::HashMap;

use carve_core::compile::{EventKind, compile_file};
use carve_core::inventories::InvClick;
use carve_core::*;

/// A fake inventory: slots of the player and of the open menu.
#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    msgs: Vec<String>,
    online: HashMap<String, String>,
    /// (current, slot) -> (id, count)
    slots: HashMap<(bool, i64), (String, u32)>,
    owned: HashMap<String, u32>,
    logs: Vec<(LogLevel, String)>,
}

impl Mock {
    fn steve() -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m
    }
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.msgs.push(text.to_string());
    }
    fn broadcast(&mut self, text: &str) {
        self.msgs.push(text.to_string());
    }
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
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
    fn count_item(&mut self, _p: &PlayerRef, id: &str) -> u32 {
        self.owned.get(id).copied().unwrap_or(0)
    }
    fn give_item(&mut self, _p: &PlayerRef, id: &str, count: u32) {
        self.calls.push(format!("give {id} {count}"));
    }
    fn open_chest(&mut self, _p: &PlayerRef, rows: u8, title: &str) {
        self.calls.push(format!("open {rows} {title}"));
    }
    fn inv_get(&mut self, _p: &PlayerRef, current: bool, slot: i64) -> Option<(String, u32)> {
        self.slots.get(&(current, slot)).cloned()
    }
    fn inv_set(&mut self, _p: &PlayerRef, current: bool, slot: i64, item: Option<(&str, u32)>) {
        self.calls.push(format!("set {current} {slot} {item:?}"));
        match item {
            Some((i, c)) => {
                self.slots.insert((current, slot), (i.to_string(), c));
            }
            None => {
                self.slots.remove(&(current, slot));
            }
        }
    }
    fn inv_clear(&mut self, _p: &PlayerRef, current: bool) {
        self.calls.push(format!("clear {current}"));
    }
    fn inv_remove(&mut self, _p: &PlayerRef, id: &str, count: u32) -> u32 {
        self.calls.push(format!("remove {id} {count}"));
        count
    }
    fn close_inventory(&mut self, _p: &PlayerRef) {
        self.calls.push("close".into());
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

fn script(trigger: &str, body: &str) -> String {
    let lines: String = body.lines().map(|l| format!("    {l}\n")).collect();
    format!("{trigger}:\n{lines}")
}

fn load(trigger: &str, body: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("test.carve".into(), script(trigger, body))]);
    assert!(
        errs.is_empty(),
        "errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn run_with(h: &mut Mock, body: &str) {
    let mut e = load("on join", body);
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errors.is_empty(), "runtime errors: {errors:?}");
}

fn run(body: &str) -> Vec<String> {
    let mut h = Mock::steve();
    run_with(&mut h, body);
    h.calls
}

fn click_ctx(slot: i32, item: Option<(&str, u32)>) -> EventCtx {
    let mut c = EventCtx::player(steve());
    c.inv = Some(Box::new(InvClick {
        slot,
        item: item.map(|(i, n)| (i.to_string(), n)),
        click: "shift left".into(),
    }));
    c
}

#[test]
fn open_chest() {
    assert_eq!(
        run(r#"open chest with 3 rows named "Shop" to player"#),
        ["open 3 Shop"]
    );
    assert_eq!(run("open a chest inventory with 1 row to player"), ["open 1 Chest"]);
    assert_eq!(
        run(r#"open chest with 6 rows titled "&aMenu" for player"#),
        ["open 6 &aMenu"]
    );
    assert_eq!(
        run(r#"open chest with 2 rows with title "Hi" to player"#),
        ["open 2 Hi"]
    );
}

#[test]
fn open_chest_rows_from_variable() {
    assert_eq!(
        run("set {_r} to 4\nopen chest with {_r} rows to player"),
        ["open 4 Chest"]
    );
}

#[test]
fn open_chest_bad_rows() {
    let errs = errors_of(&script("on join", "open chest with 7 rows to player"));
    assert!(errs.iter().any(|e| e.to_string().contains("1 to 6 rows")), "{errs:?}");
    let mut h = Mock::steve();
    let mut e = load("on join", "set {_r} to 9\nopen chest with {_r} rows to player");
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.calls.is_empty());
    assert!(h.logs.iter().any(|(l, _)| *l == LogLevel::Error));
}

#[test]
fn set_slot() {
    assert_eq!(
        run("set slot 4 of player's current inventory to diamond"),
        ["set true 4 Some((\"diamond\", 1))"]
    );
    assert_eq!(
        run("set slot 0 of player's inventory to 5 apples"),
        ["set false 0 Some((\"apple\", 5))"]
    );
    assert_eq!(
        run("set slot 2 of current inventory of player to stone"),
        ["set true 2 Some((\"stone\", 1))"]
    );
}

#[test]
fn set_slot_air_empties() {
    assert_eq!(run("set slot 1 of player's inventory to air"), ["set false 1 None"]);
}

#[test]
fn set_slot_negative_is_an_error() {
    let mut h = Mock::steve();
    let mut e = load("on join", "set slot -1 of player's inventory to stone");
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.calls.is_empty());
    assert!(h.logs.iter().any(|(l, m)| *l == LogLevel::Error && m.contains("out of range")));
}

#[test]
fn read_slot() {
    let mut h = Mock::steve();
    h.slots.insert((true, 3), ("gold_ingot".into(), 7));
    run_with(
        &mut h,
        r#"set {_i} to slot 3 of player's current inventory
broadcast "got %{_i}%"
if slot 3 of player's inventory is not set:
    broadcast "own empty""#,
    );
    assert_eq!(h.msgs.len(), 2, "{:?}", h.msgs);
    assert!(h.msgs[0].starts_with("got "), "{:?}", h.msgs);
    assert!(h.msgs[0].contains("gold ingot") || h.msgs[0].contains("gold_ingot"), "{:?}", h.msgs);
    assert_eq!(h.msgs[1], "own empty");
}

#[test]
fn close_and_clear() {
    assert_eq!(run("close player's inventory"), ["close"]);
    assert_eq!(run("close inventory of player"), ["close"]);
    assert_eq!(run("clear player's inventory"), ["clear false"]);
    assert_eq!(run("clear inventory of player"), ["clear false"]);
    assert_eq!(run("clear player's current inventory"), ["clear true"]);
}

#[test]
fn clear_variable_still_works() {
    let mut h = Mock::steve();
    run_with(
        &mut h,
        "set {x::1} to 5\ndelete {x::*}\nif {x::1} is not set:\n    broadcast \"gone\"",
    );
    assert_eq!(h.msgs, ["gone"]);
    assert!(h.calls.is_empty());
}

#[test]
fn remove_and_add_items() {
    assert_eq!(run("remove 2 diamonds from player"), ["remove diamond 2"]);
    assert_eq!(run("remove 3 stone from player's inventory"), ["remove stone 3"]);
    assert_eq!(run("remove apple from inventory of player"), ["remove apple 1"]);
    assert_eq!(run("add 4 emeralds to player's inventory"), ["give emerald 4"]);
}

#[test]
fn remove_from_lists_is_untouched() {
    let mut h = Mock::steve();
    run_with(
        &mut h,
        "set {l::*} to 1, 2 and 3\nremove 2 from {l::*}\nadd 9 to {n}\nbroadcast \"%size of {l::*}% %{n}%\"",
    );
    assert_eq!(h.msgs, ["2 9"]);
    assert!(h.calls.is_empty());
}

#[test]
fn contains_and_not_contains() {
    let mut h = Mock::steve();
    h.owned.insert("diamond".into(), 3);
    run_with(
        &mut h,
        r#"if player's inventory contains 2 diamonds:
    broadcast "has"
if player's inventory does not contain 5 diamonds:
    broadcast "short"
if player's inventory doesn't contain stone:
    broadcast "nostone"
if player's inventory does not contain diamond:
    broadcast "bad""#,
    );
    assert_eq!(h.msgs, ["has", "short", "nostone"]);
}

#[test]
fn inventory_click_event() {
    let mut e = load(
        "on inventory click",
        r#"broadcast "slot %clicked slot%"
broadcast "item %clicked item%"
broadcast "type %click type%"
if clicked slot is 4:
    cancel event"#,
    );
    let mut h = Mock::steve();
    let out = e.fire(EventKind::InventoryClick, click_ctx(4, Some(("diamond", 2))), &mut h);
    assert!(out.cancelled);
    assert_eq!(h.msgs.len(), 3, "{:?}", h.msgs);
    assert_eq!(h.msgs[0], "slot 4");
    assert!(h.msgs[1].contains("diamond"), "{:?}", h.msgs);
    assert_eq!(h.msgs[2], "type shift left");

    let mut h = Mock::steve();
    let out = e.fire(EventKind::InventoryClick, click_ctx(7, None), &mut h);
    assert!(!out.cancelled);
}

#[test]
fn inventory_click_empty_slot_item_is_unset() {
    let mut e = load(
        "on inventory click",
        "if clicked item is not set:\n    broadcast \"empty\"",
    );
    let mut h = Mock::steve();
    e.fire(EventKind::InventoryClick, click_ctx(0, None), &mut h);
    assert_eq!(h.msgs, ["empty"]);
}

#[test]
fn inventory_close_event_compiles() {
    let mut e = load("on inventory close", "broadcast \"closed\"");
    let mut h = Mock::steve();
    e.fire(EventKind::InventoryClose, EventCtx::player(steve()), &mut h);
    assert_eq!(h.msgs, ["closed"]);
}

#[test]
fn clicked_slot_only_in_inventory_click() {
    let errs = errors_of(&script("on join", "broadcast \"%clicked slot%\""));
    assert!(!errs.is_empty());
}

#[test]
fn cancel_does_nothing_in_close() {
    assert!(!EventKind::InventoryClose.cancellable());
    assert!(EventKind::InventoryClick.cancellable());
}

#[test]
fn unsupported_host_warns_instead_of_crashing() {
    struct Bare(Vec<String>);
    impl Host for Bare {
        fn send_message(&mut self, _: &Recipient, _: &str) {}
        fn broadcast(&mut self, _: &str) {}
        fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
        fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
        fn heal(&mut self, _: &PlayerRef) {}
        fn set_health(&mut self, _: &PlayerRef, _: f64) {}
        fn kill(&mut self, _: &PlayerRef) {}
        fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
        fn execute_command(&mut self, _: &str) {}
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
        fn log(&mut self, level: LogLevel, msg: &str) {
            if level == LogLevel::Warn {
                self.0.push(msg.to_string());
            }
        }
    }
    let mut e = load(
        "on join",
        "open chest with 1 row to player\nclose player's inventory\nclear player's inventory",
    );
    let mut h = Bare(Vec::new());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.0.len(), 3, "{:?}", h.0);
}
