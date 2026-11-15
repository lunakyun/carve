use std::collections::HashMap;

use carve_core::block_ops::BlockPos;
use carve_core::compile::{EventKind, check_registry};
use carve_core::conditions_flags_metadata::{self as c2, Query};
use carve_core::scoreboards_bossbars::BarFlag;
use carve_core::skbee_nbt::Compound;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    blocks: HashMap<(i32, i32, i32), String>,
    nbt: Option<Vec<u8>>,
    bees: Option<f64>,
    flags: Vec<BarFlag>,
    played: Option<bool>,
    responsive: Option<bool>,
    logs: Vec<(LogLevel, String)>,
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
    fn default_world(&mut self) -> Option<String> {
        Some("world".into())
    }
    fn block_type(&mut self, at: &BlockPos) -> Option<String> {
        Some(
            self.blocks
                .get(&(at.x, at.y, at.z))
                .cloned()
                .unwrap_or_else(|| "air".into()),
        )
    }
    fn block_entity_nbt(&mut self, _: &str, _: i32, _: i32, _: i32) -> Option<Vec<u8>> {
        self.nbt.clone()
    }
    fn w5c2_query(&mut self, q: Query<'_>) -> Option<Value> {
        match q {
            Query::BeeCount { .. } => self.bees.map(Value::Num),
            Query::BarFlag(id, f) => (id == "raid").then(|| Value::Bool(self.flags.contains(&f))),
            Query::PlayedBefore(u) => self.played.filter(|_| u == "u1").map(Value::Bool),
            Query::ToolRequired(b) => Some(Value::Bool(matches!(b, "stone" | "obsidian" | "cobweb"))),
            Query::Responsive(_) => self.responsive.map(Value::Bool),
        }
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
    let (_, errs) = e.load(&[("t.carve".to_string(), src.to_string())]);
    assert!(
        errs.is_empty(),
        "errors: {:?}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>()
    );
    e
}

fn fire(h: &mut Mock, src: &str, kind: EventKind, ctx: EventCtx) -> Vec<String> {
    let mut e = load(src);
    e.fire(kind, ctx, h);
    assert!(
        h.logs.iter().all(|(l, _)| *l != LogLevel::Error),
        "runtime errors: {:?}",
        h.logs
    );
    std::mem::take(&mut h.calls)
}

fn check_with(mut h: Mock, cond: &str) -> bool {
    let src = format!("on join:\n    if {cond}:\n        send \"yes\"\n    else:\n        send \"no\"\n");
    fire(&mut h, &src, EventKind::Join, EventCtx::player(steve()))[0] == "yes"
}

fn check(cond: &str) -> bool {
    check_with(Mock::default(), cond)
}

fn compile_error(src: &str) -> bool {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.to_string())]);
    !errs.is_empty()
}

#[test]
fn patterns_compile() {
    c2::check_patterns().unwrap();
    check_registry().unwrap();
}

#[test]
fn damage_cause() {
    let src = "on damage:\n    if damage was caused by fall:\n        send \"fall\"\n    if the damage wasn't caused by \"lava\":\n        send \"not lava\"\n    if damage is caused by void:\n        send \"void\"\n";
    let mut ctx = EventCtx::player(steve());
    ctx.damage_cause = Some("fall".into());
    let out = fire(&mut Mock::default(), src, EventKind::Damage, ctx);
    assert_eq!(out, vec!["fall", "not lava"]);
    let mut ctx = EventCtx::player(steve());
    ctx.damage_cause = Some("out_of_world".into());
    let out = fire(&mut Mock::default(), src, EventKind::Damage, ctx);
    assert_eq!(out, vec!["not lava", "void"]);
    // Only in damage events.
    assert!(compile_error(
        "on join:\n    if damage was caused by fall:\n        send \"x\"\n"
    ));
}

#[test]
fn cause_aliases() {
    assert!(c2::cause_matches("player_attack", "entity attack"));
    assert!(c2::cause_matches("arrow", "projectile"));
    assert!(c2::cause_matches("minecraft:drown", "drowning"));
    assert!(c2::cause_matches("in_wall", "in wall"));
    assert!(!c2::cause_matches("fall", "projectile"));
}

#[test]
fn egg_hatch_only_in_egg_event() {
    assert!(compile_error(
        "on join:\n    if egg will hatch:\n        send \"x\"\n"
    ));
    load(
        "on throwing of an egg:\n    if the egg will hatch:\n        send \"y\"\n    if egg won't hatch:\n        send \"n\"\n",
    );
}

#[test]
fn entity_storage() {
    let h = || Mock {
        bees: Some(3.0),
        ..Mock::default()
    };
    assert!(check_with(h(), "entity storage of location(1, 2, 3) is full"));
    assert!(!check_with(h(), "location(1, 2, 3)'s entity storage isn't full"));
    let two = Mock {
        bees: Some(2.0),
        ..Mock::default()
    };
    assert!(check_with(
        two,
        "the entity storage of location(1, 2, 3) is not full"
    ));
    // Not a hive: the game can't answer, so neither form is true.
    assert!(!check("entity storage of location(1, 2, 3) is full"));
    assert!(!check("entity storage of location(1, 2, 3) is not full"));
}

#[test]
fn equippable() {
    assert!(check("diamond helmet can be dispensed"));
    assert!(check("diamond_helmet can swap equipment"));
    assert!(check(
        "diamond helmet will lose durability when the wearer is hurt"
    ));
    assert!(check("diamond helmet can't be sheared off"));
    assert!(check("diamond helmet cannot be equipped on entities"));
    assert!(check("saddle can be sheared off"));
    assert!(check("saddle can be equipped onto entities"));
    assert!(check("carved pumpkin can't swap equipment"));
    assert!(check("elytra won't lose durability"));
    assert!(check("\"white_carpet\" can be sheared off"));
    // No equippable component.
    assert!(!check("stone can be dispensed"));
    assert!(check("stone cannot be dispensed"));
}

#[test]
fn tables_sorted() {
    let mut last = "";
    for (id, _) in c2::EQUIPPABLE {
        assert!(*id > last, "{id} out of order");
        last = id;
    }
    assert_eq!(
        c2::equippable("netherite_boots").map(|f| f & c2::EQ_DISPENSABLE),
        Some(1)
    );
}

#[test]
fn bossbar_flags() {
    let h = || Mock {
        flags: vec![BarFlag::DarkenSky],
        ..Mock::default()
    };
    assert!(check_with(h(), "bossbar \"raid\" has darken sky flag"));
    assert!(check_with(h(), "bossbar \"raid\" has the flag darken sky"));
    assert!(!check_with(h(), "bossbar \"raid\" has fog flag"));
    assert!(check_with(h(), "bossbar \"raid\" doesn't have the fog flag"));
    assert!(check_with(h(), "bossbar \"raid\" does not have boss music flag"));
    // Unknown bar: no answer.
    assert!(!check_with(h(), "bossbar \"x\" has darken sky flag"));
    assert!(!check_with(h(), "bossbar \"x\" doesn't have darken sky flag"));
}

#[test]
fn tooltips() {
    assert!(check("tooltip of diamond is shown"));
    assert!(check("the entire tooltip of diamond is shown"));
    assert!(!check("additional tooltip of diamond is hidden"));
    assert!(check("diamond has an additional tooltip"));
    assert!(!check("diamond doesn't have an entire tooltip"));
}

#[test]
fn loot_table() {
    let with = Compound::parse("{LootTable:\"minecraft:chests/simple_dungeon\"}").unwrap();
    let h = Mock {
        nbt: Some(with.to_bytes().unwrap()),
        ..Mock::default()
    };
    assert!(check_with(h, "location(1, 2, 3) has a loot table"));
    let empty = Compound::parse("{Items:[]}").unwrap();
    let h = Mock {
        nbt: Some(empty.to_bytes().unwrap()),
        ..Mock::default()
    };
    assert!(check_with(h, "location(1, 2, 3) doesn't have a loottable"));
    // No block entity.
    assert!(check("location(1, 2, 3) does not have loot table"));
    assert!(!check("location(1, 2, 3) has loot table"));
}

#[test]
fn metadata_and_persistent_data() {
    let src = "on join:\n    if player has metadata \"k\":\n        send \"early\"\n    set metadata value \"k\" of player to 5\n    if player has metadata value \"K\":\n        send \"md\"\n    if player doesn't have metadata tag \"other\":\n        send \"no other\"\n    set persistent data value \"p\" of player to \"x\"\n    if player has persistent data tag \"p\":\n        send \"pd\"\n    if player has data tag \"p\":\n        send \"pd2\"\n    delete persistent data value \"p\" of player\n    if player does not have persistent data tag \"p\":\n        send \"pd gone\"\n    if player has persistent data tag \"k\":\n        send \"mixed\"\n";
    let out = fire(
        &mut Mock::default(),
        src,
        EventKind::Join,
        EventCtx::player(steve()),
    );
    assert_eq!(out, vec!["md", "no other", "pd", "pd2", "pd gone"]);
}

#[test]
fn played_before() {
    let yes = Mock {
        played: Some(true),
        ..Mock::default()
    };
    assert!(check_with(yes, "player has played before"));
    let no = Mock {
        played: Some(false),
        ..Mock::default()
    };
    assert!(check_with(no, "player hasn't played on this server before"));
    let no = Mock {
        played: Some(false),
        ..Mock::default()
    };
    assert!(!check_with(no, "player has already played on the server before"));
    // Unknown: neither.
    assert!(!check("player has played before"));
    assert!(!check("player has not played before"));
}

#[test]
fn preferred_tool() {
    let mut blocks = HashMap::new();
    blocks.insert((1, 2, 3), "stone".to_string());
    let h = |b: &HashMap<(i32, i32, i32), String>| Mock {
        blocks: b.clone(),
        ..Mock::default()
    };
    assert!(check_with(
        h(&blocks),
        "wooden pickaxe is the preferred tool for location(1, 2, 3)"
    ));
    assert!(check_with(
        h(&blocks),
        "diamond_shovel isn't the preferred tool of location(1, 2, 3)"
    ));
    assert!(check_with(h(&blocks), "stick is not preferred tool for stone"));
    // Dirt drops with anything.
    assert!(check_with(h(&blocks), "stick is preferred tool for dirt"));
    // Obsidian needs a diamond pickaxe.
    assert!(check_with(
        h(&blocks),
        "iron pickaxe is not the preferred tool for obsidian"
    ));
    assert!(check_with(
        h(&blocks),
        "diamond pickaxe is the preferred tool for obsidian"
    ));
    assert!(check_with(
        h(&blocks),
        "shears are the preferred tools for cobweb"
    ));
    assert!(c2::correct_for_drops("netherite_pickaxe", "ancient_debris"));
    assert!(!c2::correct_for_drops("stone_pickaxe", "diamond_ore"));
}

#[test]
fn responsive() {
    let src = "on join:\n    spawn an interaction at location(0, 0, 0)\n    if last spawned entity is responsive:\n        send \"r\"\n    if last spawned entity isn't responsive:\n        send \"not r\"\n";
    // The mock has no entities: `last spawned entity` is none, which is never responsive.
    let out = fire(
        &mut Mock::default(),
        src,
        EventKind::Join,
        EventCtx::player(steve()),
    );
    assert_eq!(out, vec!["not r"]);
    assert!(!check("player is responsive"));
}

#[test]
fn plain_lines_skip_the_table() {
    // Ordinary conditions still compile and run as before.
    assert!(check("1 is 1"));
    assert!(check("\"abc\" contains \"b\""));
}

#[test]
fn doc_forms_compile() {
    load(
        "on join:\n    set {_loc} to location(1, 2, 3)\n    if entity storage of block at {_loc} is full:\n        stop\n    if block at {_loc} has a loot table:\n        stop\n    if diamond pickaxe is the preferred tool for block at {_loc}:\n        stop\n    if player's tool is {_loc}'s preferred tool:\n        stop\n    if tooltip of player's tool is shown:\n        stop\n    if player does not have data tag \"kit\":\n        stop\n",
    );
}

#[test]
fn ingame_script_loads() {
    let src =
        include_str!("../../../tests/ingame/areas/conditions-flags-metadata/conditions-flags-metadata.carve");
    load(src);
}
