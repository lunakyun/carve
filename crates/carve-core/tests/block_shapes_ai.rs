//! Wave 5, chunk 4: block spheres and lines, books, world border
//! settings, item damage, ender chests, AI, attributes,
//! enchanting offers, inventory actions and event values.

use std::collections::HashMap;

use carve_core::block_shapes_ai::{self as c4, BookData, BorderField, Q};
use carve_core::combat::EntSlot;
use carve_core::compile::EventKind;
use carve_core::events_blocks_players::ExtEvent;
use carve_core::events_game::{Ev, make_ctx};
use carve_core::inventories::InvClick;
use carve_core::item_meta::ItemMeta;
use carve_core::value::{EntityRef, Value};
use carve_core::*;

#[derive(Default)]
struct Mock {
    msgs: Vec<String>,
    calls: Vec<String>,
    online: HashMap<String, String>,
    logs: Vec<(LogLevel, String)>,
    held: Option<(String, u32)>,
    held_meta: Option<ItemMeta>,
    book: Option<Vec<u8>>,
    border: HashMap<String, Value>,
    attr: HashMap<String, f64>,
    ai: bool,
    effects: Vec<String>,
}

impl Mock {
    fn steve() -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m.ai = true;
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
    fn player_location(&mut self, _p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((0.5, 64.5, 0.5, "world".into()))
    }
    fn default_world(&mut self) -> Option<String> {
        Some("world".into())
    }
    fn held_item(&mut self, _p: &PlayerRef) -> Option<(String, u32)> {
        self.held.clone()
    }
    fn held_item_meta(&mut self, _p: &PlayerRef) -> Option<ItemMeta> {
        self.held_meta.clone()
    }
    fn set_held_item_meta(&mut self, _p: &PlayerRef, meta: &ItemMeta) -> bool {
        self.held_meta = Some(meta.clone());
        true
    }
    fn add_effect(&mut self, _p: &PlayerRef, effect: &str, ticks: u32, amplifier: u8) {
        self.effects.push(format!("{effect} {ticks} {amplifier}"));
    }
    fn block_shape_query(&mut self, q: Q<'_>) -> Option<Value> {
        let yes = Some(Value::Bool(true));
        match q {
            Q::Brew(_, 0) => Some(Value::Item {
                id: "potion".into(),
                count: 1,
                meta: None,
            }),
            Q::Brew(_, 3) => Some(Value::Item {
                id: "nether_wart".into(),
                count: 2,
                meta: None,
            }),
            Q::Brew(..) => Some(Value::None),
            Q::SetBrew(p, s, v) => {
                self.calls
                    .push(format!("brew {} {} {} slot {s} {v:?}", p.x, p.y, p.z));
                yes
            }
            Q::Border(w, f) => self.border.get(&format!("{w} {f:?}")).cloned(),
            Q::SetBorder(w, f, v) => {
                self.border.insert(format!("{w} {f:?}"), v.clone());
                yes
            }
            Q::HeldBook(_) => self
                .book
                .clone()
                .map(|b| Value::List(b.into_iter().map(|x| Value::Num(f64::from(x))).collect())),
            Q::SetHeldBook(_, b) => {
                self.book = Some(b.to_vec());
                yes
            }
            Q::BarExists(id) => Some(Value::Bool(id == "raid")),
            Q::Ender(_, Some(s)) => Some(Value::Item {
                id: format!("slot{s}"),
                count: 1,
                meta: None,
            }),
            Q::Ender(_, None) => Some(Value::List(vec![Value::Num(1.0), Value::Num(2.0)])),
            Q::SetEnder(_, s, v) => {
                self.calls.push(format!("ender {s} {v}"));
                yes
            }
            Q::ClearEnder(_) => {
                self.calls.push("ender clear".into());
                yes
            }
            Q::OpenEnder(_) => {
                self.calls.push("ender open".into());
                yes
            }
            Q::Ai(_) => Some(Value::Bool(self.ai)),
            Q::SetAi(_, on) => {
                self.ai = on;
                yes
            }
            Q::Attr(_, n, total) => self
                .attr
                .get(n)
                .map(|v| Value::Num(if total { v + 1.0 } else { *v })),
            Q::SetAttr(_, n, v) => {
                self.attr.insert(n.to_string(), v);
                yes
            }
            Q::ResetAttr(_, n) => {
                self.attr.remove(n);
                yes
            }
            Q::CarriedBlock(_) => Some(Value::Text("minecraft:grass_block[snowy=false]".into())),
            Q::SetCarriedBlock(_, d) => {
                self.calls.push(format!("carry {d:?}"));
                yes
            }
        }
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

// ---------- blocks ----------

#[test]
fn sphere_cuboid_line_and_ray() {
    let s = c4::sphere(0.5, 64.5, 0.5, 1.0, &None).expect("sphere");
    assert_eq!(s.len(), 7);
    assert_eq!(c4::sphere(0.5, 64.5, 0.5, 2.0, &None).expect("r2").len(), 33);
    assert!(c4::sphere(0.0, 0.0, 0.0, 99.0, &None).is_err());
    let m = msgs(
        "set {_a::*} to blocks in radius 1 of player\nsend \"%size of {_a::*}%\"\nset {_b::*} to all blocks around player in radius 1\nsend \"%size of {_b::*}%\"\nset {_c::*} to blocks between location(0, 0, 0) and location(1, 1, 1)\nsend \"%size of {_c::*}%\"\nset {_d::*} to blocks from location(0, 0, 0) to location(3, 0, 0)\nsend \"%size of {_d::*}%\"\nsend \"%{_d::4}%\"\nset {_e::*} to blocks above player\nsend \"%size of {_e::*}%\"",
    );
    assert_eq!(m[..4], ["7", "7", "8", "4"]);
    assert!(m[4].contains('3'), "{m:?}");
    assert_eq!(m[5], "100");
    let m = msgs("set {_n} to 0\nloop blocks in radius 1 of player:\n    add 1 to {_n}\nsend \"%{_n}%\"");
    assert_eq!(m, vec!["7"]);
}

#[test]
fn line_is_connected() {
    let l = c4::line((0.5, 0.5, 0.5), (5.5, 3.5, -2.5), &None);
    let pos = |v: &Value| match v {
        Value::Location { x, y, z, .. } => (*x as i32, *y as i32, *z as i32),
        _ => panic!("not a location"),
    };
    for w in l.windows(2) {
        let (a, b) = (pos(&w[0]), pos(&w[1]));
        assert_eq!(
            (a.0 - b.0).abs() + (a.1 - b.1).abs() + (a.2 - b.2).abs(),
            1,
            "{a:?} {b:?}"
        );
    }
    assert_eq!(pos(l.last().expect("end")), (5, 3, -3));
}

#[test]
fn brewing_stand_slots() {
    let mut h = Mock::steve();
    run_with(
        &mut h,
        "send \"%first bottle slot of block at player%\"\nsend \"%brewing stand's ingredient slot of block at player%\"\nsend \"%second bottle of block at player%\"\nset fuel slot of block at player to blaze powder",
    );
    assert_eq!(h.msgs, vec!["potion", "2 nether wart", "<none>"]);
    assert!(
        h.calls[0].contains("blaze_powder") && h.calls[0].contains("slot 4"),
        "{:?}",
        h.calls
    );
}

// ---------- books ----------

#[test]
fn book_round_trip() {
    let b = BookData {
        title: "Tales".into(),
        author: "Steve".into(),
        pages: vec!["one".into(), "two".into()],
    };
    let bytes = b.encode().expect("encode");
    assert_eq!(BookData::decode(&bytes), Some(b));
}

#[test]
fn book_values_on_the_tool() {
    let mut h = Mock::steve();
    h.held = Some(("written_book".into(), 1));
    run_with(
        &mut h,
        "set book title of player's tool to \"Tales\"\nset book author of player's tool to \"Ann\"\nset page 2 of player's tool to \"second\"\nsend \"%book title of player's tool%|%author of player's tool%|%page 2 of player's tool%|%size of book pages of player's tool%\"",
    );
    assert_eq!(h.msgs, vec!["Tales|Ann|second|2"]);
    let b = BookData::decode(h.book.as_ref().expect("book")).expect("decode");
    assert_eq!(b.pages, vec!["", "second"]);
}

// ---------- boss bars, borders ----------

#[test]
fn boss_bar_by_id() {
    assert_eq!(
        msgs("send \"%boss bar with id \"raid\"%\"\nsend \"%bossbar from key \"other\"%\""),
        vec!["raid", "<none>"]
    );
}

#[test]
fn world_border_settings() {
    let mut h = Mock::steve();
    run_with(
        &mut h,
        "set world border damage amount of world border of world \"world\" to 2\nadd 1 to world border damage amount of world border of world \"world\"\nset world border damage buffer of world \"world\" to 7\nset world border center of world border of world \"world\" to location(10, 0, 20)\nsend \"%world border damage amount of world border of world \"world\"%\"",
    );
    assert_eq!(
        h.border.get(&format!("world {:?}", BorderField::DamageBuffer)),
        Some(&Value::Num(7.0))
    );
    assert!(
        matches!(h.border.get(&format!("world {:?}", BorderField::Center)), Some(Value::Location { x, z, .. }) if *x == 10.0 && *z == 20.0)
    );
    assert_eq!(h.msgs, vec!["3"]);
}

// ---------- potion effects ----------

#[test]
fn unknown_potion_effect() {
    assert!(!compile_err(&script("on join", "set {_e} to potion effect of flying")).is_empty());
}

// ---------- items ----------

#[test]
fn damage_and_durability_of_items() {
    assert_eq!(
        msgs(
            "set {_s} to diamond sword with damage 10\nsend \"%damage of {_s}%|%durability of {_s}%\"\nsend \"%damage of (iron pickaxe damaged by 5)%\""
        ),
        vec!["10|1551", "5"]
    );
}

#[test]
fn damage_of_the_tool() {
    let mut h = Mock::steve();
    h.held = Some(("iron_sword".into(), 1));
    run_with(
        &mut h,
        "set damage of player's tool to 40\nadd 2 to damage of player's tool\nsend \"%durability of player's tool%\"",
    );
    assert_eq!(h.held_meta.as_ref().and_then(|m| m.damage), Some(42));
    assert_eq!(h.msgs, vec!["208"]);
    run_with(&mut h, "set durability of player's tool to 250");
    assert_eq!(h.held_meta.as_ref().and_then(|m| m.damage), None);
}

#[test]
fn damage_survives_json() {
    let m = ItemMeta {
        damage: Some(3),
        ..ItemMeta::default()
    };
    assert_eq!(ItemMeta::from_json(&m.to_json()).map(|b| *b), Some(m));
}

// ---------- ender chests ----------

#[test]
fn ender_chest() {
    let mut h = Mock::steve();
    run_with(
        &mut h,
        "send \"%slot 3 of ender chest of player%\"\nsend \"%slot 4 of player's ender chest%\"\nsend \"%size of player's ender chest%\"\nset slot 0 of ender chest of player to diamond\nclear ender chest of player\nopen player's ender chest to player",
    );
    assert_eq!(h.msgs, vec!["slot3", "slot4", "2"]);
    assert_eq!(h.calls, vec!["ender 0 diamond", "ender clear", "ender open"]);
}

// ---------- entities ----------

#[test]
fn ai_and_attributes() {
    let mut h = Mock::steve();
    run_with(
        &mut h,
        "set movement speed attribute of player to 0.2\nadd 0.1 to generic movement speed attribute of player\nsend \"%movement speed attribute of player%|%movement speed total attribute value of player%\"\nset ai of player to false\nsend \"%ai of player%|%player's artificial intelligence%\"\nreset max health attribute of player",
    );
    let m = &h.msgs;
    assert!(m[0].starts_with("0.3"), "{m:?}");
    assert!(m[0].contains("|1.3"), "{m:?}");
    assert_eq!(m[1], "false|false");
}

#[test]
fn carried_block() {
    let mut e = load_src(&script(
        "on death",
        "send \"%carried block of victim%\"\nset carried block of victim to stone\nreset carried block of victim",
    ));
    let mut h = Mock::steve();
    let ender = EntityRef {
        uuid: "e1".into(),
        kind: "enderman".into(),
    };
    e.fire(
        EventKind::Death,
        EventCtx::entity_victim(EntSlot::Ent(ender)),
        &mut h,
    );
    no_errors(&h);
    assert_eq!(h.msgs, vec!["minecraft:grass_block[snowy=false]"]);
    assert_eq!(h.calls, vec!["carry Some(\"stone\")", "carry None"]);
}

#[test]
fn event_entity_by_type() {
    let src = script(
        "on damage",
        "send \"%the zombie%|%the creature%|%event-skeleton%\"",
    );
    let mut e = load_src(&src);
    let mut h = Mock::steve();
    let z = EntityRef {
        uuid: "z1".into(),
        kind: "zombie".into(),
    };
    e.fire(
        EventKind::Damage,
        EventCtx::entity_victim(EntSlot::Ent(z)),
        &mut h,
    );
    no_errors(&h);
    assert_eq!(h.msgs.len(), 1);
    let parts: Vec<&str> = h.msgs[0].split('|').collect();
    assert_ne!(parts[0], "<none>");
    assert_eq!(parts[0], parts[1]);
    assert_eq!(parts[2], "<none>");
}

// ---------- enchanting ----------

#[test]
fn enchantment_level_of_text() {
    assert_eq!(
        msgs(
            "send \"%enchantment level of \"sharpness 5\"%|%enchant level of \"sharpness IV\"%|%enchantment level of \"mending\"%\""
        ),
        vec!["5|4|1"]
    );
}

#[test]
fn enchant_offers_in_prepare() {
    let body = "send \"%size of enchantment offers%\"\nsend \"%cost of enchantment offer 1%|%enchantment level of enchant offer 2%\"\nsend \"%enchantment cost of first enchantment offer%|%enchantment bonus%|%enchant item%\"\nsend \"%enchantment offer 3%\"";
    let mut e = load_src(&script("on enchant prepare", body));
    let mut h = Mock::steve();
    let nums = [12.0, 2.0, 3.0, 33.0, 1.0, 7.0, 40.0, 2.0, 0.0, -1.0, 0.0];
    e.fire(
        EventKind::Game(Ev::EnchantPrepare),
        make_ctx(Ev::EnchantPrepare, Some(steve()), &nums, &["diamond_sword"], &[]),
        &mut h,
    );
    no_errors(&h);
    assert_eq!(h.msgs[0], "2");
    assert_eq!(h.msgs[1], "3|2");
    assert_eq!(h.msgs[2], "3|12|diamond_sword");
    assert_eq!(h.msgs[3], "<none>");
}

#[test]
fn enchant_cost_and_hint() {
    let body =
        "send \"%enchanting cost%|%exp cost%|%enchantment hint%|%enchantment hint level%|%enchant item%\"";
    let mut e = load_src(&script("on enchant", body));
    let mut h = Mock::steve();
    e.fire(
        EventKind::Game(Ev::Enchant),
        make_ctx(
            Ev::Enchant,
            Some(steve()),
            &[1.0, 2.0, 1.0, 3.0],
            &["diamond_sword", "sharpness"],
            &[],
        ),
        &mut h,
    );
    no_errors(&h);
    assert_eq!(h.msgs, vec!["2|2|sharpness|3|diamond_sword"]);
}

#[test]
fn offers_only_in_prepare() {
    assert!(compile_err(&script("on join", "send \"%enchantment offers%\"")).contains("enchant prepare"));
}

// ---------- inventory clicks ----------

fn click(click: &str, item: Option<(&str, u32)>, cursor: Option<(&str, u32)>) -> EventCtx {
    let mut c = EventCtx::player(steve());
    c.inv = Some(Box::new(InvClick {
        slot: 2,
        item: item.map(|(i, n)| (i.to_string(), n)),
        click: click.into(),
        menu: None,
        cursor: cursor.map(|(i, n)| (i.to_string(), n)),
    }));
    c
}

#[test]
fn inventory_action_and_cursor() {
    let body = "send \"%inventory action%|%cursor slot of player%\"\nif inventory action is \"pickup all\":\n    send \"picked\"";
    let mut e = load_src(&script("on inventory click", body));
    let mut h = Mock::steve();
    e.fire(
        EventKind::InventoryClick,
        click("left", Some(("stone", 3)), None),
        &mut h,
    );
    e.fire(
        EventKind::InventoryClick,
        click("right", None, Some(("dirt", 5))),
        &mut h,
    );
    e.fire(
        EventKind::InventoryClick,
        click("left", Some(("stone", 3)), Some(("dirt", 1))),
        &mut h,
    );
    no_errors(&h);
    assert_eq!(
        h.msgs,
        vec![
            "pickup all|<none>",
            "picked",
            "place one|5 dirt",
            "swap with cursor|dirt"
        ]
    );
}

#[test]
fn inventory_action_rules() {
    let it = ("stone".to_string(), 10);
    let other = ("dirt".to_string(), 1);
    assert_eq!(c4::inventory_action("right", 1, Some(&it), None), "pickup_half");
    assert_eq!(
        c4::inventory_action("left", -999, None, Some(&it)),
        "drop_all_cursor"
    );
    assert_eq!(c4::inventory_action("drop", 1, Some(&it), None), "drop_one_slot");
    assert_eq!(
        c4::inventory_action("control drop", 1, Some(&it), None),
        "drop_all_slot"
    );
    assert_eq!(
        c4::inventory_action("shift left", 1, Some(&it), None),
        "move_to_other_inventory"
    );
    assert_eq!(c4::inventory_action("number key", 1, None, None), "hotbar_swap");
    assert_eq!(
        c4::inventory_action("left", 1, Some(&it), Some(&other)),
        "swap_with_cursor"
    );
    assert_eq!(
        c4::inventory_action(
            "left",
            1,
            Some(&("stone".into(), 60)),
            Some(&("stone".into(), 10))
        ),
        "place_some"
    );
}

#[test]
fn cursor_only_in_clicks() {
    assert!(compile_err(&script("on join", "send \"%cursor slot of player%\"")).contains("inventory click"));
}

// ---------- event values ----------

#[test]
fn broadcast_message_and_consumed_item() {
    let mut e = load_src(&script(
        "on broadcast",
        "send \"got %broadcast message%\" to console",
    ));
    let mut h = Mock::steve();
    let mut ctx = EventCtx::console();
    ctx.ev = vec![Value::Text("hello".into()), Value::Text("Server".into())];
    e.fire(EventKind::Ext(ExtEvent::Broadcast), ctx, &mut h);
    no_errors(&h);
    assert_eq!(h.msgs, vec!["got hello"]);

    let mut e = load_src(&script("on consume", "send \"%consumed item%\""));
    let mut h = Mock::steve();
    let extra = carve_core::events_items::EventExtra {
        item: Some(("bread".into(), 1)),
        ..Default::default()
    };
    e.fire(EventKind::Consume, EventCtx::with_extra(steve(), extra), &mut h);
    no_errors(&h);
    assert_eq!(h.msgs, vec!["bread"]);
    assert!(!compile_err(&script("on join", "send \"%broadcast message%\"")).is_empty());
}

#[test]
fn plain_lines_still_parse() {
    // Words this module looks at must not break the old meanings.
    msgs(
        "set {_x} to block at player\nsend \"%type of block at player%\"\nset {_l} to 3 blocks above player\nsend \"%first element of {_l::*}%\"\nset {_v} to max(1, 2)\napply speed 2 to player for 5 seconds",
    );
}
