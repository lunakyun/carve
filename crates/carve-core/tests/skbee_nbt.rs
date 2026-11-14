use std::collections::HashMap;

use carve_core::compile::{EventKind, check_registry, compile_file};
use carve_core::skbee_nbt::{Compound, ItemComp, Tag, parse_snbt};
use carve_core::*;

/// A player holding a diamond sword whose components are kept as raw bytes.
struct Mock {
    held: bool,
    comps: HashMap<ItemComp, Vec<u8>>,
    calls: Vec<String>,
    errors: Vec<String>,
}

impl Mock {
    fn new(held: bool) -> Self {
        Mock {
            held,
            comps: HashMap::new(),
            calls: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl Host for Mock {
    fn send_message(&mut self, _: &Recipient, text: &str) {
        self.calls.push(text.to_string());
    }
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
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
    fn held_item(&mut self, _: &PlayerRef) -> Option<(String, u32)> {
        self.held.then(|| ("diamond_sword".to_string(), 1))
    }
    fn held_component(&mut self, _: &PlayerRef, c: ItemComp) -> Option<Vec<u8>> {
        self.comps.get(&c).cloned()
    }
    fn set_held_component(&mut self, _: &PlayerRef, c: ItemComp, b: Option<Vec<u8>>) -> bool {
        match b {
            Some(b) => self.comps.insert(c, b),
            None => self.comps.remove(&c),
        };
        true
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn run_with(mut h: Mock, body: &str) -> Mock {
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
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.errors.is_empty(), "runtime errors: {:?}", h.errors);
    h
}

fn run(body: &str) -> Mock {
    run_with(Mock::new(true), body)
}

fn bad(line: &str) -> bool {
    compile_file("b.carve", &format!("on join:\n    {line}\n")).is_err()
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

// ---- the value type ----

#[test]
fn snbt_parses_every_kind() {
    let c = Compound::parse(
        r#"{a:1,b:"x",c:[1,2],d:1b,e:2.5f,f:[I;1,2],g:{h:true},i:2s,j:3L,k:1.5,l:'q',m:[B;1,2],n:[L;5],o:unquoted}"#,
    )
    .unwrap();
    assert_eq!(c.get("a"), Some(&Tag::Int(1)));
    assert_eq!(c.get("b"), Some(&Tag::Str("x".into())));
    assert_eq!(c.get("c"), Some(&Tag::List(vec![Tag::Int(1), Tag::Int(2)])));
    assert_eq!(c.get("d"), Some(&Tag::Byte(1)));
    assert_eq!(c.get("e"), Some(&Tag::Float(2.5)));
    assert_eq!(c.get("f"), Some(&Tag::IntArray(vec![1, 2])));
    assert_eq!(c.get_path("g;h"), Some(&Tag::Byte(1)));
    assert_eq!(c.get("i"), Some(&Tag::Short(2)));
    assert_eq!(c.get("j"), Some(&Tag::Long(3)));
    assert_eq!(c.get("k"), Some(&Tag::Double(1.5)));
    assert_eq!(c.get("l"), Some(&Tag::Str("q".into())));
    assert_eq!(c.get("m"), Some(&Tag::ByteArray(vec![1, 2])));
    assert_eq!(c.get("n"), Some(&Tag::LongArray(vec![5])));
    assert_eq!(c.get("o"), Some(&Tag::Str("unquoted".into())));
}

#[test]
fn snbt_errors() {
    for s in ["{a:1", "{a 1}", "[1,\"x\"]", "{a:\"open}", "{a:1} x", ""] {
        assert!(parse_snbt(s).is_err(), "{s}");
    }
    assert!(Compound::parse("[1]").is_err());
    let deep = format!("{}{}", "[".repeat(200), "]".repeat(200));
    assert!(parse_snbt(&deep).is_err());
}

#[test]
fn snbt_round_trip() {
    let src = r#"{a:1,b:"x\"y",c:[1,2],d:1b,e:2.5f,f:[I;1,2],g:{h:1b},"odd key":3L,k:1.5d}"#;
    let c = Compound::parse(src).unwrap();
    let again = Compound::parse(&c.to_snbt()).unwrap();
    assert_eq!(c, again);
}

#[test]
fn binary_round_trip() {
    let c = Compound::parse(r#"{a:1,b:"héllo",c:[1,2],d:1b,e:2.5f,f:[I;1,2],g:{h:[{x:1},{x:2}]},j:3L,k:1.5,m:[B;1,2],n:[L;5],s:7s}"#).unwrap();
    let bytes = c.to_bytes().unwrap();
    assert_eq!(bytes[0], 10);
    assert_eq!(Compound::from_bytes(&bytes).unwrap(), c);
    assert_eq!(Compound::from_bytes(&[0]).unwrap(), Compound::default());
    assert!(Compound::from_bytes(&bytes[..bytes.len() - 3]).is_err());
    assert!(Compound::from_bytes(&[3, 0]).is_err());
}

#[test]
fn paths() {
    let mut c = Compound::parse("{a:{b:[10,20,30]},n:1}").unwrap();
    assert_eq!(c.get_path("a;b[1]"), Some(&Tag::Int(20)));
    assert!(c.set_path("a;b[1]", Tag::Int(99)));
    assert_eq!(c.get_path("a;b[1]"), Some(&Tag::Int(99)));
    assert!(!c.set_path("a;b[1]", Tag::Str("x".into())));
    assert!(c.set_path("x;y;z", Tag::Int(5)));
    assert_eq!(c.get_path("x;y;z"), Some(&Tag::Int(5)));
    assert!(!c.set_path("n;deeper", Tag::Int(1)));
    assert!(c.remove_path("a;b[0]"));
    assert_eq!(c.get_path("a;b[0]"), Some(&Tag::Int(99)));
    assert!(c.remove_path("x"));
    assert!(!c.remove_path("x"));
    assert!(c.get_path("a;;b").is_none());
}

#[test]
fn merge_is_deep() {
    let mut a = Compound::parse("{x:{p:1,q:2},y:1}").unwrap();
    a.merge(&Compound::parse("{x:{q:9,r:3},z:4}").unwrap());
    assert_eq!(a, Compound::parse("{x:{p:1,q:9,r:3},y:1,z:4}").unwrap());
}

#[test]
fn component_bytes() {
    let b = ItemComp::MaxStackSize.encode(&Value::Num(16.0)).unwrap().unwrap();
    assert_eq!(b, vec![16]);
    assert_eq!(ItemComp::MaxStackSize.decode(&b), Value::Num(16.0));
    let b = ItemComp::MaxStackSize
        .encode(&Value::Num(500.0))
        .unwrap()
        .unwrap();
    assert_eq!(ItemComp::MaxStackSize.decode(&b), Value::Num(99.0));
    let b = ItemComp::Rarity
        .encode(&Value::Text("Epic".into()))
        .unwrap()
        .unwrap();
    assert_eq!(b, vec![3]);
    assert_eq!(ItemComp::Rarity.decode(&b), Value::Text("epic".into()));
    assert!(ItemComp::Rarity.encode(&Value::Text("shiny".into())).is_err());
    let b = ItemComp::ItemModel
        .encode(&Value::Text("stick".into()))
        .unwrap()
        .unwrap();
    assert_eq!(
        ItemComp::ItemModel.decode(&b),
        Value::Text("minecraft:stick".into())
    );
    let b = ItemComp::CustomModelData
        .encode(&Value::Num(7.0))
        .unwrap()
        .unwrap();
    assert_eq!(ItemComp::CustomModelData.decode(&b), Value::Num(7.0));
    assert_eq!(
        ItemComp::GlintOverride.encode(&Value::Bool(true)).unwrap(),
        Some(vec![1])
    );
    assert_eq!(ItemComp::Unbreakable.encode(&Value::Bool(false)).unwrap(), None);
    assert_eq!(
        ItemComp::Unbreakable.encode(&Value::Bool(true)).unwrap(),
        Some(vec![])
    );
}

// ---- scripts ----

#[test]
fn nbt_from_text_and_tags() {
    let h = run("set {_n} to nbt from \"{a:1,b:\"\"hi\"\",c:{d:2.5}}\"\n\
         send \"%int tag \"\"a\"\" of {_n}%\"\n\
         send \"%string tag \"\"b\"\" of {_n}%\"\n\
         send \"%double tag \"\"c;d\"\" of {_n}%\"\n\
         send \"%tag \"\"a\"\" of {_n}%\"\n\
         send \"%string tag \"\"a\"\" of {_n}%\"");
    assert_eq!(h.calls, ["1", "hi", "2.5", "1", "<none>"]);
}

#[test]
fn set_and_delete_tags_in_a_variable() {
    let h = run("set {_n} to nbt from \"{}\"\n\
         set int tag \"a\" of {_n} to 5\n\
         set string tag \"deep;name\" of {_n} to \"x\"\n\
         set tag \"flag\" of {_n} to true\n\
         set boolean tag \"b\" of {_n} to true\n\
         send \"%{_n}%\"\n\
         delete tag \"a\" of {_n}\n\
         remove tag \"deep;name\" from {_n}\n\
         send \"%snbt of {_n}%\"\n\
         delete nbt of {_n}\n\
         send \"%{_n}%\"");
    assert_eq!(
        h.calls,
        [
            "{a:5,deep:{name:\"x\"},flag:1b,b:1b}",
            "{deep:{},flag:1b,b:1b}",
            "<none>"
        ]
    );
}

#[test]
fn typed_set_clamps_and_converts() {
    let h = run("set {_n} to nbt from \"{}\"\n\
         set byte tag \"a\" of {_n} to 300\n\
         set float tag \"f\" of {_n} to 1.5\n\
         set long tag \"l\" of {_n} to 4\n\
         set short tag \"s\" of {_n} to 2\n\
         set double tag \"d\" of {_n} to 2\n\
         send \"%{_n}%\"");
    assert_eq!(h.calls, ["{a:127b,f:1.5f,l:4L,s:2s,d:2d}"]);
}

#[test]
fn compounds_and_lists_as_values() {
    let h = run("set {_n} to nbt from \"{}\"\n\
         set compound tag \"c\" of {_n} to nbt from \"{x:1}\"\n\
         set list tag \"l\" of {_n} to \"[1,2,3]\"\n\
         set int array tag \"ia\" of {_n} to \"[I;4,5]\"\n\
         send \"%{_n}%\"\n\
         send \"%int tag \"\"c;x\"\" of {_n}%\"\n\
         send \"%int tag \"\"l[2]\"\" of {_n}%\"");
    assert_eq!(h.calls, ["{c:{x:1},l:[1,2,3],ia:[I;4,5]}", "1", "3"]);
}

#[test]
fn tag_names_and_merge() {
    let h = run("set {_n} to nbt from \"{a:1,b:2}\"\n\
         set {_k::*} to tag names of {_n}\n\
         send \"%tag names of {_n}%\"\n\
         add \"{b:9,c:3}\" to nbt of {_n}\n\
         merge nbt from \"{d:{e:1}}\" into {_n}\n\
         send \"%{_n}%\"");
    assert_eq!(h.calls, ["a and b", "{a:1,b:9,c:3,d:{e:1}}"]);
}

#[test]
fn has_tag_conditions() {
    let h = run("set {_n} to nbt from \"{a:1,s:\"\"x\"\"}\"\n\
         if {_n} has tag \"a\":\n\
         \x20   send \"yes a\"\n\
         if {_n} has int tag \"s\":\n\
         \x20   send \"bad\"\n\
         if {_n} has string tag \"s\":\n\
         \x20   send \"yes s\"\n\
         if {_n} does not have tag \"zzz\":\n\
         \x20   send \"no zzz\"\n\
         if {_n} doesn't have tag \"a\":\n\
         \x20   send \"bad2\"");
    assert_eq!(h.calls, ["yes a", "yes s", "no zzz"]);
}

#[test]
fn item_custom_data() {
    let h = run("set int tag \"power\" of nbt of player's tool to 7\n\
         set string tag \"id;name\" of nbt of player's tool to \"blade\"\n\
         send \"%int tag \"\"power\"\" of nbt of player's tool%\"\n\
         send \"%string tag \"\"id;name\"\" of player's tool%\"\n\
         send \"%nbt of player's tool%\"\n\
         if player's tool has tag \"power\":\n\
         \x20   send \"has\"\n\
         delete tag \"power\" of nbt of player's tool\n\
         send \"%nbt of player's tool%\"");
    assert_eq!(h.calls[0], "7");
    assert_eq!(h.calls[1], "blade");
    assert_eq!(h.calls[2], "{power:7,id:{name:\"blade\"}}");
    assert_eq!(h.calls[3], "has");
    assert_eq!(h.calls[4], "{id:{name:\"blade\"}}");
    // The stored bytes are real network NBT.
    let stored = Compound::from_bytes(&h.comps[&ItemComp::CustomData]).unwrap();
    assert_eq!(stored, Compound::parse("{id:{name:\"blade\"}}").unwrap());
}

#[test]
fn item_set_and_clear_all() {
    let h = run("set nbt of player's tool to nbt from \"{a:1}\"\n\
         add \"{b:2}\" to nbt of player's tool\n\
         send \"%nbt of player's tool%\"\n\
         delete nbt of player's tool\n\
         send \"%nbt of player's tool%\"");
    assert_eq!(h.calls, ["{a:1,b:2}", "{}"]);
    assert!(!h.comps.contains_key(&ItemComp::CustomData));
}

#[test]
fn no_item_means_none_and_no_change() {
    let h = run_with(
        Mock::new(false),
        "set int tag \"a\" of nbt of player's tool to 1\n\
         set custom model data of player's tool to 5\n\
         send \"%nbt of player's tool%\"\n\
         send \"%custom model data of player's tool%\"",
    );
    assert_eq!(h.calls, ["<none>", "<none>"]);
    assert!(h.comps.is_empty());
}

#[test]
fn components() {
    let h = run("set custom model data of player's tool to 42\n\
         set max stack size of player's tool to 16\n\
         set rarity of player's tool to epic\n\
         set item model of player's tool to \"my:model\"\n\
         set glint override of player's tool to true\n\
         set unbreakable of player's tool to true\n\
         send \"%custom model data of player's tool%\"\n\
         send \"%max stack size of player's tool%\"\n\
         send \"%rarity of player's tool%\"\n\
         send \"%item model of player's tool%\"\n\
         send \"%glint override of player's tool%\"\n\
         send \"%unbreakable of player's tool%\"\n\
         reset custom model data of player's tool\n\
         set unbreakable of player's tool to false\n\
         send \"%custom model data of player's tool%\"\n\
         send \"%unbreakable of player's tool%\"");
    assert_eq!(
        h.calls,
        ["42", "16", "epic", "my:model", "true", "true", "<none>", "false"]
    );
}

#[test]
fn tool_word_variants() {
    let h = run("set custom model data of tool of player to 3\n\
         send \"%custom model data of held item of player%\"");
    assert_eq!(h.calls, ["3"]);
}

#[test]
fn nbt_values_survive_in_global_variables() {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[(
        "t.carve".into(),
        "on join:\n    set {saved} to nbt from \"{a:1}\"\n".into(),
    )]);
    assert!(errs.is_empty());
    let mut h = Mock::new(true);
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let json = e.vars.to_json();
    let back = Variables::from_json(&json).unwrap();
    assert_eq!(
        back.get("saved"),
        Value::Nbt(Box::new(Compound::parse("{a:1}").unwrap()))
    );
}

#[test]
fn bad_input_errors() {
    assert!(bad("set rarity of {x} to epic"));
    assert!(bad("set tag \"a\" of 5 to 1"));
    assert!(bad("send \"%custom model data of 5%\""));
    // Normal lines still compile and behave.
    assert!(!bad("set {x} to 5"));
    assert!(!bad("set {x} to max(1, 2)"));
    assert!(!bad("delete {x}"));
    assert!(!bad("add 1 to {x}"));
    assert!(!bad("set max health of player to 30"));
}

#[test]
fn bad_snbt_is_a_runtime_error() {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[(
        "t.carve".into(),
        "on join:\n    set {_n} to nbt from \"{a:\"\n".into(),
    )]);
    assert!(errs.is_empty());
    let mut h = Mock::new(true);
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.errors.len(), 1, "{:?}", h.errors);
}

#[test]
fn host_without_components_reports_it() {
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
            true
        }
        fn schedule_wait(&mut self, _: u64, _: u64) {}
        fn now(&mut self) -> u64 {
            0
        }
        fn log(&mut self, level: LogLevel, msg: &str) {
            if level == LogLevel::Error {
                self.0.push(msg.into());
            }
        }
        fn held_item(&mut self, _: &PlayerRef) -> Option<(String, u32)> {
            Some(("stick".into(), 1))
        }
    }
    let mut e = Engine::default();
    let (_, errs) = e.load(&[(
        "t.carve".into(),
        "on join:\n    set custom model data of player's tool to 1\n".into(),
    )]);
    assert!(errs.is_empty());
    let mut h = Bare(Vec::new());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.0.len(), 1, "{:?}", h.0);
}
