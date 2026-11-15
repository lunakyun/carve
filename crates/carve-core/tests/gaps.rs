//! Inventory values, held item editing and the smaller Skript gaps
//! (`sorted ... by`, timespan parts, regex replace, quit message, mob
//! death, spawn named, push forward/away, velocity vectors).

use std::collections::HashMap;

use carve_core::combat::EntSlot;
use carve_core::compile::EventKind;
use carve_core::inventories::InvClick;
use carve_core::item_meta::ItemMeta;
use carve_core::value::{EntityRef, Value};
use carve_core::world_effects::FxPos;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    msgs: Vec<String>,
    online: HashMap<String, String>,
    slots: HashMap<(bool, i64), (String, u32)>,
    held: Option<(String, u32)>,
    held_meta: Option<ItemMeta>,
    logs: Vec<(LogLevel, String)>,
    vel: HashMap<String, (f64, f64, f64)>,
    ents: u32,
    set_meta: Option<ItemMeta>,
    quit: Option<String>,
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
    fn open_chest(&mut self, _p: &PlayerRef, rows: u8, title: &str) {
        self.calls.push(format!("open {rows} {title}"));
    }
    fn inv_get(&mut self, _p: &PlayerRef, current: bool, slot: i64) -> Option<(String, u32)> {
        self.slots.get(&(current, slot)).cloned()
    }
    fn inv_set(&mut self, _p: &PlayerRef, current: bool, slot: i64, item: Option<(&str, u32)>) {
        self.calls.push(format!("set {current} {slot} {item:?}"));
    }
    fn held_item(&mut self, _p: &PlayerRef) -> Option<(String, u32)> {
        self.held.clone()
    }
    fn held_item_meta(&mut self, _p: &PlayerRef) -> Option<ItemMeta> {
        self.held_meta.clone()
    }
    fn player_location(&mut self, _p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((0.0, 64.0, 0.0, "world".into()))
    }
    fn spawn_entity(&mut self, kind: &str, _at: &FxPos) -> Option<EntityRef> {
        self.ents += 1;
        self.calls.push(format!("spawn {kind}"));
        Some(EntityRef {
            uuid: format!("e{}", self.ents),
            kind: kind.to_string(),
        })
    }
    fn set_entity_name(&mut self, e: &EntityRef, name: Option<&str>) {
        self.calls.push(format!("name {} {name:?}", e.uuid));
    }
    fn entity_location(&mut self, e: &EntityRef) -> Option<(f64, f64, f64, String)> {
        (e.uuid != "u1").then(|| (3.0, 64.0, 4.0, "world".into()))
    }
    fn entity_velocity(&mut self, e: &EntityRef) -> Option<(f64, f64, f64)> {
        Some(self.vel.get(&e.uuid).copied().unwrap_or((0.0, 0.0, 0.0)))
    }
    fn supports_particle_data(&mut self) -> bool {
        true
    }
    fn spawn_particle(&mut self, p: &carve_core::world_effects::ParticleFx) {
        self.calls.push(format!("{p:?}"));
    }
    fn menu_update(&mut self, id: u64, slot: u32, item: &Value) {
        self.calls.push(format!("update {} {slot} {item}", id != 0));
    }
    fn set_held_item_meta(&mut self, _p: &PlayerRef, meta: &ItemMeta) -> bool {
        self.set_meta = Some(meta.clone());
        true
    }
    fn entity_rotation(&mut self, _e: &EntityRef) -> Option<(f64, f64)> {
        Some((90.0, 0.0))
    }
    fn set_quit_message(&mut self, text: &str) {
        self.quit = Some(text.to_string());
    }
    fn set_entity_velocity(&mut self, e: &EntityRef, v: (f64, f64, f64)) {
        self.calls
            .push(format!("vel {} {:.2} {:.2} {:.2}", e.uuid, v.0, v.1, v.2));
        self.vel.insert(e.uuid.clone(), v);
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn script(trigger: &str, body: &str) -> String {
    let lines: String = body.lines().map(|l| format!("    {l}\n")).collect();
    format!("{trigger}:\n{lines}")
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

fn no_errors(h: &Mock) {
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errors.is_empty(), "runtime errors: {errors:?}");
}

fn run_with(h: &mut Mock, body: &str) {
    let mut e = load_src(&script("on join", body));
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
    no_errors(h);
}

fn msgs(body: &str) -> Vec<String> {
    let mut h = Mock::steve();
    run_with(&mut h, body);
    h.msgs
}

fn compile_err(src: &str) -> String {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("bad.carve".into(), src.to_string())]);
    errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
}

// ---------- already there, checked here so the parity rows stay true ----------

#[test]
fn do_if_inline() {
    assert_eq!(msgs("send \"yes\" if 1 is 1\nsend \"no\" if 1 is 2"), vec!["yes"]);
}

#[test]
fn filter_section_and_event_cancelled() {
    let out = msgs(
        "set {_l::*} to 1, 5 and 9\nfilter {_l::*} to match:\n    input > 2\nsend \"%{_l::*}%\"\nif event is not cancelled:\n    send \"live\"\ncancel event\nif event is cancelled:\n    send \"gone\"",
    );
    assert_eq!(out, vec!["5 and 9", "live", "gone"]);
}

#[test]
fn rgb_in_text() {
    assert_eq!(msgs("send \"<%rgb(255, 0, 0)%>hi\""), vec!["<#ff0000>hi"]);
}

// ---------- inventory values ----------

const SHOP: &str = "set {_gui} to chest inventory with 3 rows named \"Shop\"\nset slot 4 of {_gui} to diamond named \"Buy\"\nset slot 5 of {_gui} to 2 emeralds";

#[test]
fn inventory_value_build_and_read() {
    let out = msgs(&format!(
        "{SHOP}\nsend name of {{_gui}}\nsend \"%slot 4 of {{_gui}}%\"\nsend \"%items in {{_gui}}%\"\nsend \"%size of items in {{_gui}}%\"\nif {{_gui}} contains diamond:\n    send \"has\"\nif {{_gui}} contains 3 emeralds:\n    send \"bad\"\nif {{_gui}} doesn't contain stone:\n    send \"no stone\"\nsend \"%{{_gui}}%\""
    ));
    assert_eq!(
        out,
        vec![
            "Shop",
            "diamond named \"Buy\"",
            "diamond named \"Buy\" and 2 emerald",
            "2",
            "has",
            "no stone",
            "chest inventory named \"Shop\""
        ]
    );
}

#[test]
fn inventory_other_forms() {
    let out = msgs(
        "set {_a} to a new chest inventory named \"A\" with 1 row\nset {_b} to chest inventory\nsend \"%name of {_a}% %name of {_b}%\"\nset {_c} to {_a}\nif {_c} is {_a}:\n    send \"same\"\nif {_a} is not {_b}:\n    send \"different\"",
    );
    assert_eq!(out, vec!["A Chest", "same", "different"]);
}

#[test]
fn inventory_open_and_live_update() {
    let mut h = Mock::steve();
    run_with(
        &mut h,
        &format!(
            "{SHOP}\nopen {{_gui}} to player\nset slot 0 of {{_gui}} to stone\nclear {{_gui}}\nsend \"%size of items in {{_gui}}%\""
        ),
    );
    assert!(h.calls.contains(&"open 3 Shop".to_string()), "{:?}", h.calls);
    assert!(
        h.calls.contains(&"set true 4 Some((\"diamond\", 1))".to_string()),
        "{:?}",
        h.calls
    );
    assert!(
        h.calls.contains(&"update true 0 stone".to_string()),
        "{:?}",
        h.calls
    );
    assert!(
        h.calls.contains(&"update true 26 <none>".to_string()),
        "{:?}",
        h.calls
    );
    assert_eq!(h.msgs, vec!["0"]);
}

#[test]
fn inventory_bad_rows_and_slots() {
    let mut h = Mock::steve();
    let mut e = load_src(&script("on join", "set {_g} to chest inventory with 9 rows"));
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(
        h.logs.iter().any(|(_, m)| m.contains("1 to 6 rows")),
        "{:?}",
        h.logs
    );
    let mut h = Mock::steve();
    let mut e = load_src(&script(
        "on join",
        "set {_g} to chest inventory with 1 row\nset slot 20 of {_g} to stone",
    ));
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(
        h.logs.iter().any(|(_, m)| m.contains("outside this inventory")),
        "{:?}",
        h.logs
    );
}

#[test]
fn clear_deletes_a_plain_variable() {
    assert_eq!(msgs("set {_x} to 5\nclear {_x}\nsend \"%{_x}%\""), vec!["<none>"]);
}

#[test]
fn inventory_saved_in_global_variable() {
    let mut h = Mock::steve();
    let mut e = load_src(&script("on join", SHOP.replace("{_gui}", "{shop}").as_str()));
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let json = e.vars.to_json();
    assert!(json.contains("\"inventory\""), "{json}");
    let vars = Variables::from_json(&json).unwrap();
    let mut e2 = Engine::new(vars);
    let (_, errs) = e2.load(&[(
        "t.carve".into(),
        script(
            "on join",
            "send \"%name of {shop}% %slot 4 of {shop}% %slot 5 of {shop}%\"",
        ),
    )]);
    assert!(errs.is_empty());
    let mut h2 = Mock::steve();
    e2.fire(EventKind::Join, EventCtx::player(steve()), &mut h2);
    assert_eq!(h2.msgs, vec!["Shop diamond named \"Buy\" 2 emerald"]);
}

fn click(slot: i32, menu: Option<(u64, &str, u8)>) -> EventCtx {
    let mut c = EventCtx::player(steve());
    c.inv = Some(Box::new(InvClick {
        slot,
        item: None,
        click: "left".into(),
        menu: menu.map(|(i, t, r)| (i, t.to_string(), r)),
    }));
    c
}

#[test]
fn event_inventory_in_click() {
    let body = "if name of event-inventory is \"Shop\":\n    send \"shop %clicked slot%\"\nif clicked inventory is not set:\n    send \"below\"\nsend \"%slot 2 of event-inventory%\"\nset slot 1 of event-inventory to stone\nif event-inventory contains gold ingot:\n    send \"gold\"";
    let mut e = load_src(&script("on inventory click", body));
    let mut h = Mock::steve();
    h.slots.insert((true, 2), ("gold_ingot".into(), 1));
    e.fire(EventKind::InventoryClick, click(3, Some((7, "Shop", 3))), &mut h);
    no_errors(&h);
    assert_eq!(h.msgs, vec!["shop 3", "gold ingot", "gold"]);
    assert!(
        h.calls.contains(&"set true 1 Some((\"stone\", 1))".to_string()),
        "{:?}",
        h.calls
    );
    let mut h = Mock::steve();
    e.fire(EventKind::InventoryClick, click(40, Some((7, "Shop", 3))), &mut h);
    assert_eq!(h.msgs, vec!["shop 40", "below", "<none>"]);
    // Not a Carve menu: no event-inventory.
    let mut h = Mock::steve();
    e.fire(EventKind::InventoryClick, click(1, None), &mut h);
    assert_eq!(h.msgs, vec!["below", "<none>"]);
}

#[test]
fn event_inventory_compares_with_stored_menu() {
    let src = "on join:\n    set {shop} to chest inventory named \"Shop\"\non inventory click:\n    if event-inventory is {shop}:\n        send \"it is the shop\"\n";
    let mut e = load_src(src);
    let mut h = Mock::steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let id = match e.vars.get("shop") {
        Value::Inventory(i) => i.id,
        other => panic!("{other:?}"),
    };
    e.fire(EventKind::InventoryClick, click(0, Some((id, "Shop", 3))), &mut h);
    e.fire(
        EventKind::InventoryClick,
        click(0, Some((id + 1, "Shop", 3))),
        &mut h,
    );
    assert_eq!(h.msgs, vec!["it is the shop"]);
}

#[test]
fn items_in_player_inventory() {
    let mut h = Mock::steve();
    h.slots.insert((false, 0), ("diamond".into(), 3));
    h.slots.insert((false, 8), ("stone".into(), 1));
    run_with(
        &mut h,
        "send \"%items in player's inventory%\"\nloop all items in inventory of player:\n    send \"%loop-value%\"",
    );
    assert_eq!(h.msgs, vec!["3 diamond and stone", "3 diamond", "stone"]);
}

// ---------- held item ----------

fn held_mock() -> Mock {
    let mut h = Mock::steve();
    h.held = Some(("diamond_sword".into(), 1));
    h.held_meta = Some(ItemMeta {
        name: Some("Old".into()),
        lore: vec!["one".into()],
        enchants: vec![("unbreaking".into(), 1)],
        ..ItemMeta::default()
    });
    h
}

fn held_after(body: &str) -> ItemMeta {
    let mut h = held_mock();
    run_with(&mut h, body);
    h.set_meta.expect("no change sent")
}

#[test]
fn held_item_name_and_lore() {
    assert_eq!(
        held_after("set name of player's tool to \"&aSword\"")
            .name
            .as_deref(),
        Some("&aSword")
    );
    assert_eq!(held_after("reset the name of player's held item").name, None);
    assert_eq!(
        held_after("set lore of tool of player to \"a\" and \"b\"").lore,
        vec!["a", "b"]
    );
    assert_eq!(
        held_after("add \"two\" to lore of player's tool").lore,
        vec!["one", "two"]
    );
    assert!(held_after("clear lore of player's tool").lore.is_empty());
    assert_eq!(
        held_after("set {_l::*} to \"x\" and \"y\"\nset lore of player's tool to {_l::*}").lore,
        vec!["x", "y"]
    );
}

#[test]
fn held_item_enchant() {
    let m = held_after("enchant player's tool with sharpness 5 and unbreaking 3");
    assert_eq!(
        m.enchants,
        vec![("unbreaking".into(), 3), ("sharpness".into(), 5)]
    );
    assert!(held_after("disenchant player's tool").enchants.is_empty());
    assert_eq!(
        held_after("enchant player's tool with mending").level_of("mending"),
        1
    );
}

#[test]
fn held_item_empty_hand_does_nothing() {
    let mut h = Mock::steve();
    run_with(&mut h, "set name of player's tool to \"x\"");
    assert!(h.set_meta.is_none());
}

// ---------- lists, timespans, regex ----------

#[test]
fn sorted_by_expression() {
    let out = msgs(
        "set {_l::*} to \"ccc\", \"a\" and \"bb\"\nsend \"%sorted {_l::*} by length of input%\"\nset {_n::*} to 3, -5 and 1\nset {_s::*} to sorted {_n::*} by input * input\nsend \"%{_s::*}%\"",
    );
    assert_eq!(out, vec!["a, bb and ccc", "1, 3 and -5"]);
}

#[test]
fn timespan_possessive_parts() {
    let out = msgs(
        "set {_t} to 2 minutes and 30 seconds\nsend \"%{_t}'s minutes% %{_t}'s seconds% %{_t}'s hours%\"",
    );
    assert_eq!(out, vec!["2 30 0"]);
}

#[test]
fn regex_replace() {
    let out = msgs(
        "set {_x} to \"a1b22c333\"\nreplace all regex \"[0-9]+\" in {_x} with \"#\"\nsend {_x}\nset {_y} to \"aaa\"\nreplace first regex \"a\" with \"b\" in {_y}\nsend {_y}\nsend \"x1y2\" with all regex \"\\d\" replaced with \"_\"\nset {_z} to \"Hi  there   you\"\nregex replace \" +\" in {_z} with \" \"\nsend {_z}",
    );
    assert_eq!(out, vec!["a#b#c#", "baa", "x_y_", "Hi there you"]);
}

#[test]
fn regex_bad_pattern_is_a_runtime_error() {
    let mut h = Mock::steve();
    let mut e = load_src(&script(
        "on join",
        "set {_x} to \"a\"\nreplace all regex \"(\" in {_x} with \"b\"",
    ));
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(
        h.logs.iter().any(|(_, m)| m.contains("invalid regex")),
        "{:?}",
        h.logs
    );
}

// ---------- quit message ----------

#[test]
fn set_quit_message() {
    let mut e = load_src("on quit:\n    set quit message to \"bye %player%\"\n    send quit message\n");
    let mut h = Mock::steve();
    let mut ctx = EventCtx::player(steve());
    ctx.message = Some("Steve left".into());
    e.fire(EventKind::Quit, ctx, &mut h);
    no_errors(&h);
    assert_eq!(h.quit.as_deref(), Some("bye Steve"));
    assert_eq!(h.msgs, vec!["bye Steve"]);
    assert!(compile_err("on join:\n    set quit message to \"x\"\n").contains("only be set in 'on quit'"));
}

// ---------- entities ----------

#[test]
fn spawn_named() {
    let mut h = Mock::steve();
    run_with(
        &mut h,
        "spawn a zombie named \"Bob\" at player\nspawn 2 pigs at player named \"P\"",
    );
    assert_eq!(
        h.calls,
        vec![
            "spawn zombie",
            "name e1 Some(\"Bob\")",
            "spawn pig",
            "name e2 Some(\"P\")",
            "spawn pig",
            "name e3 Some(\"P\")"
        ]
    );
}

#[test]
fn push_forward_backward_and_away() {
    let mut h = Mock::steve();
    // yaw 90: facing -x.
    run_with(
        &mut h,
        "push player forward with speed 2\npush player backwards\nspawn a pig at player\npush last spawned entity away from player with speed 5\npush last spawned entity towards location(3, 64, 0)",
    );
    assert!(
        h.calls.contains(&"vel u1 -2.00 0.00 0.00".to_string()),
        "{:?}",
        h.calls
    );
    assert!(
        h.calls.contains(&"vel u1 -1.00 0.00 0.00".to_string()),
        "{:?}",
        h.calls
    );
    // The pig is at (3, 64, 4), the player at (0, 64, 0): away is (0.6, 0, 0.8).
    assert!(
        h.calls.contains(&"vel e1 3.00 0.00 4.00".to_string()),
        "{:?}",
        h.calls
    );
    assert!(
        h.calls.contains(&"vel e1 3.00 0.00 3.00".to_string()),
        "{:?}",
        h.calls
    );
}

#[test]
fn set_velocity_to_vector() {
    let mut h = Mock::steve();
    run_with(&mut h, "set velocity of player to vector(0, 1, 0.5)");
    assert_eq!(h.calls, vec!["vel u1 0.00 1.00 0.50"]);
    assert!(compile_err("on join:\n    set velocity of player to 5\n").contains("must be a vector"));
}

#[test]
fn mob_death_trigger() {
    let src = "on death of zombie:\n    send \"zombie died\" to console\non death:\n    send \"%victim% died\" to console\non death of player:\n    send \"player died\" to console\n";
    let mut e = load_src(src);
    let mut h = Mock::steve();
    let zombie = EntityRef {
        uuid: "z1".into(),
        kind: "zombie".into(),
    };
    e.fire(
        EventKind::Death,
        EventCtx::entity_victim(EntSlot::Ent(zombie)),
        &mut h,
    );
    no_errors(&h);
    assert_eq!(h.msgs, vec!["zombie died", "zombie died"]);
    h.msgs.clear();
    e.fire(EventKind::Death, EventCtx::player(steve()), &mut h);
    assert_eq!(h.msgs, vec!["Steve died", "player died"]);
}

#[test]
fn rgb_color_in_particles() {
    let mut h = Mock::steve();
    run_with(
        &mut h,
        "play dust particles at player with color rgb(0, 255, 0)\nset {_c} to rgb(0, 0, 255)\nplay dust particles at player with color {_c}",
    );
    assert_eq!(h.calls.len(), 2, "{:?}", h.calls);
    assert!(h.calls[0].contains("[0, 255, 0]"), "{:?}", h.calls);
    assert!(h.calls[1].contains("[0, 0, 255]"), "{:?}", h.calls);
}
