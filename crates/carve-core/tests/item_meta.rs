use std::collections::HashMap;

use carve_core::compile::{EventKind, check_registry};
use carve_core::item_meta::ItemMeta;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    online: HashMap<String, String>,
    logs: Vec<(LogLevel, String)>,
    held: Option<(String, u32)>,
    held_meta: Option<ItemMeta>,
    sent: Vec<String>,
}

impl Host for Mock {
    fn send_message(&mut self, _: &Recipient, m: &str) {
        self.sent.push(m.to_string());
    }
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.calls.push(c.to_string());
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
    fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
        true
    }
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
    fn held_item(&mut self, _: &PlayerRef) -> Option<(String, u32)> {
        self.held.clone()
    }
    fn held_item_meta(&mut self, _: &PlayerRef) -> Option<ItemMeta> {
        self.held_meta.clone()
    }
    fn online_players(&mut self) -> Vec<PlayerRef> {
        vec![steve()]
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn blade_meta() -> ItemMeta {
    ItemMeta {
        name: Some("&bBlade".into()),
        lore: vec!["first".into(), "second".into()],
        enchants: vec![("sharpness".into(), 5), ("sweeping_edge".into(), 2)],
    }
}

fn run_with(src: &str, held: Option<(&str, u32, Option<ItemMeta>)>) -> Mock {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "{}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    let mut h = Mock::default();
    h.online.insert("u1".into(), "Steve".into());
    if let Some((id, n, m)) = held {
        h.held = Some((id.into(), n));
        h.held_meta = m;
    }
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    h
}

fn ok(line: &str) -> Vec<String> {
    let h = run_with(&format!("on join:\n    {line}\n"), None);
    assert!(h.errors().is_empty(), "{:?}", h.errors());
    h.calls
}

fn sent(line: &str) -> Vec<String> {
    let h = run_with(
        &format!("on join:\n    {line}\n"),
        Some(("diamond_sword", 1, Some(blade_meta()))),
    );
    assert!(h.errors().is_empty(), "{:?}", h.errors());
    h.sent
}

impl Mock {
    fn errors(&self) -> Vec<&String> {
        self.logs
            .iter()
            .filter(|(l, _)| *l == LogLevel::Error)
            .map(|(_, m)| m)
            .collect()
    }
}

fn compiles(line: &str) -> bool {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), format!("on join:\n    {line}\n"))]);
    errs.is_empty()
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn give_named() {
    assert_eq!(
        ok(r#"give player diamond sword named "Blade""#),
        [r#"give Steve minecraft:diamond_sword[custom_name="Blade"] 1"#]
    );
}

#[test]
fn give_named_with_colour_codes() {
    assert_eq!(
        ok(r#"give player diamond sword named "&bBlade""#),
        ["give Steve minecraft:diamond_sword[custom_name=\"§bBlade\"] 1"]
    );
}

#[test]
fn give_lore_two_lines() {
    assert_eq!(
        ok(r#"give player diamond sword with lore "a" and "b""#),
        [r#"give Steve minecraft:diamond_sword[lore=["a","b"]] 1"#]
    );
}

#[test]
fn give_enchanted() {
    assert_eq!(
        ok("give player diamond sword enchanted with sharpness 5"),
        [r#"give Steve minecraft:diamond_sword[enchantments={"minecraft:sharpness":5}] 1"#]
    );
}

#[test]
fn give_enchanted_two_and_roman_and_default_level() {
    assert_eq!(
        ok("give player diamond sword enchanted with sharpness iii and sweeping edge and unbreaking 3"),
        [
            r#"give Steve minecraft:diamond_sword[enchantments={"minecraft:sharpness":3,"minecraft:sweeping_edge":1,"minecraft:unbreaking":3}] 1"#
        ]
    );
}

#[test]
fn give_everything_in_any_order_with_count() {
    assert_eq!(
        ok(r#"give 2 diamond swords enchanted with sharpness 5 named "X" with lore "L" to player"#),
        [
            r#"give Steve minecraft:diamond_sword[custom_name="X",lore=["L"],enchantments={"minecraft:sharpness":5}] 2"#
        ]
    );
}

#[test]
fn give_count_override_keeps_meta() {
    assert_eq!(
        ok(r#"give 3 of diamond sword named "N" to player"#),
        [r#"give Steve minecraft:diamond_sword[custom_name="N"] 3"#]
    );
}

#[test]
fn give_plain_item_unchanged() {
    assert_eq!(ok("give player 2 diamonds"), ["give Steve minecraft:diamond 2"]);
}

#[test]
fn give_quotes_are_escaped() {
    let c = ok(r#"give player stick named "say ""hi""""#);
    assert_eq!(c, [r#"give Steve minecraft:stick[custom_name="say \"hi\""] 1"#]);
}

#[test]
fn variable_holds_meta_and_gives_it() {
    let c = ok("set {_i} to diamond sword named \"V\" with lore \"x\"\n    give player {_i}");
    assert_eq!(
        c,
        [r#"give Steve minecraft:diamond_sword[custom_name="V",lore=["x"]] 1"#]
    );
}

#[test]
fn name_of_tool() {
    assert_eq!(sent("send name of player's tool"), ["&bBlade"]);
    assert_eq!(sent("send player's tool's name"), ["&bBlade"]);
}

#[test]
fn name_of_item_without_name_is_none() {
    let h = run_with(
        "on join:\n    send name of player's tool\n",
        Some(("stick", 1, None)),
    );
    assert_eq!(h.sent, ["<none>"]);
}

#[test]
fn lore_of_tool() {
    assert_eq!(sent("send lore of player's tool"), ["first and second"]);
}

#[test]
fn lore_of_loops() {
    assert_eq!(
        sent("loop lore of player's tool:\n        send loop-value"),
        ["first", "second"]
    );
}

#[test]
fn enchantments_of_tool() {
    assert_eq!(
        sent("send enchantments of player's tool"),
        ["sharpness 5 and sweeping edge 2"]
    );
}

#[test]
fn item_type_and_amount() {
    let h = run_with(
        "on join:\n    send item type of player's tool\n    send item amount of player's tool\n    send amount of player's tool\n",
        Some(("diamond_sword", 3, Some(blade_meta()))),
    );
    assert_eq!(h.sent, ["diamond sword", "3", "3"]);
}

#[test]
fn amount_of_list_still_counts_list() {
    let h = run_with(
        "on join:\n    set {_l::*} to 1, 2 and 3\n    send amount of {_l::*}\n",
        None,
    );
    assert_eq!(h.sent, ["3"]);
}

#[test]
fn enchanted_condition() {
    assert_eq!(
        sent("if player's tool is enchanted with sharpness 5:\n        send \"yes\""),
        ["yes"]
    );
    assert_eq!(
        sent("if player's tool is enchanted with sharpness 4:\n        send \"yes\""),
        ["yes"]
    );
    assert!(sent("if player's tool is enchanted with sharpness 6:\n        send \"yes\"").is_empty());
    assert!(sent("if player's tool is enchanted with unbreaking:\n        send \"yes\"").is_empty());
    assert_eq!(
        sent("if player's tool is enchanted with sweeping edge:\n        send \"yes\""),
        ["yes"]
    );
}

#[test]
fn enchanted_any_and_negated() {
    assert_eq!(
        sent("if player's tool is enchanted:\n        send \"yes\""),
        ["yes"]
    );
    assert_eq!(
        sent("if player's tool is not enchanted with fire aspect:\n        send \"no fire\""),
        ["no fire"]
    );
    let h = run_with(
        "on join:\n    if player's tool is not enchanted:\n        send \"plain\"\n",
        Some(("stick", 1, None)),
    );
    assert_eq!(h.sent, ["plain"]);
}

#[test]
fn enchanted_on_item_built_in_script() {
    assert_eq!(
        ok("if diamond pickaxe enchanted with efficiency 3 is enchanted with efficiency 2:\n        execute console command \"say ok\"")
            .len(),
        1
    );
}

#[test]
fn compare_ignores_meta() {
    let h = run_with(
        "on join:\n    if player's tool is diamond sword:\n        send \"same\"\n    if player's tool is diamond sword named \"Other\":\n        send \"same too\"\n",
        Some(("diamond_sword", 1, Some(blade_meta()))),
    );
    assert_eq!(h.sent, ["same", "same too"]);
}

#[test]
fn compare_different_type_is_false() {
    let h = run_with(
        "on join:\n    if player's tool is stick:\n        send \"stick\"\n",
        Some(("diamond_sword", 1, Some(blade_meta()))),
    );
    assert!(h.sent.is_empty());
}

#[test]
fn has_item_ignores_meta() {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[(
        "t.carve".into(),
        "on join:\n    if player has diamond sword named \"x\":\n        send \"has\"\n".into(),
    )]);
    assert!(errs.is_empty());
}

#[test]
fn named_item_prints_with_name() {
    assert_eq!(
        ok_sent(r#"send diamond sword named "Blade""#),
        ["diamond sword named \"Blade\""]
    );
}

fn ok_sent(line: &str) -> Vec<String> {
    run_with(&format!("on join:\n    {line}\n"), None).sent
}

#[test]
fn with_name_form_and_list_store() {
    assert_eq!(
        ok(r#"give player stick with name "N""#),
        [r#"give Steve minecraft:stick[custom_name="N"] 1"#]
    );
    assert_eq!(
        sent("set {_e::*} to enchantments of player's tool\n    send size of {_e::*}\n    send {_e::1}"),
        ["2", "sharpness 5"]
    );
}

#[test]
fn bad_syntax_is_an_error() {
    assert!(!compiles("give player diamond sword enchanted with nonsense"));
    assert!(!compiles("give player diamond sword named"));
    assert!(compiles(r#"give player diamond sword named "x""#));
}

#[test]
fn meta_json_roundtrip_in_variables() {
    let src = "on join:\n    set {i} to diamond sword named \"&bBlade\" with lore \"a\" and \"b\" enchanted with sharpness 5\n";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty());
    let mut h = Mock::default();
    h.online.insert("u1".into(), "Steve".into());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let back = Variables::from_json(&e.save_now().unwrap()).unwrap();
    let Value::Item { id, count, meta } = back.get("i") else {
        panic!("not an item");
    };
    assert_eq!((id.as_str(), count), ("diamond_sword", 1));
    let m = meta.unwrap();
    assert_eq!(m.name.as_deref(), Some("&bBlade"));
    assert_eq!(m.lore, ["a", "b"]);
    assert_eq!(m.enchants, [("sharpness".to_string(), 5)]);
}

#[test]
fn plain_item_json_has_no_meta() {
    let src = "on join:\n    set {i} to 2 diamonds\n";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty());
    let mut h = Mock::default();
    h.online.insert("u1".into(), "Steve".into());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let json = e.save_now().unwrap();
    assert!(!json.contains("meta"), "{json}");
}

#[test]
fn plain_text_from_json_component() {
    assert_eq!(
        carve_core::item_meta::plain_from_json(r#"{"text":"a","extra":[{"text":"b"},"c"]}"#),
        "abc"
    );
    assert_eq!(carve_core::item_meta::plain_from_json(r#""hi""#), "hi");
}

#[test]
fn enchant_parsing() {
    use carve_core::lexer::lex;
    let t = lex("sweeping edge 3 and").unwrap();
    assert_eq!(
        carve_core::item_meta::enchant_at(&t, 0),
        Some(("sweeping_edge", 3, 3))
    );
    let t = lex("luck of the sea iv").unwrap();
    assert_eq!(
        carve_core::item_meta::enchant_at(&t, 0),
        Some(("luck_of_the_sea", 4, 5))
    );
    let t = lex("curse of binding").unwrap();
    assert_eq!(
        carve_core::item_meta::enchant_at(&t, 0),
        Some(("binding_curse", 0, 3))
    );
    let t = lex("diamond").unwrap();
    assert_eq!(carve_core::item_meta::enchant_at(&t, 0), None);
}
