use carve_core::compile::{EventKind, compile_file};
use carve_core::w2_events_4::{self as ev, ExtEvent};
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    clock: i64,
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
    fn execute_command(&mut self, _: &str) {}
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
    fn wall_clock_millis(&mut self) -> i64 {
        self.clock
    }
    fn log(&mut self, _: LogLevel, _: &str) {}
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
    assert!(
        errs.is_empty(),
        "unexpected errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn compile_errors(src: &str) -> Vec<String> {
    compile_file("bad.carve", src)
        .err()
        .map(|e| e.iter().map(|x| x.to_string()).collect())
        .unwrap_or_default()
}

fn fire(e: &mut Engine, h: &mut Mock, k: ExtEvent, p: Option<PlayerRef>, vals: Vec<Value>) -> FireOutcome {
    e.fire(EventKind::Ext(k), ev::ctx(p, vals), h)
}

fn sent(h: &Mock) -> Vec<String> {
    h.calls
        .iter()
        .map(|c| c.replace("send console ", "").replace("broadcast ", "B:"))
        .collect()
}

fn loc(x: f64, y: f64, z: f64) -> Value {
    Value::Location {
        x,
        y,
        z,
        world: None,
    }
}

#[test]
fn every_header_parses() {
    for name in [
        "server list ping",
        "ping",
        "sheep regrow wool",
        "shoot",
        "sign change",
        "slime split",
        "smelt",
        "start smelt",
        "spawn change",
        "sponge absorb",
        "spread",
        "swim toggle",
        "tame",
        "target",
        "throwing of an egg",
        "tool change",
        "vault display item",
        "vehicle collision",
        "vehicle create",
        "vehicle damage",
        "vehicle destroy",
        "vehicle enter",
        "vehicle exit",
        "vehicle move",
        "villager career change",
        "weather change",
        "weather change to rain",
        "weather change to clear",
        "world init",
        "world load",
        "world save",
        "world unload",
        "zombie break door",
    ] {
        let src = format!("on {name}:\n    stop\n");
        assert!(compile_file("t.carve", &src).is_ok(), "on {name} should compile");
    }
}

#[test]
fn unknown_event_still_errors() {
    let errs = compile_errors("on banana split:\n    stop\n");
    assert_eq!(errs.len(), 1);
    assert!(errs[0].contains("unknown event"));
}

#[test]
fn ping_reads_and_sets_values() {
    let mut e = engine(
        "on server list ping:\n    set event-motd to \"Hi %event-hostname%\"\n    set event-max-players to event-players + 10\n",
    );
    let mut h = Mock::default();
    let o = fire(
        &mut e,
        &mut h,
        ExtEvent::ServerListPing,
        None,
        vec![
            Value::Text("old".into()),
            Value::Num(20.0),
            Value::Num(3.0),
            Value::Text("play.example".into()),
            Value::Text("1.2.3.4:5".into()),
        ],
    );
    assert!(!o.cancelled);
    assert_eq!(o.set.len(), 2);
    assert_eq!(o.set[0], (0, Value::Text("Hi play.example".into())));
    assert_eq!(o.set[1], (1, Value::Num(13.0)));
}

#[test]
fn ping_cannot_be_cancelled() {
    let errs = compile_errors("on server list ping:\n    cancel event\n");
    assert_eq!(errs.len(), 1);
    assert!(errs[0].contains("can't be cancelled"), "{errs:?}");
}

#[test]
fn read_only_value_cannot_be_set() {
    let errs = compile_errors("on server list ping:\n    set event-hostname to \"x\"\n");
    assert!(errs.iter().any(|m| m.contains("can't be changed")), "{errs:?}");
}

#[test]
fn unknown_value_lists_choices() {
    let errs = compile_errors("on smelt:\n    send \"%event-banana%\" to console\n");
    assert!(errs.iter().any(|m| m.contains("event-source")), "{errs:?}");
}

#[test]
fn sign_change_lines_and_cancel() {
    let mut e = engine(
        "on sign change:\n    send \"%player% wrote %event-line-1%\" to console\n    if event-line-2 contains \"bad\":\n        cancel event\n    set event-line-3 to \"[ok]\"\n",
    );
    let mut h = Mock::default();
    let vals = |l2: &str| {
        vec![
            loc(1.0, 2.0, 3.0),
            Value::Text("hello".into()),
            Value::Text(l2.into()),
            Value::Text(String::new()),
            Value::Text(String::new()),
        ]
    };
    let o = fire(&mut e, &mut h, ExtEvent::SignChange, Some(steve()), vals("fine"));
    assert!(!o.cancelled);
    assert_eq!(o.set, vec![(3, Value::Text("[ok]".into()))]);
    let o = fire(&mut e, &mut h, ExtEvent::SignChange, Some(steve()), vals("bad word"));
    assert!(o.cancelled);
    assert_eq!(sent(&h)[0], "Steve wrote hello");
}

#[test]
fn smelt_items_and_block() {
    let mut e = engine(
        "on smelt:\n    send \"%event-source% -> %event-result%\" to console\non start smelt:\n    set event-time to 10\n",
    );
    let mut h = Mock::default();
    fire(
        &mut e,
        &mut h,
        ExtEvent::Smelt,
        None,
        vec![
            loc(4.0, 5.0, 6.0),
            Value::Item {
                id: "iron_ore".into(),
                count: 1,
            },
            Value::Item {
                id: "iron_ingot".into(),
                count: 1,
            },
        ],
    );
    assert_eq!(sent(&h).len(), 1);
    assert!(sent(&h)[0].contains("iron ore") && sent(&h)[0].contains("iron ingot"), "{:?}", sent(&h));
    let o = fire(
        &mut e,
        &mut h,
        ExtEvent::StartSmelt,
        None,
        vec![Value::None, Value::None, Value::Num(200.0)],
    );
    assert_eq!(o.set, vec![(2, Value::Num(10.0))]);
}

#[test]
fn vehicle_enter_has_player_and_entities() {
    let mut e = engine(
        "on vehicle enter:\n    send \"%player% entered %event-vehicle%\" to console\n    cancel event\n",
    );
    let mut h = Mock::default();
    let o = fire(
        &mut e,
        &mut h,
        ExtEvent::VehicleEnter,
        Some(steve()),
        vec![Value::Num(77.0), Value::Player(steve())],
    );
    assert!(o.cancelled);
    assert_eq!(sent(&h), vec!["Steve entered 77"]);
}

#[test]
fn weather_values() {
    let mut e = engine(
        "on weather change to rain:\n    send \"rain in %event-world%\" to console\non weather change:\n    send \"any %event-weather%\" to console\n",
    );
    let mut h = Mock::default();
    let vals = vec![Value::World("world".into()), Value::Text("rain".into())];
    fire(&mut e, &mut h, ExtEvent::WeatherChange, None, vals.clone());
    fire(&mut e, &mut h, ExtEvent::WeatherToRain, None, vals);
    fire(
        &mut e,
        &mut h,
        ExtEvent::WeatherToClear,
        None,
        vec![Value::World("world".into()), Value::Text("clear".into())],
    );
    assert_eq!(sent(&h), vec!["any rain", "rain in world"]);
}

#[test]
fn egg_throw_changes_hatching() {
    let mut e = engine(
        "on throwing of an egg:\n    if event-hatching is true:\n        set event-hatching to false\n        set event-hatches to 0\n",
    );
    let mut h = Mock::default();
    let o = fire(
        &mut e,
        &mut h,
        ExtEvent::EggThrow,
        Some(steve()),
        vec![Value::Bool(true), Value::Num(1.0), Value::Text("chicken".into())],
    );
    assert_eq!(o.set, vec![(0, Value::Bool(false)), (1, Value::Num(0.0))]);
}

#[test]
fn world_events_value() {
    let mut e = engine(
        "on world load:\n    send \"loaded %event-world%\" to console\non world unload:\n    cancel event\n",
    );
    let mut h = Mock::default();
    fire(&mut e, &mut h, ExtEvent::WorldLoad, None, vec![Value::World("nether".into())]);
    assert_eq!(sent(&h), vec!["loaded nether"]);
    let o = fire(&mut e, &mut h, ExtEvent::WorldUnload, None, vec![Value::World("x".into())]);
    assert!(o.cancelled);
}

#[test]
fn tool_change_slots() {
    let mut e = engine("on tool change:\n    send \"%event-previous-slot% to %event-new-slot%\" to console\n");
    let mut h = Mock::default();
    fire(
        &mut e,
        &mut h,
        ExtEvent::ToolChange,
        Some(steve()),
        vec![Value::Num(0.0), Value::Num(4.0)],
    );
    assert_eq!(sent(&h), vec!["0 to 4"]);
}

#[test]
fn event_values_do_not_leak_to_other_events() {
    let errs = compile_errors("on join:\n    send \"%event-motd%\" to console\n");
    assert!(!errs.is_empty());
}

#[test]
fn at_real_time_headers() {
    assert_eq!(ev::parse_at_real_time("at 14:30 in real time"), Some(14 * 60 + 30));
    assert_eq!(ev::parse_at_real_time("at 6pm in real time"), Some(18 * 60));
    assert_eq!(ev::parse_at_real_time("at 12am in real time"), Some(0));
    assert_eq!(ev::parse_at_real_time("at 12pm in real time"), Some(12 * 60));
    assert_eq!(ev::parse_at_real_time("at 25:00 in real time"), None);
    assert_eq!(ev::parse_at_real_time("at 10:61 in real time"), None);
    assert_eq!(ev::parse_at_real_time("at 10:00"), None);
}

#[test]
fn system_time_fires_once_per_minute() {
    let mut e = engine(
        "at 12:30 in real time:\n    broadcast \"noon-ish\"\nat 12:31 in real time:\n    send \"later\" to console\n",
    );
    let day = 1_794_614_400_000i64;
    assert_eq!(day.rem_euclid(86_400_000), 0);
    let mut h = Mock::default();
    let mut last = 0i64;
    // First poll only records the minute.
    h.clock = day + (12 * 60 + 29) * 60_000 + 5_000;
    ev::poll_system_time(&mut e, &mut last, &mut h);
    assert!(h.calls.is_empty());
    // Same minute again: nothing.
    h.clock += 10_000;
    ev::poll_system_time(&mut e, &mut last, &mut h);
    assert!(h.calls.is_empty());
    // 12:30, polled twice in the same minute: runs once.
    h.clock = day + (12 * 60 + 30) * 60_000 + 1_000;
    ev::poll_system_time(&mut e, &mut last, &mut h);
    h.clock += 20_000;
    ev::poll_system_time(&mut e, &mut last, &mut h);
    assert_eq!(sent(&h), vec!["B:noon-ish"]);
    // 12:31.
    h.clock = day + (12 * 60 + 31) * 60_000;
    ev::poll_system_time(&mut e, &mut last, &mut h);
    assert_eq!(sent(&h), vec!["B:noon-ish", "later"]);
}

#[test]
fn bad_at_time_is_an_error() {
    let errs = compile_errors("at 99:00 in real time:\n    stop\n");
    assert!(!errs.is_empty());
}

#[test]
fn event_values_work_with_lists_and_vars() {
    let mut e = engine(
        "on spread:\n    set {_n} to 1\n    add 1 to {_n}\n    send \"%{_n}% %event-world%\" to console\n",
    );
    let mut h = Mock::default();
    fire(
        &mut e,
        &mut h,
        ExtEvent::Spread,
        None,
        vec![Value::None, Value::None, Value::World("w".into())],
    );
    assert_eq!(sent(&h), vec!["2 w"]);
}
