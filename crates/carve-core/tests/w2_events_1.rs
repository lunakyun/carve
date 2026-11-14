use carve_core::compile::check_registry;
use carve_core::w2_events_1::ExtEvent;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        match to {
            Recipient::Player(p) => self.calls.push(format!("send {} {}", p.name, text)),
            Recipient::Console => self.calls.push(format!("console {text}")),
        }
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

fn loc(x: f64, y: f64, z: f64) -> Value {
    Value::Location {
        x,
        y,
        z,
        world: Some("world".into()),
    }
}

fn item(id: &str) -> Value {
    Value::Item {
        id: id.into(),
        count: 1,
    }
}

fn world() -> Value {
    Value::World("world".into())
}

fn engine(src: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    e
}

fn errors_of(src: &str) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    errs.iter().map(|e| e.to_string()).collect()
}

/// Fire `ev` with a player (or none) and the given values. Returns the host
/// calls and whether the event was cancelled.
fn fire(src: &str, ev: ExtEvent, player: bool, vals: Vec<Value>) -> (Vec<String>, bool) {
    let mut e = engine(src);
    let mut h = Mock::default();
    let mut ctx = if player {
        EventCtx::player(steve())
    } else {
        EventCtx::console()
    };
    ctx.ev = vals;
    let out = e.fire(EventKind::Ext(ev), ctx, &mut h);
    let errs: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errs.is_empty(), "runtime errors: {errs:?}");
    (h.calls, out.cancelled)
}

#[test]
fn registry_still_compiles() {
    check_registry().unwrap();
}

#[test]
fn every_event_name_parses_and_has_a_trigger() {
    let names = [
        "anvil prepare",
        "arm swing",
        "bed enter",
        "bed leave",
        "bell ring",
        "block damage",
        "block drop",
        "block fertilize",
        "block growth",
        "book edit",
        "book sign",
        "brewing complete",
        "brewing fuel",
        "brewing start",
        "broadcast",
        "bucket catch entity",
        "bucket empty",
        "bucket fill",
        "burn",
        "can build check",
        "chunk generate",
        "chunk unload",
        "combust",
        "connect",
        "dispense",
        "egg throw",
        "elytra boost",
    ];
    for n in names {
        let src = format!("on {n}:\n    stop\n");
        let mut e = Engine::default();
        let (_, errs) = e.load(&[("t.carve".into(), src)]);
        assert!(errs.is_empty(), "{n}: {errs:?}");
        assert_eq!(e.table().triggers, 1, "{n}");
    }
    assert!(ExtEvent::parse("nonsense").is_none());
}

#[test]
fn anvil_prepare_values() {
    let (c, cancelled) = fire(
        "on anvil prepare:\n    send \"%event-text% %event-number%\" to player\n",
        ExtEvent::AnvilPrepare,
        true,
        vec![Value::Text("Excalibur".into()), Value::Num(5.0)],
    );
    assert_eq!(c, ["send Steve Excalibur 5"]);
    assert!(!cancelled);
}

#[test]
fn anvil_prepare_cannot_be_cancelled() {
    let e = errors_of("on anvil prepare:\n    cancel event\n");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].contains("can't be cancelled"), "{e:?}");
}

#[test]
fn bed_enter_cancel() {
    let (c, cancelled) = fire(
        "on bed enter:\n    send \"night\" to player\n    cancel event\n",
        ExtEvent::BedEnter,
        true,
        vec![loc(1.0, 64.0, 2.0)],
    );
    assert_eq!(c, ["send Steve night"]);
    assert!(cancelled);
}

#[test]
fn bed_leave_location() {
    let (c, _) = fire(
        "on bed leave:\n    set {_l} to event-location\n    send \"%{_l}%\" to player\n",
        ExtEvent::BedLeave,
        true,
        vec![loc(1.0, 64.0, 2.0)],
    );
    assert_eq!(c.len(), 1);
    assert!(c[0].contains("64"), "{c:?}");
}

#[test]
fn block_growth_old_and_new_block() {
    let (c, cancelled) = fire(
        "on block growth:\n    broadcast \"%event-old-block% to %event-block%\"\n    cancel event\n",
        ExtEvent::BlockGrowth,
        false,
        vec![loc(0.0, 0.0, 0.0), world(), item("carrots"), item("wheat")],
    );
    assert_eq!(c.len(), 1);
    assert!(c[0].contains("wheat") && c[0].contains("carrots"), "{c:?}");
    assert!(cancelled);
}

#[test]
fn block_drop_has_optional_player() {
    let src = "on block drop:\n    broadcast \"%event-number%\"\n    if player is set:\n        broadcast \"by player\"\n";
    let vals = vec![loc(0.0, 0.0, 0.0), world(), item("dirt"), Value::Num(2.0)];
    let (c, _) = fire(src, ExtEvent::BlockDrop, false, vals.clone());
    assert_eq!(c, ["broadcast 2"]);
    let (c, _) = fire(src, ExtEvent::BlockDrop, true, vals);
    assert_eq!(c, ["broadcast 2", "broadcast by player"]);
}

#[test]
fn broadcast_event_text_and_cancel() {
    let src = "on broadcast:\n    if event-text contains \"secret\":\n        cancel event\n";
    let (c, cancelled) = fire(
        src,
        ExtEvent::Broadcast,
        false,
        vec![Value::Text("a secret".into()), Value::Text("Server".into())],
    );
    assert!(c.is_empty());
    assert!(cancelled);
    let (_, cancelled) = fire(
        src,
        ExtEvent::Broadcast,
        false,
        vec![Value::Text("hello".into()), Value::Text("Server".into())],
    );
    assert!(!cancelled);
}

#[test]
fn bucket_events() {
    let (c, _) = fire(
        "on bucket fill:\n    broadcast \"%event-item%\"\n",
        ExtEvent::BucketFill,
        true,
        vec![loc(1.0, 2.0, 3.0), item("water_bucket")],
    );
    assert_eq!(c.len(), 1);
    assert!(c[0].contains("water bucket"), "{c:?}");
    let (c, cancelled) = fire(
        "on bucket empty:\n    cancel event\n",
        ExtEvent::BucketEmpty,
        true,
        vec![loc(1.0, 2.0, 3.0), item("lava_bucket")],
    );
    assert!(c.is_empty());
    assert!(cancelled);
    let (c, _) = fire(
        "on bucket catch entity:\n    send \"%event-item%\" to player\n",
        ExtEvent::BucketCatch,
        true,
        vec![item("axolotl_bucket"), Value::Num(7.0)],
    );
    assert!(c[0].contains("axolotl bucket"), "{c:?}");
}

#[test]
fn book_edit_and_sign_are_separate() {
    let src = "on book edit:\n    broadcast \"edit %event-text% %event-number%\"\non book sign:\n    broadcast \"sign\"\n";
    let mut e = engine(src);
    let mut h = Mock::default();
    let mut ctx = EventCtx::player(steve());
    ctx.ev = vec![Value::Text("Title".into()), Value::Num(3.0), Value::Text("p1".into())];
    e.fire(EventKind::Ext(ExtEvent::BookEdit), ctx.clone(), &mut h);
    e.fire(EventKind::Ext(ExtEvent::BookSign), ctx, &mut h);
    assert_eq!(h.calls, ["broadcast edit Title 3", "broadcast sign"]);
}

#[test]
fn brewing_events() {
    let (c, _) = fire(
        "on brewing fuel:\n    broadcast \"%event-number%\"\n",
        ExtEvent::BrewingFuel,
        false,
        vec![loc(0.0, 0.0, 0.0), Value::Num(20.0)],
    );
    assert_eq!(c, ["broadcast 20"]);
    let (_, cancelled) = fire(
        "on brewing start:\n    cancel event\n",
        ExtEvent::BrewingStart,
        false,
        vec![loc(0.0, 0.0, 0.0), world(), Value::Num(400.0)],
    );
    assert!(cancelled);
    let (c, _) = fire(
        "on brewing complete:\n    broadcast \"done %event-number%\"\n",
        ExtEvent::BrewingComplete,
        false,
        vec![loc(0.0, 0.0, 0.0), Value::Num(2.0)],
    );
    assert_eq!(c, ["broadcast done 2"]);
}

#[test]
fn burn_and_can_build() {
    let (c, cancelled) = fire(
        "on burn:\n    broadcast \"%event-block% by %event-igniting-block%\"\n    cancel event\n",
        ExtEvent::Burn,
        false,
        vec![loc(0.0, 0.0, 0.0), world(), item("oak_planks"), item("fire")],
    );
    assert!(c[0].contains("oak planks") && c[0].contains("fire"), "{c:?}");
    assert!(cancelled);
    let (c, cancelled) = fire(
        "on can build check:\n    if event-boolean is true:\n        broadcast \"can\"\n    if event-block is tnt:\n        cancel event\n",
        ExtEvent::CanBuild,
        true,
        vec![item("tnt"), item("stone"), Value::Bool(true)],
    );
    assert_eq!(c, ["broadcast can"]);
    assert!(cancelled);
}

#[test]
fn chunk_events_numbers() {
    let (c, _) = fire(
        "on chunk generate:\n    broadcast \"%event-chunk-x%,%event-chunk-z%\"\n",
        ExtEvent::ChunkGenerate,
        false,
        vec![Value::Num(3.0), Value::Num(-4.0)],
    );
    assert_eq!(c, ["broadcast 3,-4"]);
    let (c, _) = fire(
        "on chunk unload:\n    broadcast \"%event-chunk-x%\"\n",
        ExtEvent::ChunkUnload,
        false,
        vec![Value::Num(9.0), Value::Num(0.0)],
    );
    assert_eq!(c, ["broadcast 9"]);
}

#[test]
fn combust_connect_dispense_egg_elytra_swing_bell_damage_fertilize() {
    let (c, _) = fire(
        "on combust:\n    broadcast \"%event-number%\"\n",
        ExtEvent::Combust,
        false,
        vec![Value::Num(12.0), Value::Timespan { millis: 8000 }],
    );
    assert_eq!(c, ["broadcast 12"]);
    let (c, cancelled) = fire(
        "on connect:\n    broadcast \"hi %player%\"\n    cancel event\n",
        ExtEvent::Connect,
        true,
        vec![],
    );
    assert_eq!(c, ["broadcast hi Steve"]);
    assert!(cancelled);
    let (c, _) = fire(
        "on dispense:\n    broadcast \"%event-item%\"\n",
        ExtEvent::Dispense,
        false,
        vec![loc(0.0, 0.0, 0.0), item("arrow")],
    );
    assert!(c[0].contains("arrow"));
    let (c, _) = fire(
        "on egg throw:\n    if event-boolean is true:\n        broadcast \"hatch %event-number%\"\n",
        ExtEvent::EggThrow,
        true,
        vec![Value::Bool(true), Value::Num(3.0), Value::Text("chicken".into())],
    );
    assert_eq!(c, ["broadcast hatch 3"]);
    let (c, cancelled) = fire(
        "on elytra boost:\n    cancel event\n    broadcast \"%event-number%\"\n",
        ExtEvent::ElytraBoost,
        true,
        vec![Value::Num(44.0)],
    );
    assert_eq!(c, ["broadcast 44"]);
    assert!(cancelled);
    let (c, _) = fire("on arm swing:\n    send \"swing\" to player\n", ExtEvent::ArmSwing, true, vec![]);
    assert_eq!(c, ["send Steve swing"]);
    let (c, _) = fire(
        "on bell ring:\n    broadcast \"%event-text%\"\n",
        ExtEvent::BellRing,
        false,
        vec![loc(0.0, 0.0, 0.0), world(), Value::None, Value::Text("north".into())],
    );
    assert_eq!(c, ["broadcast north"]);
    let (c, _) = fire(
        "on block damage:\n    if event-boolean is true:\n        broadcast \"insta\"\n",
        ExtEvent::BlockDamage,
        true,
        vec![loc(0.0, 0.0, 0.0), Value::Bool(true)],
    );
    assert_eq!(c, ["broadcast insta"]);
    let (c, _) = fire(
        "on block fertilize:\n    broadcast \"%event-number%\"\n",
        ExtEvent::BlockFertilize,
        false,
        vec![loc(0.0, 0.0, 0.0), world(), Value::Num(5.0)],
    );
    assert_eq!(c, ["broadcast 5"]);
}

#[test]
fn unknown_event_value_is_an_error() {
    let e = errors_of("on bed leave:\n    broadcast \"%event-number%\"\n");
    assert!(!e.is_empty(), "{e:?}");
}

#[test]
fn no_player_in_server_events() {
    let e = errors_of("on broadcast:\n    send \"x\" to player\n");
    assert!(!e.is_empty(), "{e:?}");
}

#[test]
fn old_events_unchanged() {
    // `event-block` works in click events (it is now lexed as one word).
    let src = "on right click:\n    if event-block is stone:\n        send \"yes\" to player\n";
    let mut e = engine(src);
    let mut h = Mock::default();
    let mut ctx = EventCtx::player(steve());
    ctx.block = Some("stone".into());
    e.fire(EventKind::RightClick, ctx, &mut h);
    assert_eq!(h.calls, ["send Steve yes"]);
}

#[test]
fn at_time_parse_and_fire() {
    let src = "at 18:00:\n    broadcast \"night\"\nat 6am:\n    broadcast \"morning\"\n";
    let mut e = engine(src);
    assert!(e.table().has_at());
    let mut h = Mock::default();
    e.fire_at_time(11998, 11999, &mut h);
    assert!(h.calls.is_empty());
    e.fire_at_time(11999, 12000, &mut h);
    assert_eq!(h.calls, ["broadcast night"]);
    h.calls.clear();
    e.fire_at_time(12000, 12001, &mut h);
    assert!(h.calls.is_empty());
    e.fire_at_time(23999, 1, &mut h);
    assert_eq!(h.calls, ["broadcast morning"]);
}

#[test]
fn at_time_errors() {
    let e = errors_of("at 99:99:\n    stop\n");
    assert_eq!(e.len(), 1, "{e:?}");
    let e = errors_of("at noon:\n    send \"x\" to player\n");
    assert!(!e.is_empty(), "{e:?}");
}
