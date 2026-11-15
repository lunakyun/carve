//! Wave 6 chunk 1: new event triggers and the pure logic behind them.

use carve_core::compile::check_registry;
use carve_core::w2_events_3::Ev3;
use carve_core::w2_events_4::{self as ev4, ExtEvent};
use carve_core::w6_chunk1 as w6;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
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
    fn broadcast(&mut self, t: &str) {
        self.calls.push(format!("broadcast {t}"));
    }
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.calls.push(format!("cmd {c}"));
    }
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
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
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
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
    assert!(errs.is_empty(), "unexpected errors: {errs:?}");
    e
}

fn errors(src: &str) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    errs.iter().map(|e| e.to_string()).collect()
}

fn run4(src: &str, k: ExtEvent, p: Option<PlayerRef>, vals: Vec<Value>) -> (Vec<String>, FireOutcome) {
    let mut e = load(src);
    let mut h = Mock::default();
    let o = e.fire(EventKind::Ext4(k), ev4::ctx(p, vals), &mut h);
    let errs: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errs.is_empty(), "runtime errors: {errs:?}");
    (h.calls, o)
}

#[test]
fn registry_still_compiles() {
    check_registry().unwrap();
}

#[test]
fn chunk_math() {
    assert_eq!(w6::chunk_of(0.0), 0);
    assert_eq!(w6::chunk_of(15.99), 0);
    assert_eq!(w6::chunk_of(16.0), 1);
    assert_eq!(w6::chunk_of(-0.1), -1);
    assert_eq!(w6::chunk_of(-16.0), -1);
    assert_eq!(w6::chunk_of(-16.1), -2);
    assert!(!w6::chunk_changed((1.0, 1.0), (15.9, 15.9)));
    assert!(w6::chunk_changed((15.9, 1.0), (16.1, 1.0)));
    assert!(w6::chunk_changed((1.0, 0.5), (1.0, -0.5)));
}

#[test]
fn jump_edge() {
    assert!(w6::jump_started(0, 16));
    assert!(w6::jump_started(1, 17));
    assert!(!w6::jump_started(16, 17));
    assert!(!w6::jump_started(16, 0));
    assert!(!w6::jump_started(0, 1));
}

#[test]
fn plate_and_kinds() {
    assert!(w6::plate_pressed(0, 15));
    assert!(!w6::plate_pressed(15, 15));
    assert!(!w6::plate_pressed(15, 0));
    assert!(w6::is_pressure_plate("minecraft:stone_pressure_plate"));
    assert!(w6::is_pressure_plate("oak_pressure_plate"));
    assert!(!w6::is_pressure_plate("minecraft:stone"));
    assert!(w6::is_zombie_kind("minecraft:husk"));
    assert!(!w6::is_zombie_kind("creeper"));
}

#[test]
fn step_on_header() {
    assert_eq!(
        w6::split_step_on("step on stone"),
        ("step on", Some(("stepon", "stone".into())))
    );
    assert_eq!(w6::split_step_on("step on"), ("step on", None));
    assert_eq!(w6::split_step_on("move"), ("move", None));
    assert!(w6::step_rhs("Oak Planks").is_some());
    assert!(w6::step_rhs("a minecraft:stone").is_some());
    assert!(w6::step_rhs("").is_none());
    assert!(w6::step_rhs("a-b!").is_none());
}

#[test]
fn new_event_names_compile() {
    for name in [
        "player pickup arrow",
        "riptide",
        "attempt attack",
        "experience spawn",
        "projectile collide",
        "pressure plate",
        "trip",
        "player chunk enter",
        "player chunk enters",
        "jump",
        "world border bounds change",
        "world border bounds finish change",
        "world border center change",
        "armor change",
        "spectate",
        "script unload",
        "unload",
        "server stop",
        "step on stone",
        "walk on oak planks",
        "tool change",
    ] {
        let e = errors(&format!("on {name}:\n    send \"x\" to console\n"));
        assert!(e.is_empty(), "{name}: {e:?}");
    }
}

#[test]
fn step_on_filters_the_block() {
    let src = "on step on stone:\n    send \"stone\" to player\n";
    let vals = |b: &str| vec![Value::Text(b.into()), Value::None];
    let (calls, _) = run4(src, ExtEvent::StepOn, Some(steve()), vals("stone"));
    assert_eq!(calls, ["send Steve stone"]);
    let (calls, _) = run4(src, ExtEvent::StepOn, Some(steve()), vals("dirt"));
    assert!(calls.is_empty());
}

#[test]
fn chunk_values() {
    let src = "on player chunk enter:\n    send \"%event-chunk-x% %event-chunk-z% from %event-previous-chunk-x%\" to player\n";
    let vals = vec![
        Value::Num(1.0),
        Value::Num(-2.0),
        Value::Num(0.0),
        Value::Num(-2.0),
    ];
    let (calls, _) = run4(src, ExtEvent::ChunkEnter, Some(steve()), vals);
    assert_eq!(calls, ["send Steve 1 -2 from 0"]);
}

#[test]
fn past_event_item_in_tool_change() {
    let src = "on tool change:\n    send \"%event-item% / %past event-item%\" to player\n";
    let it = |id: &str| Value::Item {
        id: id.into(),
        count: 1,
        meta: None,
    };
    let vals = vec![Value::Num(0.0), Value::Num(1.0), it("stone"), it("dirt")];
    let (calls, _) = run4(src, ExtEvent::ToolChange, Some(steve()), vals);
    assert_eq!(calls.len(), 1);
    assert!(
        calls[0].contains("stone") && calls[0].contains("dirt"),
        "{calls:?}"
    );
}

#[test]
fn armor_change_values() {
    let src = "on armor change:\n    send \"%event-slot% %event-item%\" to player\n";
    let vals = vec![
        Value::Text("helmet".into()),
        Value::Item {
            id: "iron_helmet".into(),
            count: 1,
            meta: None,
        },
        Value::None,
    ];
    let (calls, _) = run4(src, ExtEvent::ArmorChange, Some(steve()), vals);
    assert!(calls[0].starts_with("send Steve helmet iron helmet"), "{calls:?}");
}

#[test]
fn world_border_values() {
    let src =
        "on world border bounds change:\n    send \"%event-old-size% -> %event-new-size%\" to console\n";
    let vals = vec![Value::World("world".into()), Value::Num(100.0), Value::Num(50.0)];
    let (calls, _) = run4(src, ExtEvent::BorderBounds, None, vals);
    assert_eq!(calls, ["send console 100 -> 50"]);
}

#[test]
fn unload_and_stop_run() {
    let (calls, _) = run4(
        "on unload:\n    send \"bye\" to console\n",
        ExtEvent::ScriptUnload,
        None,
        vec![],
    );
    assert_eq!(calls, ["send console bye"]);
    let (calls, _) = run4(
        "on server stop:\n    send \"stopping\" to console\n",
        ExtEvent::ServerStop,
        None,
        vec![],
    );
    assert_eq!(calls, ["send console stopping"]);
}

#[test]
fn can_build_set_boolean() {
    let src = "on can build check:\n    set event-boolean to false\n";
    let mut e = load(src);
    let mut h = Mock::default();
    let ctx = EventCtx {
        ev: vec![Value::None, Value::None, Value::Bool(true)],
        ..EventCtx::player(steve())
    };
    let o = e.fire(
        EventKind::Ext(carve_core::w2_events_1::ExtEvent::CanBuild),
        ctx,
        &mut h,
    );
    assert_eq!(o.set, vec![(0, Value::Bool(false))]);
    assert!(!errors("on join:\n    set event-boolean to false\n").is_empty());
}

#[test]
fn ev3_location_and_entity_values() {
    let loc = Value::Location {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        world: Some("world".into()),
    };
    let mut e = load("on piston extend:\n    send \"at %event-location%\" to console\n");
    let mut h = Mock::default();
    let mut c = EventCtx::console();
    c.ev = vec![loc];
    e.fire(EventKind::Ext3(Ev3::PistonExtend), c, &mut h);
    assert_eq!(h.calls.len(), 1);
    assert!(
        h.calls[0].contains('1') && h.calls[0].contains('3'),
        "{:?}",
        h.calls
    );

    let mut e = load("on leash:\n    send \"leashed %type of event-entity%\" to console\n");
    let mut h = Mock::default();
    let mut c = EventCtx::player(steve());
    c.ev = vec![Value::Entity(EntityRef {
        uuid: "x".into(),
        kind: "cow".into(),
    })];
    e.fire(EventKind::Ext3(Ev3::Leash), c, &mut h);
    assert_eq!(h.calls, ["send console leashed cow"]);
    assert_eq!(Ev3::PiglinBarter.fields().len(), 2);
}
