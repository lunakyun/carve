use std::collections::HashMap;

use carve_core::block_ops::BlockPos;
use carve_core::compile::{EventKind, check_registry};
use carve_core::conditions_entity_checks::{Check, max_stack};
use carve_core::expr_vectors_server::Query;
use carve_core::skbee_nbt::Compound;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    /// slot -> (item, count)
    inv: HashMap<i64, (String, u32)>,
    mode: Option<String>,
    blocks: HashMap<(i32, i32, i32), String>,
    cooldown: Option<i64>,
    hidden: bool,
    checks: Vec<Check>,
    sign_nbt: Option<Vec<u8>>,
    dimension: Option<String>,
    cmd: bool,
    logs: Vec<(LogLevel, String)>,
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        let who = match to {
            Recipient::Player(p) => p.name.clone(),
            Recipient::Console => "console".into(),
        };
        self.calls.push(format!("send {who} {text}"));
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
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        name.eq_ignore_ascii_case("steve").then(steve)
    }
    fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
    fn player_location(&mut self, p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        (p.uuid == "u1").then(|| (0.5, 64.0, 0.5, "world".to_string()))
    }
    fn default_world(&mut self) -> Option<String> {
        Some("world".into())
    }
    fn gamemode(&mut self, _: &PlayerRef) -> Option<String> {
        self.mode.clone()
    }
    fn inv_get(&mut self, _: &PlayerRef, _: bool, slot: i64) -> Option<(String, u32)> {
        Some(self.inv.get(&slot).cloned().unwrap_or_else(|| ("air".into(), 0)))
    }
    fn can_see(&mut self, _: &PlayerRef, _: &PlayerRef) -> bool {
        !self.hidden
    }
    fn item_cooldown(&mut self, _: &PlayerRef, _: &str) -> Option<i64> {
        self.cooldown
    }
    fn block_type(&mut self, at: &BlockPos) -> Option<String> {
        Some(
            self.blocks
                .get(&(at.x, at.y, at.z))
                .cloned()
                .unwrap_or_else(|| "air".into()),
        )
    }
    fn block_trait(&mut self, id: &str, t: carve_core::conditions_entity_block::BlockTrait) -> Option<bool> {
        match t {
            carve_core::conditions_entity_block::BlockTrait::Solid => Some(id == "stone"),
            _ => Some(false),
        }
    }
    fn block_entity_nbt(&mut self, _: &str, _: i32, _: i32, _: i32) -> Option<Vec<u8>> {
        self.sign_nbt.clone()
    }
    fn server_query(&mut self, q: Query<'_>) -> Option<Value> {
        match q {
            Query::Environment(_) => self.dimension.clone().map(Value::Text),
            _ => None,
        }
    }
    fn held_component(&mut self, _: &PlayerRef, _: carve_core::skbee_nbt::ItemComp) -> Option<Vec<u8>> {
        self.cmd.then(|| vec![1])
    }
    fn entity_check(&mut self, c: Check, _: &Value) -> Option<bool> {
        self.checks.contains(&c).then_some(true).or(Some(false))
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn run_with(mut h: Mock, src: &str) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".to_string(), src.to_string())]);
    assert!(
        errs.is_empty(),
        "errors: {:?}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>()
    );
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(
        h.logs.iter().all(|(l, _)| *l != LogLevel::Error),
        "runtime errors: {:?}",
        h.logs
    );
    h.calls
}

fn check_with(h: Mock, cond: &str) -> bool {
    let src = format!("on join:\n    if {cond}:\n        send \"yes\"\n    else:\n        send \"no\"\n");
    run_with(h, &src)[0].ends_with("yes")
}

fn check(cond: &str) -> bool {
    check_with(Mock::default(), cond)
}

fn compile_error(cond: &str) -> bool {
    let src = format!("on join:\n    if {cond}:\n        send \"yes\"\n");
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    !errs.is_empty()
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn alphanumeric() {
    assert!(check("\"abc123\" is alphanumeric"));
    assert!(!check("\"abc 123\" is alphanumeric"));
    assert!(!check("\"abc!\" is alphanumeric"));
    assert!(check("\"abc!\" isn't alphanumeric"));
    assert!(check("\"a b\" is not alphanumeric"));
    assert!(!check("\"\" is alphanumeric"));
    assert!(!check("5 is alphanumeric"));
}

#[test]
fn event_cancelled() {
    let src = "on join:\n    if event is cancelled:\n        send \"early\"\n    cancel event\n    if the event is cancelled:\n        send \"cancelled\"\n    if event isn't cancelled:\n        send \"live\"\n";
    let calls = run_with(Mock::default(), src);
    assert_eq!(calls.len(), 1);
    assert!(calls[0].ends_with("cancelled"));
}

#[test]
fn can_hold() {
    let mut h = Mock::default();
    // Everything empty: room for a lot.
    assert!(check_with(Mock::default(), "player can hold 64 of stone"));
    assert!(check_with(
        Mock::default(),
        "player's inventory can hold 3 of diamond_sword"
    ));
    // Full inventory of dirt: no room for stone, room only for dirt up to a stack.
    for s in 0..36 {
        h.inv.insert(s, ("dirt".into(), 64));
    }
    assert!(!check_with(
        Mock {
            inv: h.inv.clone(),
            ..Mock::default()
        },
        "player can hold 1 of stone"
    ));
    assert!(check_with(
        Mock {
            inv: h.inv.clone(),
            ..Mock::default()
        },
        "player can't hold 1 of stone"
    ));
    h.inv.insert(5, ("dirt".into(), 60));
    assert!(check_with(
        Mock {
            inv: h.inv.clone(),
            ..Mock::default()
        },
        "player can hold 4 of dirt"
    ));
    assert!(!check_with(
        Mock {
            inv: h.inv.clone(),
            ..Mock::default()
        },
        "player can hold 5 of dirt"
    ));
    // Swords stack to one, so two swords need two empty slots.
    h.inv.insert(6, ("air".into(), 0));
    assert!(check_with(
        Mock {
            inv: h.inv.clone(),
            ..Mock::default()
        },
        "player can hold diamond_sword"
    ));
    assert!(!check_with(
        Mock {
            inv: h.inv.clone(),
            ..Mock::default()
        },
        "player can hold 2 of diamond_sword"
    ));
}

#[test]
fn stack_sizes() {
    assert_eq!(max_stack("stone"), 64);
    assert_eq!(max_stack("diamond_sword"), 1);
    assert_eq!(max_stack("ender_pearl"), 16);
    assert_eq!(max_stack("oak_sign"), 16);
    assert_eq!(max_stack("water_bucket"), 1);
    assert_eq!(max_stack("bucket"), 16);
}

#[test]
fn build_and_pick_up_follow_game_mode() {
    let mode = |m: &str| Mock {
        mode: Some(m.into()),
        ..Mock::default()
    };
    assert!(check_with(
        mode("survival"),
        "player can build at location(1, 2, 3)"
    ));
    assert!(!check_with(
        mode("adventure"),
        "player can build at location(1, 2, 3)"
    ));
    assert!(check_with(
        mode("adventure"),
        "player can't build location(1, 2, 3)"
    ));
    assert!(check_with(mode("creative"), "player can pick up items"));
    assert!(!check_with(mode("spectator"), "player can pick up items"));
    assert!(check_with(mode("spectator"), "player can't pick up items"));
    // Unknown game mode: the answer is false, even when negated.
    assert!(!check_with(Mock::default(), "player can pick up items"));
    assert!(!check_with(Mock::default(), "player can't pick up items"));
}

#[test]
fn can_see_players() {
    assert!(check("player can see player"));
    assert!(!check_with(
        Mock {
            hidden: true,
            ..Mock::default()
        },
        "player can see player"
    ));
    assert!(check_with(
        Mock {
            hidden: true,
            ..Mock::default()
        },
        "player can't see player"
    ));
}

#[test]
fn line_of_sight() {
    let mut wall = Mock::default();
    for y in 60..70 {
        wall.blocks.insert((5, y, 0), "stone".into());
    }
    let open = "location(0.5, 65, 0.5, \"world\") has line of sight to location(10.5, 65, 0.5, \"world\")";
    assert!(check(open));
    assert!(!check_with(wall, open));
    assert!(check(
        "location(0.5, 65, 0.5, \"world\") doesn't have line of sight to location(0.5, 65, 300.5, \"world\")"
    ));
    let mut glass = Mock::default();
    glass.blocks.insert((5, 65, 0), "glass".into());
    assert!(check_with(glass, open));
    assert!(check(
        "player has a direct line of sight to location(3.5, 65, 0.5, \"world\")"
    ));
}

#[test]
fn item_cooldown() {
    let cd = |t| Mock {
        cooldown: Some(t),
        ..Mock::default()
    };
    assert!(check_with(cd(20), "player has item cooldown for ender_pearl"));
    assert!(check_with(cd(20), "player has a cooldown on ender_pearl"));
    assert!(!check_with(cd(0), "player has item cooldown for ender_pearl"));
    assert!(check_with(
        cd(0),
        "player doesn't have item cooldown for ender_pearl"
    ));
    assert!(!check("player has item cooldown for ender_pearl"));
}

#[test]
fn sedated_hive() {
    let at = "location(1.5, 70, 1.5, \"world\")";
    let mut hive = Mock::default();
    hive.blocks.insert((1, 70, 1), "bee_nest".into());
    hive.blocks.insert((1, 67, 1), "campfire".into());
    assert!(check_with(hive, &format!("{at} is sedated")));
    let mut cold = Mock::default();
    cold.blocks.insert((1, 70, 1), "beehive".into());
    assert!(!check_with(cold, &format!("{at} is sedated")));
    let mut other = Mock::default();
    other.blocks.insert((1, 70, 1), "stone".into());
    other.blocks.insert((1, 69, 1), "campfire".into());
    assert!(check_with(other, &format!("{at} isn't sedated")));
}

#[test]
fn respawn_anchors() {
    let dim = |d: &str| Mock {
        dimension: Some(d.into()),
        ..Mock::default()
    };
    assert!(check_with(
        dim("minecraft:the_nether"),
        "respawn anchors work in world \"nether\""
    ));
    assert!(!check_with(
        dim("minecraft:overworld"),
        "respawn anchors work in world \"world\""
    ));
    assert!(check_with(
        dim("minecraft:overworld"),
        "respawn anchors don't work in world \"world\""
    ));
    assert!(!check("respawn anchors work in world \"world\""));
}

#[test]
fn glowing_text() {
    let nbt = |on: i8| {
        let c = Compound::parse(&format!(
            "{{front_text:{{has_glowing_text:{on}b}},back_text:{{has_glowing_text:0b}}}}"
        ))
        .unwrap();
        c.to_bytes().unwrap()
    };
    let at = "location(1, 70, 1, \"world\")";
    assert!(check_with(
        Mock {
            sign_nbt: Some(nbt(1)),
            ..Mock::default()
        },
        &format!("{at} has glowing text")
    ));
    assert!(!check_with(
        Mock {
            sign_nbt: Some(nbt(0)),
            ..Mock::default()
        },
        &format!("{at} has glowing text")
    ));
    assert!(check_with(
        Mock {
            sign_nbt: Some(nbt(0)),
            ..Mock::default()
        },
        &format!("{at} doesn't have glowing text")
    ));
}

#[test]
fn custom_model_data() {
    assert!(check_with(
        Mock {
            cmd: true,
            ..Mock::default()
        },
        "player has custom model data"
    ));
    assert!(!check("player has a custom model data"));
    assert!(check("player doesn't have custom model data"));
}

#[test]
fn game_answered_checks() {
    let yes = |c| Mock {
        checks: vec![c],
        ..Mock::default()
    };
    assert!(check_with(yes(Check::ChatColors), "player can see chat colors"));
    assert!(!check("player can see chat colours"));
    assert!(check_with(yes(Check::ChatMessages), "player can see messages"));
    assert!(check_with(
        yes(Check::ChatMessages),
        "player can see chat messages"
    ));
    assert!(check_with(yes(Check::ChatFiltering), "player has chat filtering"));
    assert!(check("player doesn't have chat filtering"));
    assert!(check_with(yes(Check::ClientWeather), "player has custom weather"));
    assert!(check_with(yes(Check::ClientWeather), "player has client weather"));
    assert!(check_with(yes(Check::InLiquid), "player is in liquid"));
    assert!(check_with(yes(Check::Wet), "player is wet"));
    assert!(check("player isn't wet"));
    assert!(check_with(yes(Check::Sheared), "player is sheared"));
    assert!(check_with(yes(Check::HasAi), "player has ai"));
    assert!(check_with(yes(Check::CanBreed), "player can breed"));
    assert!(check_with(yes(Check::Stared), "player has been stared at"));
    assert!(check_with(
        yes(Check::BellRinging),
        "location(1, 2, 3) is ringing"
    ));
}

/// A host that cannot answer: positive and negated forms are both false.
struct Silent(Mock);

impl Host for Silent {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        self.0.send_message(to, text);
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
        self.0.resolve_player(uuid)
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        self.0.find_player(name)
    }
    fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, _: LogLevel, _: &str) {}
}

#[test]
fn unknown_is_false_both_ways() {
    for cond in [
        "player is sheared",
        "player isn't sheared",
        "player has ai",
        "player doesn't have ai",
    ] {
        let src = format!("on join:\n    if {cond}:\n        send \"yes\"\n    else:\n        send \"no\"\n");
        let mut e = Engine::default();
        let (_, errs) = e.load(&[("t.carve".to_string(), src)]);
        assert!(errs.is_empty());
        let mut h = Silent(Mock::default());
        e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
        assert!(h.0.calls[0].ends_with("no"), "{cond}");
    }
}

#[test]
fn damage_cause_condition_works() {
    // `damage cause` is an expression in `on damage`; the condition is a comparison.
    let src = "on damage:\n    if damage cause is \"fall\":\n        send \"ouch\"\n";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".to_string(), src.to_string())]);
    assert!(
        errs.is_empty(),
        "{:?}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>()
    );
}

#[test]
fn bad_syntax_reports_errors() {
    assert!(compile_error("player can hold"));
    assert!(compile_error("player has line of sight"));
    assert!(compile_error("respawn anchors work"));
}
