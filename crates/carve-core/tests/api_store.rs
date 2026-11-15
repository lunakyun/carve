//! The one store (what changed, old files) and the checks for actions that
//! used to be console commands (sounds, warnings with the script line).

use carve_core::compile::EventKind;
use carve_core::persist::{Changes, read_legacy};
use carve_core::skbee_bound::{Bound, Bounds};
use carve_core::skbee_scoreboard_teams::{SbEdit, TeamProp};
use carve_core::sounds::{SoundKind, clamp_levels, classify, namespaced};
use carve_core::*;

#[derive(Default)]
struct Mock {
    logs: Vec<(LogLevel, String)>,
    cmds: Vec<String>,
    failures: Vec<String>,
    /// `true`: keep failures for the engine (like the plugin); `false`: use the default (log at once).
    collect: bool,
    now: u64,
}

impl Host for Mock {
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
        self.now
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
    fn fail(&mut self, msg: &str) {
        if self.collect {
            self.failures.push(msg.to_string());
        } else {
            self.log(LogLevel::Warn, msg);
        }
    }
    fn take_failures(&mut self) -> Vec<String> {
        std::mem::take(&mut self.failures)
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn engine(src: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("test.carve".into(), src.into())]);
    assert!(errs.is_empty(), "{errs:?}");
    e
}

fn warnings(h: &Mock) -> Vec<&str> {
    h.logs
        .iter()
        .filter(|(l, _)| *l == LogLevel::Warn)
        .map(|(_, m)| m.as_str())
        .collect()
}

// ---- warnings with the script line ----

#[test]
fn a_failed_call_is_logged_with_the_script_line() {
    let mut e = engine("on join:\n    send \"hi\"\n    kick \"Ghost\"\n");
    let mut h = Mock {
        collect: true,
        ..Mock::default()
    };
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(warnings(&h), ["test.carve:3: kick: Ghost is not online"]);
    assert!(h.cmds.is_empty(), "no console command");
}

#[test]
fn a_host_that_does_not_collect_still_logs_the_failure() {
    let mut e = engine("on join:\n    kick \"Ghost\"\n");
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(warnings(&h), ["kick: Ghost is not online"]);
}

#[test]
fn every_failure_of_one_line_is_logged() {
    let mut e = engine("on join:\n    kick \"A\"\n    kick \"B\"\n");
    let mut h = Mock {
        collect: true,
        ..Mock::default()
    };
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(
        warnings(&h),
        [
            "test.carve:2: kick: A is not online",
            "test.carve:3: kick: B is not online"
        ]
    );
}

// ---- sounds ----

#[test]
fn sound_ids_are_sorted_so_the_lookup_works() {
    let text = include_str!("../src/sound_ids.txt");
    let ids: Vec<&str> = text.lines().collect();
    assert!(ids.len() > 1500, "{} sounds", ids.len());
    assert!(ids.windows(2).all(|w| w[0] < w[1]), "sorted, no repeats");
}

#[test]
fn classify_sounds() {
    assert_eq!(classify("entity.experience_orb.pickup"), SoundKind::Vanilla);
    assert_eq!(classify("minecraft:block.note_block.bass"), SoundKind::Vanilla);
    assert_eq!(classify("mypack:custom.boom"), SoundKind::Custom);
    assert_eq!(classify("entity.not_a_sound"), SoundKind::Unknown);
    assert_eq!(classify("minecraft:nope"), SoundKind::Unknown);
}

#[test]
fn namespace_and_levels() {
    assert_eq!(namespaced("block.stone.hit"), "minecraft:block.stone.hit");
    assert_eq!(namespaced("p:x"), "p:x");
    assert_eq!(clamp_levels(-3.0, 9.0), (0.0, 2.0));
    assert_eq!(clamp_levels(2.5, 0.5), (2.5, 0.5));
    assert_eq!(clamp_levels(f64::NAN, f64::INFINITY), (0.0, 0.0));
}

// ---- scoreboard edits keep their old commands ----

#[test]
fn scoreboard_edits_make_the_vanilla_commands() {
    let t = |e: SbEdit| e.command();
    assert_eq!(
        t(SbEdit::AddObjective {
            name: "kills".into(),
            criteria: "dummy".into(),
            display: Some("Kills".into())
        }),
        "scoreboard objectives add kills dummy {\"text\":\"Kills\"}"
    );
    assert_eq!(
        t(SbEdit::RemoveObjective("kills".into())),
        "scoreboard objectives remove kills"
    );
    assert_eq!(
        t(SbEdit::SetScore {
            holder: "Steve".into(),
            objective: "kills".into(),
            value: 3
        }),
        "scoreboard players set Steve kills 3"
    );
    assert_eq!(
        t(SbEdit::ResetScore {
            holder: "Steve".into(),
            objective: None
        }),
        "scoreboard players reset Steve"
    );
    assert_eq!(
        t(SbEdit::Join {
            team: "red".into(),
            holder: "Steve".into()
        }),
        "team join red Steve"
    );
    assert_eq!(
        t(SbEdit::Leave {
            holder: "Steve".into()
        }),
        "team leave Steve"
    );
    assert_eq!(
        t(SbEdit::TeamModify {
            team: "red".into(),
            prop: TeamProp::FriendlyFire,
            arg: "false".into()
        }),
        "team modify red friendlyFire false"
    );
    assert_eq!(
        t(SbEdit::TeamModify {
            team: "red".into(),
            prop: TeamProp::Color,
            arg: "reset".into()
        }),
        "team modify red color reset"
    );
}

#[test]
fn team_scripts_reach_the_host_as_scoreboard_edits() {
    let mut e = engine(
        "on join:\n    register team \"red\"\n    add player to team \"red\"\n    unregister team \"red\"\n",
    );
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.cmds, ["team add red", "team join red Steve", "team remove red"]);
}

// ---- variables: only changed keys ----

fn names(c: &[(String, Option<Vec<u8>>)]) -> Vec<&str> {
    c.iter().map(|(k, _)| k.as_str()).collect()
}

#[test]
fn only_changed_variables_are_written() {
    let mut v = Variables::default();
    v.set("a".into(), Value::Num(1.0));
    v.set("b".into(), Value::Text("x".into()));
    let c = v.take_changes();
    assert_eq!(names(&c), ["a", "b"]);
    assert_eq!(c[0].1.as_deref(), Some(&b"1.0"[..]));
    assert!(v.take_changes().is_empty(), "nothing changed since");
    v.set("b".into(), Value::Text("y".into()));
    v.set_str("b", Value::Text("z".into()));
    let c = v.take_changes();
    assert_eq!(names(&c), ["b"], "one change per key, however often it was set");
    v.set("a".into(), Value::None);
    v.delete("never-set");
    let c = v.take_changes();
    assert_eq!(c, vec![("a".to_string(), None)], "deleting writes a delete");
}

#[test]
fn list_changes_are_per_item() {
    let mut v = Variables::default();
    v.list_push("l::", Value::Num(1.0));
    v.list_push("l::", Value::Num(2.0));
    v.list_push("l::", Value::Num(3.0));
    assert_eq!(names(&v.take_changes()), ["l::1", "l::2", "l::3"]);
    v.list_remove("l::", &Value::Num(2.0));
    assert_eq!(v.take_changes(), vec![("l::2".to_string(), None)]);
    v.delete("l::*");
    let c = v.take_changes();
    assert_eq!(names(&c), ["l::1", "l::3"]);
    assert!(c.iter().all(|(_, b)| b.is_none()));
    assert_eq!(v.list_len("l::"), 0);
}

#[test]
fn a_value_that_cannot_be_saved_is_deleted_from_the_store() {
    let mut v = Variables::default();
    v.set("e".into(), Value::Num(1.0));
    v.take_changes();
    v.set(
        "e".into(),
        Value::Entity(EntityRef {
            uuid: "x".into(),
            kind: "pig".into(),
        }),
    );
    assert_eq!(v.take_changes(), vec![("e".to_string(), None)]);
    v.set("nan".into(), Value::Num(f64::NAN));
    assert_eq!(v.take_changes(), vec![("nan".to_string(), None)]);
}

#[test]
fn variables_round_trip_through_entries() {
    let mut v = Variables::default();
    v.set("n".into(), Value::Num(4.5));
    v.set("t".into(), Value::Text("hi \"q\"".into()));
    v.set("p".into(), Value::Player(steve()));
    v.set("flag".into(), Value::Bool(true));
    v.set(
        "loc".into(),
        Value::Location {
            x: 1.5,
            y: 64.0,
            z: -2.0,
            world: Some("world".into()),
        },
    );
    v.set(
        "i".into(),
        Value::Item {
            id: "diamond_sword".into(),
            count: 2,
            meta: Some(Box::new(carve_core::item_meta::ItemMeta {
                name: Some("&6Blade".into()),
                enchants: vec![("sharpness".into(), 5)],
                ..Default::default()
            })),
        },
    );
    v.list_push("list::", Value::Num(7.0));
    v.list_push("list::", Value::Text("two".into()));
    let entries = v.entries();
    assert_eq!(entries.len(), 8);
    let (back, bad) = Variables::from_entries(entries);
    assert!(bad.is_empty());
    for k in ["n", "t", "p", "flag", "loc", "i", "list::1", "list::2"] {
        assert_eq!(back.get(k), v.get(k), "{k}");
    }
    assert_eq!(back.list_len("list::"), 2);
    let mut back = back;
    assert!(back.take_changes().is_empty(), "loading marks nothing as changed");
    back.list_push("list::", Value::Num(9.0));
    assert_eq!(
        names(&back.take_changes()),
        ["list::3"],
        "the list goes on where it left off"
    );
}

#[test]
fn unreadable_entries_are_reported_and_skipped() {
    let (v, bad) = Variables::from_entries(vec![
        ("ok".to_string(), b"3".to_vec()),
        ("broken".to_string(), b"{nope".to_vec()),
        ("null".to_string(), b"null".to_vec()),
    ]);
    assert_eq!(v.get("ok"), Value::Num(3.0));
    assert_eq!(bad, ["broken", "null"]);
}

#[test]
fn entries_equal_the_old_json() {
    let mut v = Variables::default();
    v.set("a".into(), Value::Num(1.0));
    v.set("b".into(), Value::Text("x".into()));
    let from_json = Variables::from_json(&v.to_json()).unwrap();
    let (from_entries, _) = Variables::from_entries(v.entries());
    assert_eq!(from_json.entries(), from_entries.entries());
}

// ---- engine: when to write ----

#[test]
fn the_engine_writes_after_the_debounce_and_only_what_changed() {
    let mut e = engine("on join:\n    set {n} to 4.5\n    set {list::a} to 1\n");
    let mut h = Mock {
        now: 1000,
        ..Mock::default()
    };
    assert!(e.persist_due(1000).is_none());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(e.persist_due(5999).is_none(), "debounce not over");
    let c = e.persist_due(6000).expect("write after 5s");
    assert_eq!(names(&c.vars), ["list::a", "n"]);
    assert!(c.bounds.is_empty());
    assert!(e.persist_due(30_000).is_none(), "nothing new");
    // Only the changed key goes the second time.
    let mut e2 = engine("on join:\n    set {n} to {n} + 1\n");
    e2.vars = {
        let mut v = Variables::default();
        v.set("n".into(), Value::Num(1.0));
        v.set("other".into(), Value::Num(9.0));
        v.take_changes();
        v
    };
    e2.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let c = e2.persist_now().expect("unload writes");
    assert_eq!(names(&c.vars), ["n"]);
    assert!(e2.persist_now().is_none());
}

#[test]
fn bounds_write_only_what_changed_and_keep_their_order() {
    let mk = |id: &str, x: i32| Bound {
        id: id.into(),
        world: "world".into(),
        min: [x, 0, 0],
        max: [x + 9, 9, 9],
        owners: vec![],
        members: vec![],
        flags: Default::default(),
        temporary: false,
    };
    let mut b = Bounds::default();
    b.create(mk("Spawn", 0));
    b.create(mk("Arena", 100));
    b.create(mk("Mine", 200));
    let first = b.take_changes();
    assert_eq!(names(&first), ["arena", "mine", "spawn"]);
    assert!(b.take_changes().is_empty());
    b.create(mk("Arena", 150));
    assert_eq!(names(&b.take_changes()), ["arena"], "only the edited one");
    assert!(b.delete("Spawn"));
    let c = b.take_changes();
    // Spawn is gone; Arena and Mine moved up one place.
    assert_eq!(names(&c), ["arena", "mine", "spawn"]);
    assert!(c.iter().find(|(k, _)| k == "spawn").unwrap().1.is_none());

    let (back, bad) = Bounds::from_entries(b.entries());
    assert!(bad.is_empty());
    let ids: Vec<&str> = back.all().iter().map(|x| x.id.as_str()).collect();
    assert_eq!(ids, ["Arena", "Mine"], "creation order survives the store");
    assert_eq!(back.get("arena").unwrap().min, [150, 0, 0]);
}

#[test]
fn temporary_bounds_are_not_written() {
    let mut b = Bounds::default();
    b.create(Bound {
        id: "tmp".into(),
        world: "world".into(),
        min: [0; 3],
        max: [1; 3],
        owners: vec![],
        members: vec![],
        flags: Default::default(),
        temporary: true,
    });
    assert!(b.take_changes().is_empty());
}

#[test]
fn changes_report_emptiness() {
    assert!(Changes::default().is_empty());
}

// ---- the old files ----

#[test]
fn old_files_become_store_rows() {
    let mut v = Variables::default();
    v.set("score".into(), Value::Num(7.0));
    v.list_push("l::", Value::Text("a".into()));
    let bounds = r#"{"bounds":[
        {"id":"Spawn","world":"world","min":[0,0,0],"max":[9,9,9],"owners":[["u1","Steve"]],"members":[],"flags":{"pvp":false}},
        {"id":"Mine","world":"world","min":[10,0,0],"max":[19,9,9]}]}"#;
    let l = read_legacy(Some(&v.to_json()), Some(bounds), Some("uuid-a\n\n  uuid-b  \n"));
    assert!(l.broken.is_empty());
    assert_eq!(l.vars.len(), 2);
    assert_eq!(l.bounds.len(), 2);
    assert_eq!(l.seen, ["uuid-a", "uuid-b"]);
    let (back, bad) = Bounds::from_entries(l.bounds);
    assert!(bad.is_empty());
    let ids: Vec<&str> = back.all().iter().map(|x| x.id.as_str()).collect();
    assert_eq!(ids, ["Spawn", "Mine"], "file order is kept");
    assert_eq!(back.get("spawn").unwrap().owners, vec![steve()]);
}

#[test]
fn a_broken_old_file_is_named_and_the_others_still_load() {
    let l = read_legacy(Some("{ nope"), Some("{\"bounds\":[]}"), Some("uuid-a"));
    assert_eq!(l.broken.len(), 1);
    assert_eq!(l.broken[0].0, "variables.json");
    assert!(l.vars.is_empty());
    assert_eq!(l.seen, ["uuid-a"]);
    let none = read_legacy(None, None, None);
    assert!(none.is_empty() && none.broken.is_empty());
}
