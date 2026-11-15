use carve_core::compile::{EventKind, compile_file};
use carve_core::parsing_loot_components::Q;
use carve_core::*;

#[derive(Default)]
struct Mock {
    sent: Vec<String>,
    errors: Vec<String>,
    stack: Option<f64>,
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
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
    fn find_player(&mut self, n: &str) -> Option<PlayerRef> {
        (n == "Steve").then(steve)
    }
    fn has_permission(&mut self, _p: &PlayerRef, _n: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _t: u64, _k: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn game_fact(&mut self, q: Q<'_>) -> Option<Value> {
        match q {
            Q::MaxStack("ender_pearl") => self.stack.map(Value::Num),
            Q::MaxStack(_) => None,
        }
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        if matches!(level, LogLevel::Error) {
            self.errors.push(msg.to_string());
        }
    }
}

fn run_with(h: &mut Mock, body: &str) -> Vec<String> {
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
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
    assert!(h.errors.is_empty(), "{:?}", h.errors);
    std::mem::take(&mut h.sent)
}

fn run(body: &str) -> Vec<String> {
    run_with(&mut Mock::default(), body)
}

#[test]
fn parsed_as_integer_boolean_player() {
    let out = run(
        "send \"%\"12\" parsed as integer%|%\"1.5\" parsed as integer%|%\"-3\" parsed as integer%\"\n\
         send \"%\"yes\" parsed as boolean%|%\"off\" parsed as boolean%|%\"maybe\" parsed as boolean%\"\n\
         send \"%\"Steve\" parsed as player%|%\"Nobody\" parsed as player%\"",
    );
    assert_eq!(out, vec!["12|<none>|-3", "true|false|<none>", "Steve|<none>"]);
}

#[test]
fn parsed_as_number_still_works() {
    assert_eq!(run("send \"%\"2.5\" parsed as number%\""), vec!["2.5"]);
}

#[test]
fn formatted_human_readable() {
    let out = run("set {_t} to 65 minutes\n\
         send \"%{_t} formatted human-readable%|%{_t} formatted%\"");
    assert_eq!(out, vec!["1 hour and 5 minutes|1:05:00"]);
}

#[test]
fn keys_of_items_and_loot_tables() {
    let out = run(
        "send \"%key of diamond%|%key of loot table from key \"chests/simple_dungeon\"%|%key of loot table from key \"x:y\"%\"",
    );
    assert_eq!(out, vec!["minecraft:diamond|minecraft:chests/simple_dungeon|x:y"]);
}

#[test]
fn loot_context_parts() {
    let out = run(
        "set {_c} to loot context at location(1, 64, 2, \"world\") with luck 3 with looter player\n\
         send \"%loot luck of {_c}%|%looter of {_c}%|%looted entity of {_c}%\"\n\
         send \"%loot location of {_c}%\"",
    );
    assert_eq!(out[0], "3|Steve|<none>");
    assert!(out[1].contains("64"), "{}", out[1]);
}

#[test]
fn max_stack_size_asks_the_server_first() {
    let mut h = Mock {
        stack: Some(8.0),
        ..Mock::default()
    };
    assert_eq!(
        run_with(&mut h, "send \"%max stack size of ender pearl%\""),
        vec!["8"]
    );
    // Without a host answer the built-in table is used.
    assert_eq!(
        run("send \"%max stack size of ender pearl%|%max stack size of diamond sword%\""),
        vec!["16|1"]
    );
}

#[test]
fn item_with_components_compiles() {
    let src = "on join:\n    give diamond sword with custom model data 7 to player\n    give stick with enchantment glint true to player\n    give stick with item model \"minecraft:blaze_rod\" to player\n";
    assert!(compile_file("t.carve", src).is_ok());
}
