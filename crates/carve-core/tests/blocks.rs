use std::collections::HashMap;

use carve_core::block_ops::BlockPos;
use carve_core::compile::{EventKind, compile_file};
use carve_core::*;

/// A tiny world: a map of blocks. Everything else is empty.
#[derive(Default)]
struct Mock {
    blocks: HashMap<(i32, i32, i32), String>,
    msgs: Vec<String>,
    logs: Vec<(LogLevel, String)>,
}

impl Mock {
    fn new() -> Self {
        let mut m = Mock::default();
        // Steve stands at (1.5, 64, -3.0), so his block is (1, 64, -3).
        m.blocks.insert((1, 63, -3), "grass_block".into());
        m.blocks.insert((1, 64, -3), "air".into());
        m.blocks.insert((3, 70, 4), "obsidian".into());
        m.blocks.insert((10, 5, 10), "stone".into());
        m
    }
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.msgs.push(text.to_string());
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
    fn find_player(&mut self, _name: &str) -> Option<PlayerRef> {
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
        Some((1.5, 64.0, -3.0, "world".to_string()))
    }
    fn block_type(&mut self, at: &BlockPos) -> Option<String> {
        Some(
            self.blocks
                .get(&(at.x, at.y, at.z))
                .cloned()
                .unwrap_or_else(|| "air".into()),
        )
    }
    fn set_block(&mut self, at: &BlockPos, id: &str) {
        self.blocks.insert((at.x, at.y, at.z), id.to_string());
    }
    fn break_block(&mut self, at: &BlockPos) {
        self.blocks.insert((at.x, at.y, at.z), "air".into());
    }
    fn targeted_block(&mut self, _p: &PlayerRef, _max: f64) -> Option<BlockPos> {
        Some(BlockPos {
            x: 3,
            y: 70,
            z: 4,
            world: Some("world".into()),
        })
    }
}

/// Only the required methods, so the block defaults run.
#[derive(Default)]
struct Bare {
    cmds: Vec<String>,
}

impl Host for Bare {
    fn send_message(&mut self, _: &Recipient, _: &str) {}
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.cmds.push(c.to_string());
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
        Some((1.5, 64.0, -3.0, "world".to_string()))
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn script(body: &str) -> String {
    let lines: String = body.lines().map(|l| format!("    {l}\n")).collect();
    format!("on join:\n{lines}")
}

fn errors_of(body: &str) -> Vec<ScriptError> {
    compile_file("bad.carve", &script(body)).err().unwrap_or_default()
}

fn load(body: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("test.carve".into(), script(body))]);
    assert!(
        errs.is_empty(),
        "errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn run_with(body: &str, h: &mut impl Host) {
    let mut e = load(body);
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
}

fn run(body: &str) -> Mock {
    let mut h = Mock::new();
    run_with(body, &mut h);
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errors.is_empty(), "runtime errors: {errors:?}");
    h
}

#[test]
fn type_of_block_at_location() {
    let h = run("send \"%type of block at location(10, 5, 10)%\" to player");
    assert_eq!(h.msgs, vec!["stone"]);
}

#[test]
fn block_at_floors_coordinates() {
    let h = run("send \"%type of block at location(10.9, 5.2, 10.5)%\" to player");
    assert_eq!(h.msgs, vec!["stone"]);
}

#[test]
fn location_at_xyz_in_world() {
    let h = run("set {_l} to location at 10, 5, 10 in world \"world\"\n\
         send \"%type of block at {_l}%\" to player\n\
         send \"%world of {_l}%\" to player");
    assert_eq!(h.msgs, vec!["stone", "world"]);
}

#[test]
fn location_at_xyz_without_world() {
    let h = run("send \"%x of location at 1, 2, 3%\" to player");
    assert_eq!(h.msgs, vec!["1"]);
}

#[test]
fn block_below_and_above_player() {
    let h = run("send \"%type of block below player%\" to player\n\
         send \"%type of block above player%\" to player\n\
         send \"%type of block at player%\" to player");
    assert_eq!(h.msgs, vec!["grass block", "air", "air"]);
}

#[test]
fn block_compass_directions() {
    let mut h = Mock::new();
    h.blocks.insert((2, 64, -3), "dirt".into());
    h.blocks.insert((1, 64, -4), "sand".into());
    run_with(
        "send \"%type of block east of player%\" to player\n\
         send \"%type of block north of player%\" to player",
        &mut h,
    );
    assert_eq!(h.msgs, vec!["dirt", "sand"]);
}

#[test]
fn targeted_block_of_player() {
    let h = run("send \"%type of targeted block of player%\" to player\n\
         send \"%type of player's targeted block%\" to player\n\
         send \"%type of targeted block%\" to player");
    assert_eq!(h.msgs, vec!["obsidian", "obsidian", "obsidian"]);
}

#[test]
fn block_in_variable() {
    let h = run("set {_b} to block below player\n\
         send \"%type of {_b}%\" to player\n\
         set block {_b} to stone\n\
         send \"%type of {_b}%\" to player\n\
         send \"%y of {_b}%\" to player");
    assert_eq!(h.msgs, vec!["grass block", "stone", "63"]);
}

#[test]
fn set_block_and_read_back() {
    let h = run("set block at location(5, 70, 5) to oak planks");
    assert_eq!(h.blocks.get(&(5, 70, 5)).map(String::as_str), Some("oak_planks"));
}

#[test]
fn set_block_relative_to_player() {
    let h = run("set block below player to diamond block");
    assert_eq!(
        h.blocks.get(&(1, 63, -3)).map(String::as_str),
        Some("diamond_block")
    );
}

#[test]
fn set_targeted_block_to_air() {
    let h = run("set targeted block of player to air");
    assert_eq!(h.blocks.get(&(3, 70, 4)).map(String::as_str), Some("air"));
}

#[test]
fn break_block() {
    let h = run("break block at location(10, 5, 10)");
    assert_eq!(h.blocks.get(&(10, 5, 10)).map(String::as_str), Some("air"));
    let h = run("destroy targeted block of player");
    assert_eq!(h.blocks.get(&(3, 70, 4)).map(String::as_str), Some("air"));
}

#[test]
fn block_is_condition() {
    let h = run(
        "if block below player is grass block:\n    send \"grass\" to player\n\
         if block below player is not stone:\n    send \"not stone\" to player\n\
         if block at location(10, 5, 10) is dirt or stone:\n    send \"either\" to player\n\
         if block above player is dirt:\n    send \"wrong\" to player\n\
         if targeted block of player is not air:\n    send \"target\" to player",
    );
    assert_eq!(h.msgs, vec!["grass", "not stone", "either", "target"]);
}

#[test]
fn block_is_else() {
    let h = run("if block above player is stone:\n    send \"a\" to player\nelse:\n    send \"b\" to player");
    assert_eq!(h.msgs, vec!["b"]);
}

#[test]
fn type_compares_in_equality() {
    let h = run("if type of block at location(10, 5, 10) is stone:\n    send \"ok\" to player");
    assert_eq!(h.msgs, vec!["ok"]);
}

#[test]
fn block_in_a_loop() {
    let h = run("loop 3 times:\n    set block at location(loop-value, 80, 0) to stone");
    for x in 1..=3 {
        assert_eq!(h.blocks.get(&(x, 80, 0)).map(String::as_str), Some("stone"));
    }
}

#[test]
fn bad_block_type_is_an_error() {
    let errs = errors_of("set block at location(1, 2, 3) to 5");
    assert!(!errs.is_empty());
}

#[test]
fn block_at_needs_a_place() {
    assert!(!errors_of("send \"%type of block at 5%\" to player").is_empty());
}

#[test]
fn text_with_odd_characters_is_not_a_block() {
    let mut h = Mock::new();
    run_with("set block at location(1, 2, 3) to \"stone; op Steve\"", &mut h);
    assert!(!h.blocks.contains_key(&(1, 2, 3)));
    assert!(h.logs.iter().any(|(l, _)| *l == LogLevel::Error));
}

#[test]
fn out_of_range_coordinates_do_nothing() {
    let mut h = Mock::new();
    let before = h.blocks.len();
    run_with("set block at location(99999999999, 2, 3) to stone", &mut h);
    assert_eq!(h.blocks.len(), before);
    assert!(h.logs.iter().any(|(l, _)| *l == LogLevel::Warn));
}

#[test]
fn defaults_use_setblock_commands() {
    let mut h = Bare::default();
    run_with(
        "set block at location(1, 2, 3) to stone\nbreak block at location(4, 5, 6)",
        &mut h,
    );
    assert_eq!(
        h.cmds,
        vec![
            "setblock 1 2 3 minecraft:stone",
            "setblock 4 5 6 minecraft:air destroy"
        ]
    );
}

#[test]
fn defaults_cannot_read_blocks() {
    let mut h = Bare::default();
    run_with(
        "if block below player is stone:\n    set block above player to dirt",
        &mut h,
    );
    assert!(h.cmds.is_empty());
}

#[test]
fn negative_coordinates_floor_down() {
    let h = run("set block at location(-0.5, 10, -1.5) to stone");
    assert_eq!(h.blocks.get(&(-1, 10, -2)).map(String::as_str), Some("stone"));
}
