use std::collections::HashMap;

use carve_core::compile::{EventKind, check_registry};
use carve_core::w2_expressions_1::ArmorSlot;
use carve_core::*;

#[derive(Default)]
struct Mock {
    sent: Vec<String>,
    online: HashMap<String, String>,
    inv: HashMap<String, u32>,
    armor: Vec<(ArmorSlot, String, u32)>,
    nbt: Option<Vec<u8>>,
    asked: Vec<String>,
    logs: Vec<(LogLevel, String)>,
}

impl Mock {
    fn new() -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m
    }
}

impl Host for Mock {
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
        self.online.get(uuid).cloned()
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
        self.logs.push((level, msg.to_string()));
    }
    fn player_location(&mut self, _p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((1.5, 64.0, -3.0, "world".into()))
    }
    fn count_item(&mut self, _p: &PlayerRef, id: &str) -> u32 {
        self.inv.get(id).copied().unwrap_or(0)
    }
    fn biome_at(&mut self, world: &str, x: i32, y: i32, z: i32) -> Option<String> {
        self.asked.push(format!("biome {world} {x} {y} {z}"));
        Some("birch_forest".into())
    }
    fn armor_item(&mut self, _p: &PlayerRef, slot: ArmorSlot) -> Option<(String, u32)> {
        self.armor
            .iter()
            .find(|(s, _, _)| *s == slot)
            .map(|(_, id, n)| (id.clone(), *n))
    }
    fn block_entity_nbt(&mut self, world: &str, x: i32, y: i32, z: i32) -> Option<Vec<u8>> {
        self.asked.push(format!("nbt {world} {x} {y} {z}"));
        self.nbt.clone()
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn load(src: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn run(lines: &[&str], h: &mut Mock) {
    let mut src = String::from("on join:\n");
    for l in lines {
        src.push_str("    ");
        src.push_str(l);
        src.push('\n');
    }
    let mut e = load(&src);
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
    let bad: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(bad.is_empty(), "runtime errors: {bad:?}");
}

fn out(lines: &[&str]) -> Vec<String> {
    let mut h = Mock::new();
    run(lines, &mut h);
    h.sent
}

fn nbt(entries: &[(&str, u8, Vec<u8>)]) -> Vec<u8> {
    let mut b = vec![10u8];
    for (k, t, v) in entries {
        b.push(*t);
        b.extend((k.len() as u16).to_be_bytes());
        b.extend(k.as_bytes());
        b.extend(v);
    }
    b.push(0);
    b
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn eternity() {
    let o = out(&[
        "set {a} to an eternity",
        "set {b} to eternity",
        r#"if {a} > 5 days:"#,
        r#"    send "longer""#,
        "set {c} to now + an eternity",
        r#"send "ok""#,
    ]);
    assert_eq!(o[0], "longer");
    assert_eq!(o[1], "ok");
}

#[test]
fn angles() {
    let o = out(&[
        r#"send "%90 degrees%|%1 degree%|%180 radians%|%3.14159265358979 rad%""#,
        r#"set {a} to 45 degrees + 45 degrees"#,
        r#"send "%{a}%""#,
    ]);
    assert_eq!(o[1], "90");
    let parts: Vec<&str> = o[0].split('|').collect();
    assert_eq!(parts[0], "90");
    assert_eq!(parts[1], "1");
    assert!(parts[2].starts_with("10313.24"), "{}", parts[2]);
    assert!(parts[3].starts_with("180"), "{}", parts[3]);
}

#[test]
fn altitude() {
    let o = out(&[
        r#"send "%altitude of player% %altitude of location(1, 70.5, 2)% %altitude of player's location%""#,
    ]);
    assert_eq!(o, vec!["64 70.5 64"]);
}

#[test]
fn color_channels() {
    let o = out(&[
        r##"set {h} to "#ff8000""##,
        r##"send "%red value of {h}% %green value of {h}% %blue value of {h}% %alpha value of {h}%""##,
        r#"set {g} to "gold""#,
        r#"send "%red value of {g}% %green value of {g}% %blue component of {g}%""#,
        r##"set {c} to "&#102030""##,
        r#"send "%blue value of {c}%""#,
    ]);
    assert_eq!(o[0], "255 128 0 255");
    assert_eq!(o[1], "255 170 0");
    assert_eq!(o[2], "48");
}

#[test]
fn item_amounts() {
    let mut h = Mock::new();
    h.inv.insert("diamond".into(), 7);
    run(
        &[
            r#"send "%amount of diamonds in player's inventory%""#,
            r#"send "%number of diamond in inventory of player%""#,
            r#"send "%amount of dirt in player's inventory%""#,
            r#"if amount of diamonds in player's inventory > 5:"#,
            r#"    send "rich""#,
            r#"set {l::*} to 1, 2 and 3"#,
            r#"send "%amount of {l::*}%""#,
        ],
        &mut h,
    );
    assert_eq!(h.sent, vec!["7", "7", "0", "rich", "3"]);
}

#[test]
fn alphabetical_sort() {
    let o = out(&[
        r#"set {l::*} to "pear", "Apple", "banana" and "10""#,
        r#"set {s::*} to alphabetically sorted {l::*}"#,
        r#"send "%join {s::*} with ","%""#,
        r#"loop alphabetically sorted {l::*}:"#,
        r#"    send "- %loop-value%""#,
    ]);
    assert_eq!(o[0], "10,Apple,banana,pear");
    assert_eq!(&o[1..], ["- 10", "- Apple", "- banana", "- pear"]);
}

#[test]
fn biome() {
    let mut h = Mock::new();
    run(
        &[
            r#"send "%biome at player%""#,
            r#"send "%biome of location(10.9, 70, -0.5, world "nether")%""#,
        ],
        &mut h,
    );
    assert_eq!(h.sent, vec!["birch_forest", "birch_forest"]);
    assert_eq!(h.asked, vec!["biome world 1 64 -3", "biome nether 10 70 -1"]);
}

#[test]
fn armor() {
    let mut h = Mock::new();
    h.armor.push((ArmorSlot::Helmet, "iron_helmet".into(), 1));
    h.armor.push((ArmorSlot::Boots, "diamond_boots".into(), 1));
    run(
        &[
            r#"send "%helmet of player%""#,
            r#"if boots of player is set:"#,
            r#"    send "boots on""#,
            r#"if chestplate of player is not set:"#,
            r#"    send "no chest""#,
        ],
        &mut h,
    );
    assert!(
        h.sent[0].contains("iron helmet") || h.sent[0].contains("iron_helmet"),
        "{:?}",
        h.sent
    );
    assert_eq!(&h.sent[1..], ["boots on", "no chest"]);
}

#[test]
fn block_entities() {
    let mut h = Mock::new();
    h.nbt = Some(nbt(&[
        ("Levels", 3, 4i32.to_be_bytes().to_vec()),
        ("BrewTime", 2, 400i16.to_be_bytes().to_vec()),
        ("Fuel", 1, vec![17]),
        ("lit_time_remaining", 2, 100i16.to_be_bytes().to_vec()),
        ("cooking_time_spent", 2, 60i16.to_be_bytes().to_vec()),
        ("cooking_total_time", 2, 200i16.to_be_bytes().to_vec()),
    ]));
    run(
        &[
            r#"set {l} to location(5, 70, 6, world "world")"#,
            r#"send "%beacon tier of {l}% %beacon range of {l}%""#,
            r#"send "%brewing fuel level of {l}% %brewing stand fuel of {l}%""#,
            r#"send "%milliseconds of brewing time of {l}% %total milliseconds of burn time of {l}%""#,
            r#"send "%total milliseconds of cook time of {l}% %total milliseconds of total cook time of {l}%""#,
        ],
        &mut h,
    );
    assert_eq!(h.sent[0], "4 50");
    assert_eq!(h.sent[1], "17 17");
    assert_eq!(h.sent[2], "0 5000");
    assert_eq!(h.sent[3], "3000 10000");
    assert_eq!(h.asked[0], "nbt world 5 70 6");
}

#[test]
fn block_entity_missing_is_none() {
    let o = out(&[
        r#"set {l} to location(5, 70, 6, world "world")"#,
        r#"if beacon tier of {l} is not set:"#,
        r#"    send "no beacon""#,
    ]);
    assert_eq!(o, vec!["no beacon"]);
}

#[test]
fn all_scripts() {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[
        (
            "a.carve".into(),
            "on join:\n    set {s::*} to all scripts\n    send \"n=%size of {s::*}%\"\n".into(),
        ),
        ("b.carve".into(), "on quit:\n    stop\n".into()),
    ]);
    assert!(errs.is_empty(), "{errs:?}");
    let mut h = Mock::new();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.sent, vec!["n=2"]);
}

#[test]
fn old_syntax_still_works() {
    let o = out(&[
        r#"send "%1 hour and 30 minutes%""#,
        r#"set {a::*} to 1, 2 and 3"#,
        r#"send "%amount of {a::*}%""#,
    ]);
    assert_eq!(o.len(), 2);
    assert_eq!(o[1], "3");
}
