use std::collections::HashMap;

use carve_core::compile::{EventKind, compile_file};
use carve_core::world_effects::FxPos;
use carve_core::*;

struct Ent {
    uuid: String,
    kind: String,
    pos: (f64, f64, f64),
    name: Option<String>,
}

/// Records calls and keeps a tiny world of entities.
#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    msgs: Vec<String>,
    online: HashMap<String, String>,
    ents: Vec<Ent>,
    vel: HashMap<String, (f64, f64, f64)>,
    hp: HashMap<String, f64>,
    logs: Vec<(LogLevel, String)>,
    next: u32,
}

impl Mock {
    fn steve() -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m
    }

    fn with(mut self, kind: &str, pos: (f64, f64, f64)) -> Self {
        self.next += 1;
        self.ents.push(Ent {
            uuid: format!("e{}", self.next),
            kind: kind.into(),
            pos,
            name: None,
        });
        self
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
    fn kill(&mut self, p: &PlayerRef) {
        self.calls.push(format!("kill player {}", p.name));
    }
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
    fn online_players(&mut self) -> Vec<PlayerRef> {
        self.online
            .iter()
            .map(|(u, n)| PlayerRef {
                uuid: u.clone(),
                name: n.clone(),
            })
            .collect()
    }
    fn player_location(&mut self, p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        self.online
            .contains_key(&p.uuid)
            .then(|| (0.0, 64.0, 0.0, "world".to_string()))
    }
    fn spawn_entity(&mut self, kind: &str, at: &FxPos) -> Option<EntityRef> {
        self.next += 1;
        let uuid = format!("e{}", self.next);
        self.calls
            .push(format!("spawn {kind} {} {} {}", at.x, at.y, at.z));
        self.ents.push(Ent {
            uuid: uuid.clone(),
            kind: kind.into(),
            pos: (at.x, at.y, at.z),
            name: None,
        });
        Some(EntityRef {
            uuid,
            kind: kind.into(),
        })
    }
    fn entity_kill(&mut self, e: &EntityRef) {
        self.calls.push(format!("kill {}", e.uuid));
        self.ents.retain(|x| x.uuid != e.uuid);
    }
    fn entity_remove(&mut self, e: &EntityRef) {
        self.calls.push(format!("remove {}", e.uuid));
        self.ents.retain(|x| x.uuid != e.uuid);
    }
    fn entity_location(&mut self, e: &EntityRef) -> Option<(f64, f64, f64, String)> {
        self.ents
            .iter()
            .find(|x| x.uuid == e.uuid)
            .map(|x| (x.pos.0, x.pos.1, x.pos.2, "world".to_string()))
    }
    fn entity_name(&mut self, e: &EntityRef) -> Option<String> {
        self.ents.iter().find(|x| x.uuid == e.uuid)?.name.clone()
    }
    fn set_entity_name(&mut self, e: &EntityRef, name: Option<&str>) {
        if let Some(x) = self.ents.iter_mut().find(|x| x.uuid == e.uuid) {
            x.name = name.map(str::to_string);
        }
    }
    fn entities_near(&mut self, at: &FxPos, radius: f64, kind: Option<&str>) -> Vec<EntityRef> {
        self.ents
            .iter()
            .filter(|x| kind.is_none_or(|k| k == x.kind))
            .filter(|x| {
                let (dx, dy, dz) = (x.pos.0 - at.x, x.pos.1 - at.y, x.pos.2 - at.z);
                (dx * dx + dy * dy + dz * dz).sqrt() <= radius
            })
            .map(|x| EntityRef {
                uuid: x.uuid.clone(),
                kind: x.kind.clone(),
            })
            .collect()
    }
    fn entity_velocity(&mut self, e: &EntityRef) -> Option<(f64, f64, f64)> {
        Some(self.vel.get(&e.uuid).copied().unwrap_or((0.0, 0.0, 0.0)))
    }
    fn set_entity_velocity(&mut self, e: &EntityRef, v: (f64, f64, f64)) {
        self.calls.push(format!("vel {} {} {} {}", e.uuid, v.0, v.1, v.2));
        self.vel.insert(e.uuid.clone(), v);
    }
    fn entity_teleport(&mut self, e: &EntityRef, x: f64, y: f64, z: f64, _w: Option<&str>) {
        self.calls.push(format!("tp {} {x} {y} {z}", e.uuid));
    }
    fn entity_health(&mut self, e: &EntityRef) -> Option<f64> {
        Some(*self.hp.get(&e.uuid).unwrap_or(&10.0))
    }
    fn set_entity_health(&mut self, e: &EntityRef, h: f64) {
        self.calls.push(format!("hp {} {h}", e.uuid));
        self.hp.insert(e.uuid.clone(), h);
    }
}

/// Implements only the required methods, so the trait defaults run.
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
        Some((1.0, 65.0, 2.0, "world".into()))
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn errors_of(body: &str) -> Vec<ScriptError> {
    compile_file("bad.carve", &script(body)).err().unwrap_or_default()
}

fn script(body: &str) -> String {
    let lines: String = body.lines().map(|l| format!("    {l}\n")).collect();
    format!("on join:\n{lines}")
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

fn run_on(h: &mut Mock, body: &str) {
    let mut e = load(body);
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errors.is_empty(), "runtime errors: {errors:?}");
}

fn run(body: &str) -> Mock {
    let mut h = Mock::steve();
    run_on(&mut h, body);
    h
}

// ---------- spawning ----------

#[test]
fn spawn_at_player() {
    let h = run("spawn a zombie at player");
    assert_eq!(h.calls, ["spawn zombie 0 64 0"]);
}

#[test]
fn spawn_count_and_default_place() {
    let h = run("spawn 3 pigs");
    assert_eq!(h.calls, ["spawn pig 0 64 0"; 3]);
}

#[test]
fn spawn_at_location() {
    let h = run("spawn 2 iron golems at location(1, 70, 3)");
    assert_eq!(h.calls, ["spawn iron_golem 1 70 3"; 2]);
}

#[test]
fn spawn_count_from_variable_is_clamped() {
    let h = run("set {_n} to 5000\nspawn {_n} chickens");
    assert_eq!(h.calls.len(), 100);
}

#[test]
fn spawn_plurals() {
    let h = run("spawn 1 wolves\nspawn 1 zombies\nspawn a creeper");
    assert_eq!(
        h.calls,
        ["spawn wolf 0 64 0", "spawn zombie 0 64 0", "spawn creeper 0 64 0"]
    );
}

#[test]
fn spawn_unknown_type_is_an_error() {
    let errs = errors_of("spawn a flurb at player");
    assert!(
        errs.iter()
            .any(|e| e.to_string().contains("unknown entity 'flurb'")),
        "{errs:?}"
    );
}

#[test]
fn spawn_player_is_an_error() {
    assert!(!errors_of("spawn a player at player").is_empty());
}

#[test]
fn spawn_without_place_needs_a_player() {
    let src = "on load:\n    spawn a pig\n";
    assert!(compile_file("bad.carve", src).is_err());
    let ok = "on load:\n    spawn a pig at location(0, 70, 0)\n";
    assert!(compile_file("ok.carve", ok).is_ok());
}

#[test]
fn spawn_still_leaves_lightning_and_particles_alone() {
    let h = run("spawn lightning at player\nspawn 3 flame particles at player");
    assert!(
        h.calls.iter().any(|c| c.contains("lightning_bolt")),
        "{:?}",
        h.calls
    );
    assert!(h.calls.iter().any(|c| c.contains("particle")), "{:?}", h.calls);
}

#[test]
fn default_spawn_uses_summon() {
    let mut e = load("spawn a cow at player");
    let mut h = Bare::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.cmds, ["summon minecraft:cow 1 65 2"]);
}

#[test]
fn last_spawned_entity() {
    let h = run("spawn a zombie at player\nset {_z} to last spawned entity\nsend \"%type of {_z}%\"");
    assert_eq!(h.msgs, ["zombie"]);
}

#[test]
fn last_spawned_is_none_before_any_spawn() {
    let h = run("send \"%last spawned entity%\"");
    assert_eq!(h.msgs, ["<none>"]);
}

#[test]
fn last_spawned_condition_and_name() {
    let h = run(
        "spawn a pig at player\nset name of last spawned entity to \"Bob\"\nsend \"%name of last spawned entity%\"\nif last spawned entity is a pig:\n    send \"pig\"\nif last spawned entity is not a cow:\n    send \"not cow\"",
    );
    assert_eq!(h.msgs, ["Bob", "pig", "not cow"]);
}

#[test]
fn entity_location_of_last_spawned() {
    let h = run(
        "spawn a pig at location(4, 70, 5)\nsend \"%y of last spawned entity%\"\nsend \"%location of last spawned entity%\"",
    );
    assert_eq!(h.msgs, ["70", "x: 4, y: 70, z: 5 in 'world'"]);
}

// ---------- kill and remove ----------

#[test]
fn kill_last_spawned() {
    let h = run("spawn a zombie at player\nkill last spawned entity");
    assert_eq!(h.calls, ["spawn zombie 0 64 0", "kill e1"]);
}

#[test]
fn kill_entity_in_a_variable() {
    let h = run("spawn a zombie at player\nset {_e} to last spawned entity\nkill {_e}");
    assert!(h.calls.contains(&"kill e1".to_string()), "{:?}", h.calls);
}

#[test]
fn kill_player_still_works() {
    let h = run("kill player");
    assert_eq!(h.calls, ["kill player Steve"]);
}

#[test]
fn kill_all_zombies_in_radius() {
    let mut h = Mock::steve()
        .with("zombie", (1.0, 64.0, 0.0))
        .with("zombie", (50.0, 64.0, 0.0))
        .with("pig", (2.0, 64.0, 0.0));
    run_on(&mut h, "kill all zombies in radius 10 of player");
    assert_eq!(h.calls, ["kill e1"]);
}

#[test]
fn remove_entity_does_not_kill() {
    let h = run("spawn a pig at player\ndelete last spawned entity\ndespawn last spawned entity");
    assert!(
        h.calls.iter().filter(|c| c.starts_with("remove")).count() == 2,
        "{:?}",
        h.calls
    );
}

#[test]
fn delete_variable_is_untouched() {
    let h = run("set {_x} to 1\ndelete {_x}\nsend \"%{_x}%\"");
    assert_eq!(h.msgs, ["<none>"]);
}

// ---------- finding ----------

#[test]
fn nearest_entity() {
    let mut h = Mock::steve()
        .with("zombie", (9.0, 64.0, 0.0))
        .with("pig", (3.0, 64.0, 0.0))
        .with("cow", (20.0, 64.0, 0.0));
    run_on(&mut h, "send \"%type of nearest entity to player%\"");
    assert_eq!(h.msgs, ["pig"]);
}

#[test]
fn nearest_of_a_type_and_in_radius() {
    let mut h = Mock::steve()
        .with("zombie", (9.0, 64.0, 0.0))
        .with("pig", (3.0, 64.0, 0.0))
        .with("zombie", (5.0, 64.0, 0.0));
    run_on(
        &mut h,
        "set {_z} to nearest zombie to player\nsend \"%x of {_z}%\"\nset {_far} to nearest zombie to player within 2 blocks\nsend \"%{_far}%\"",
    );
    assert_eq!(h.msgs, ["5", "<none>"]);
}

#[test]
fn nearest_skips_the_subject_itself() {
    let mut h = Mock::steve()
        .with("pig", (3.0, 64.0, 0.0))
        .with("pig", (8.0, 64.0, 0.0));
    run_on(
        &mut h,
        "spawn a pig at location(0, 64, 1)\nset {_n} to nearest pig to last spawned entity\nsend \"%x of {_n}%\"",
    );
    assert_eq!(h.msgs, ["3"]);
}

#[test]
fn nearest_player() {
    let h = run("send \"%nearest player to location(0, 64, 5)%\"");
    assert_eq!(h.msgs, ["Steve"]);
}

#[test]
fn loop_entities_in_radius() {
    let mut h = Mock::steve()
        .with("zombie", (1.0, 64.0, 0.0))
        .with("pig", (2.0, 64.0, 0.0))
        .with("zombie", (90.0, 64.0, 0.0));
    run_on(
        &mut h,
        "loop all entities in radius 10 of player:\n    send \"%type of loop-entity%\"",
    );
    let mut got = h.msgs.clone();
    got.sort();
    assert_eq!(got, ["pig", "player", "zombie"]);
}

#[test]
fn loop_players_in_radius() {
    let mut h = Mock::steve();
    run_on(
        &mut h,
        "loop all players in radius 10 of player:\n    send \"hi %loop-value%\"",
    );
    assert_eq!(h.msgs, ["hi Steve"]);
    let mut h = Mock::steve();
    run_on(
        &mut h,
        "loop all players in radius 10 of location(100, 64, 0):\n    send \"hi\"",
    );
    assert!(h.msgs.is_empty());
}

#[test]
fn loop_all_players_plain_still_works() {
    let h = run("loop all players:\n    send \"p %loop-player%\"");
    assert_eq!(h.msgs, ["p Steve"]);
}

#[test]
fn size_of_entities() {
    let mut h = Mock::steve()
        .with("pig", (1.0, 64.0, 0.0))
        .with("pig", (2.0, 64.0, 0.0));
    run_on(&mut h, "send \"%size of all pigs in radius 5 of player%\"");
    assert_eq!(h.msgs, ["2"]);
}

// ---------- conditions ----------

#[test]
fn entity_is_a_type() {
    let mut h = Mock::steve().with("zombie", (1.0, 64.0, 0.0));
    run_on(
        &mut h,
        "set {_e} to nearest entity to player\nif {_e} is a zombie:\n    send \"z\"\nif {_e} isn't a pig:\n    send \"not pig\"\nif {_e} is not an enderman:\n    send \"not enderman\"\nif {_e} is a pig:\n    send \"bad\"",
    );
    assert_eq!(h.msgs, ["z", "not pig", "not enderman"]);
}

#[test]
fn player_is_a_player() {
    let h = run("if player is a player:\n    send \"yes\"\nif player is a zombie:\n    send \"no\"");
    assert_eq!(h.msgs, ["yes"]);
}

#[test]
fn other_is_conditions_still_work() {
    let h = run("set {_x} to 3\nif {_x} is 3:\n    send \"three\"\nif {_x} is not set:\n    send \"bad\"");
    assert_eq!(h.msgs, ["three"]);
}

// ---------- velocity ----------

#[test]
fn push_player_upwards_with_speed() {
    let h = run("push player upwards with speed 2");
    assert_eq!(h.calls, ["vel u1 0 2 0"]);
}

#[test]
fn push_adds_to_the_current_velocity() {
    let h = run("push player up\npush player north at speed 3");
    assert_eq!(h.calls, ["vel u1 0 1 0", "vel u1 0 1 -3"]);
}

#[test]
fn push_an_entity() {
    let h = run("spawn a pig at player\npush last spawned entity east with speed 0.5");
    assert!(h.calls.contains(&"vel e1 0.5 0 0".to_string()), "{:?}", h.calls);
}

#[test]
fn push_speed_is_clamped() {
    let h = run("push player up with speed 1000");
    assert_eq!(h.calls, ["vel u1 0 20 0"]);
}

#[test]
fn push_needs_a_valid_tail() {
    assert!(!errors_of("push player up fast").is_empty());
}

#[test]
fn set_and_read_velocity() {
    let h = run("set y velocity of player to 1.5\nsend \"%y velocity of player%\"");
    assert_eq!(h.calls, ["vel u1 0 1.5 0"]);
    assert_eq!(h.msgs, ["1.5"]);
}

// ---------- misc ----------

#[test]
fn teleport_an_entity() {
    let h = run("spawn a pig at player\nteleport last spawned entity to location(5, 70, 5)");
    assert!(h.calls.contains(&"tp e1 5 70 5".to_string()), "{:?}", h.calls);
}

#[test]
fn entity_in_a_variable_despawn_name_teleport() {
    let h = run(concat!(
        "spawn a pig at player\nset {_e} to last spawned entity\n",
        "set name of {_e} to \"Bob\"\nsend \"%name of {_e}%\"\n",
        "teleport {_e} to location(5, 70, 5)\ndespawn {_e}"
    ));
    assert_eq!(h.msgs, ["Bob"]);
    assert!(h.calls.contains(&"tp e1 5 70 5".to_string()), "{:?}", h.calls);
    assert!(h.calls.contains(&"remove e1".to_string()), "{:?}", h.calls);
}

#[test]
fn entity_health_set_add_remove() {
    let h = run(concat!(
        "spawn a pig at player\nset {_e} to last spawned entity\n",
        "set health of {_e} to 8\nremove 3 from health of {_e}\nadd 1 to health of {_e}\n",
        "send \"%health of {_e}%\"\nset health of {_e} to 0"
    ));
    assert_eq!(h.msgs, ["6"]);
    let tail: Vec<_> = h.calls.iter().skip(1).map(String::as_str).collect();
    assert_eq!(tail, ["hp e1 8", "hp e1 5", "hp e1 6", "kill e1"]);
}

#[test]
fn teleport_player_variable_to_xyz_still_works() {
    let h = run("set {_p} to player\nteleport {_p} to 1, 2, 3");
    assert!(h.logs.is_empty(), "{:?}", h.logs);
}

#[test]
fn teleport_player_still_works() {
    let h = run("teleport player to 1, 2, 3");
    assert!(h.logs.is_empty());
}

#[test]
fn entity_type_text_compares() {
    let h = run("spawn a pig at player\nif type of last spawned entity is \"pig\":\n    send \"ok\"");
    assert_eq!(h.msgs, ["ok"]);
}

#[test]
fn entities_are_not_saved_in_global_variables() {
    let mut e = load("spawn a pig at player\nset {g} to last spawned entity");
    let mut h = Mock::steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(!e.save_now().unwrap_or_default().contains("pig"));
}
