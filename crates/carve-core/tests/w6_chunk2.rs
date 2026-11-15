use carve_core::compile::EventKind;
use carve_core::w6_chunk2::{self as w6, Axis2, HostOp};
use carve_core::world_effects::FxPos;
use carve_core::*;

#[derive(Default)]
struct Mock {
    msgs: Vec<String>,
    commands: Vec<String>,
    ops: Vec<String>,
    logs: Vec<(LogLevel, String)>,
    next: u32,
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        let who = match to {
            Recipient::Player(p) => p.name.clone(),
            Recipient::Console => "console".into(),
        };
        self.msgs.push(format!("{who}: {text}"));
    }
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.commands.push(c.to_string());
    }
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "00000001-0000-0000-0000-000000000002").then(|| "Steve".to_string())
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
    fn online_players(&mut self) -> Vec<PlayerRef> {
        vec![steve()]
    }
    fn player_location(&mut self, _: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((0.0, 64.0, 0.0, "world".to_string()))
    }
    fn default_world(&mut self) -> Option<String> {
        Some("world".into())
    }
    fn spawn_entity(&mut self, kind: &str, at: &FxPos) -> Option<EntityRef> {
        self.next += 1;
        self.ops.push(format!("spawn {kind} {} {} {}", at.x, at.y, at.z));
        Some(EntityRef {
            uuid: format!("0000000{}-0000-0000-0000-00000000000a", self.next),
            kind: kind.into(),
        })
    }
    fn entity_location(&mut self, _: &EntityRef) -> Option<(f64, f64, f64, String)> {
        Some((10.0, 70.0, -3.0, "world".into()))
    }
    fn entity_name(&mut self, _: &EntityRef) -> Option<String> {
        Some("Bob".into())
    }
    fn entity_health(&mut self, _: &EntityRef) -> Option<f64> {
        Some(12.0)
    }
    fn set_entity_name(&mut self, e: &EntityRef, n: Option<&str>) {
        self.ops.push(format!("name {} {}", e.kind, n.unwrap_or("-")));
    }
    fn set_entity_health(&mut self, e: &EntityRef, h: f64) {
        self.ops.push(format!("health {} {h}", e.kind));
    }
    fn entity_remove(&mut self, e: &EntityRef) {
        self.ops.push(format!("remove {}", e.kind));
    }
    fn w6c2(&mut self, op: HostOp<'_>) -> Option<bool> {
        self.ops.push(format!("{op:?}").replace(
            "PlayerRef { uuid: \"00000001-0000-0000-0000-000000000002\", name: \"Steve\" }",
            "Steve",
        ));
        Some(true)
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "00000001-0000-0000-0000-000000000002".into(),
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

fn errors(src: &str) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".to_string(), src.to_string())]);
    errs.iter().map(|e| e.to_string()).collect()
}

fn join(lines: &str) -> Mock {
    let body: String = lines.lines().map(|l| format!("    {l}\n")).collect();
    let mut e = load(&format!("on join:\n{body}"));
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(
        h.logs.iter().all(|(l, _)| *l != LogLevel::Error),
        "runtime errors: {:?}",
        h.logs
    );
    h
}

const COW: &str = "00000001-0000-0000-0000-00000000000a";

#[test]
fn patterns_compile() {
    let n = w6::check_patterns().unwrap();
    assert!(n > 90, "only {n} patterns");
    assert_eq!(w6::pattern_sources().len(), n);
}

#[test]
fn entity_flags_use_data_merge() {
    let h = join(
        "spawn a goat\n\
         remove the left horn of last spawned entity\n\
         regrow both horns of last spawned entity\n\
         silence last spawned entity\n\
         make last spawned entity not silent\n\
         make last spawned entity scream\n\
         make last spawned entity stop screaming",
    );
    let want = [
        "{HasLeftHorn:0b}",
        "{HasLeftHorn:1b,HasRightHorn:1b}",
        "{Silent:1b}",
        "{Silent:0b}",
        "{IsScreamingGoat:1b}",
        "{IsScreamingGoat:0b}",
    ];
    let got: Vec<&str> = h
        .commands
        .iter()
        .filter_map(|c| c.strip_prefix(&format!("data merge entity {COW} ")))
        .collect();
    assert_eq!(got, want, "{:?} {:?} {:?}", h.commands, h.ops, h.logs);
}

#[test]
fn trees_and_scripts() {
    let h = join(
        "grow birch at location(1.5, 64, -2.5)\ngenerate a big tree above location(0, 70, 0)\n\
         reload script \"homes\"\ndisable script named \"a_b.carve\"\nenable skript file \"x\"",
    );
    assert_eq!(h.commands[0], "place feature minecraft:birch_checked 1 64 -3");
    assert_eq!(h.commands[1], "place feature minecraft:fancy_oak_checked 0 70 0");
    assert_eq!(
        &h.commands[2..],
        ["carve reload homes", "carve disable a_b.carve", "carve enable x"]
    );
    // A script name is a file name, never a second command.
    let mut e = load("on join:\n    reload script \"a b; stop\"");
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.commands.is_empty());
    assert!(h.logs.iter().any(|(l, _)| *l == LogLevel::Error));
}

#[test]
fn zombify_and_duplicate() {
    let h = join("spawn a villager\nzombify last spawned entity");
    assert_eq!(h.ops[1..], ["remove villager", "spawn zombie_villager 10 70 -3"]);
    assert_eq!(
        h.commands,
        [
            "data merge entity 00000002-0000-0000-0000-00000000000a {CustomName:\"Bob\",CustomNameVisible:1b,Health:12.0f}"
        ]
    );
    let h = join("spawn an allay\nmake last spawned entity duplicate");
    assert!(h.ops.iter().filter(|o| o.starts_with("spawn allay")).count() == 2);
    assert_eq!(
        h.commands
            .iter()
            .filter(|c| c.contains("DuplicationCooldown:6000L"))
            .count(),
        2
    );
}

#[test]
fn reset_and_remove_all() {
    let h = join(
        "set {n} to 5\nset {t} to \"hi\"\nset {b} to true\nset {l::*} to 1, 2, 1 and 3\n\
         reset {n}\nreset {t}\nreset {b}\nremove all 1 from {l::*}\n\
         send \"%{n}%|%{t}%|%{b}%|%{l::*}%\" to player\nreset {l::*}\nsend \"%size of {l::*}%\" to player",
    );
    assert_eq!(h.msgs, ["Steve: 0||false|2 and 3", "Steve: 0"]);
    assert!(errors("on join:\n    remove all 1 from {x}")[0].contains("list variable"));
}

#[test]
fn vector_rotation() {
    let r = |v, a, d| w6::rotate(v, &a, d);
    let close = |a: (f64, f64, f64), b: (f64, f64, f64)| {
        assert!(
            (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9 && (a.2 - b.2).abs() < 1e-9,
            "{a:?} != {b:?}"
        );
    };
    close(r((0.0, 1.0, 0.0), Axis2::X, 90.0), (0.0, 0.0, 1.0));
    close(r((1.0, 0.0, 0.0), Axis2::Y, 90.0), (0.0, 0.0, -1.0));
    close(r((1.0, 0.0, 0.0), Axis2::Z, 90.0), (0.0, 1.0, 0.0));
    // Around the y axis as a vector: the same as the plain axis.
    close(
        r((1.0, 2.0, 3.0), Axis2::Vec(0.0, 5.0, 0.0), 90.0),
        (3.0, 2.0, -1.0),
    );
    close(
        r((1.0, 2.0, 3.0), Axis2::Vec(0.0, 0.0, 0.0), 90.0),
        (1.0, 2.0, 3.0),
    );
    let h = join(
        "set {v} to vector(1, 0, 0)\nrotate {v} around the z-axis by 90\nsend \"%{v}%\" to player\nrotate {v} around vector(0, 1, 0) by 180\nsend \"%x of {v}%\" to player",
    );
    assert_eq!(h.msgs[0], "Steve: x: 0, y: 1, z: 0");
    assert!(
        h.msgs[1] == "Steve: 0" || h.msgs[1] == "Steve: -0",
        "{:?}",
        h.msgs
    );
}

#[test]
fn host_effects() {
    let h = join(
        "allow flight for player\ndisallow flight for player\n\
         make player see location(1, 2, 3) as stone\nmake player see location(1, 2, 3) as original block\n\
         stop sound \"entity.cow.moo\" in master for player\nstop all sounds in music for player\n\
         spawn a sheep\nshear last spawned entity\nunshear last spawned entity\ntame last spawned entity",
    );
    let ops = h.ops.join("\n");
    assert!(
        ops.contains("AllowFlight(Steve, true)") && ops.contains("AllowFlight(Steve, false)"),
        "{ops}"
    );
    assert!(
        ops.contains("FakeBlock(Steve, \"world\", 1, 2, 3, Some(\"stone\"))"),
        "{ops}"
    );
    assert!(
        ops.contains("FakeBlock(Steve, \"world\", 1, 2, 3, None)"),
        "{ops}"
    );
    assert!(
        ops.contains("StopSound(Steve, Some(\"entity.cow.moo\"), \"master\")"),
        "{ops}"
    );
    assert!(ops.contains("StopSound(Steve, None, \"music\")"), "{ops}");
    assert!(ops.contains("Tame("), "{ops}");
    let merges: Vec<&String> = h.commands.iter().filter(|c| c.contains("Sheared")).collect();
    assert_eq!(merges.len(), 2, "{:?}", h.commands);
    assert!(merges[0].ends_with("{Sheared:1b}") && merges[1].ends_with("{Sheared:0b}"));
}

#[test]
fn ride_swing_equip_any_entity() {
    let h = join(
        "spawn a pig\nset {p} to last spawned entity\nspawn a zombie\nset {z} to last spawned entity\n\
         make {z} ride {p}\nmake {z} dismount\nmake {z} leave its vehicle\nmake {z} swing main hand\n\
         make {z} swing off hand\nmake player swing\n\
         equip {z} with diamond helmet and iron boots\nequip player with shield\nmake {z} wear stick",
    );
    let cmds = &h.commands;
    assert!(
        cmds[0].starts_with("ride 00000002-") && cmds[0].contains(" mount 00000001-"),
        "{cmds:?}"
    );
    assert!(
        cmds[1].ends_with("dismount") && cmds[2].ends_with("dismount"),
        "{cmds:?}"
    );
    assert!(
        cmds[3].ends_with("mainhand") && cmds[4].ends_with("offhand"),
        "{cmds:?}"
    );
    assert!(cmds.contains(&"swing Steve mainhand".to_string()), "{cmds:?}");
    let eq: Vec<&String> = cmds.iter().filter(|c| c.starts_with("item replace")).collect();
    assert_eq!(eq.len(), 4, "{eq:?}");
    assert!(eq[0].ends_with("armor.head with minecraft:diamond_helmet"));
    assert!(eq[1].ends_with("armor.feet with minecraft:iron_boots"));
    assert_eq!(
        eq[2],
        "item replace entity Steve weapon.offhand with minecraft:shield"
    );
}
